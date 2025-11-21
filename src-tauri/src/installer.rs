use std::{
    cmp::Ordering,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::downloader::{move_file, DownloadManager, DownloadProgress};
use crate::launcher::{LauncherProfile, LauncherProfiles};
use crate::state::{InstallerState, ModLoaderState, ModState, ResourceState};
use crate::{
    config::{ModPackConfig, ResourceEntry},
    APP_FOLDER_NAME,
};

const STATE_FILE_NAME: &str = "installer-state.json";
const TEMP_DIR_NAME: &str = ".temp";

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum InstallerMode {
    Install,
}

pub struct Installer {
    app: AppHandle,
    download_manager: DownloadManager,
    config: ModPackConfig,
    install_dir: PathBuf,
    temp_dir: PathBuf,
    state_path: PathBuf,
}

impl Installer {
    pub fn new(app: AppHandle, config_path: PathBuf, install_dir: PathBuf) -> Result<Self> {
        let app_dir = install_dir.join(APP_FOLDER_NAME);
        Ok(Self {
            app,
            download_manager: DownloadManager::new()?,
            config: ModPackConfig::load_from_path(&config_path)?,
            install_dir: install_dir.clone(),
            temp_dir: app_dir.join(TEMP_DIR_NAME),
            state_path: app_dir.join(STATE_FILE_NAME),
        })
    }

    pub fn can_install() -> bool {
        return PathBuf::from("config.yaml").exists();
    }

    pub fn run(mut self, mode: InstallerMode) -> Result<()> {
        self.emit_progress(0.);
        match mode {
            InstallerMode::Install => self.run_install(),
        }
    }

    fn run_install(&mut self) -> Result<()> {
        log::info!("Starting installation...");
        self.prepare_temp_dir()?;
        let mut state =
            InstallerState::load_or_new(&self.state_path, &self.app.package_info().version)?;
        let total_steps = self.total_download_steps();
        let mut completed_steps = 0u32;
        // Download Mod loader
        self.emit_change_phase(Phase::DownloadModLoader);
        let loader_config = &self.config.mod_loader;
        if let Some(downloaded_loader) = state.get_mod_loader() {
            if !downloaded_loader.equals(loader_config) {
                log::error!(
                    "Mod loader {} is downloaded, but uploaded file was changed. Skipping.",
                    loader_config.name
                );
            } else {
                log::info!(
                    "Mod loader {} is already downloaded, skipping download.",
                    loader_config.name
                );
            }
        } else {
            let file_name = self.ensure_download(
                &loader_config.url,
                &loader_config.name,
                &loader_config.hash,
                &self.install_dir,
                completed_steps,
                total_steps,
            )?;
            state.set_mod_loader(ModLoaderState {
                file_name,
                url: loader_config.url.clone(),
                hash: loader_config.hash.clone(),
            });
            state.save(&self.state_path)?;
        }
        completed_steps += 1u32;
        self.emit_progress(completed_steps as f32 / total_steps as f32);
        // Mods
        self.emit_change_phase(Phase::DownloadMods);
        let mods_dir = self.get_mods_dir();
        for mod_entry in &self.config.mods {
            let needs_download = state.get_mod(mod_entry).map_or(true, |downloaded_mod| {
                if !downloaded_mod.equals(mod_entry) {
                    log::warn!(
                        "Mod {} is downloaded, but uploaded file was changed.",
                        mod_entry.name
                    );
                    true
                } else {
                    log::info!(
                        "Mod {} is already downloaded, skipping download.",
                        mod_entry.name
                    );
                    false
                }
            });
            if needs_download {
                let url = mod_entry.source.get_download_url();
                let file_name = self.ensure_download(
                    &url,
                    &mod_entry.name,
                    &mod_entry.hash,
                    &mods_dir,
                    completed_steps,
                    total_steps,
                )?;
                state.add_mod(ModState {
                    file_name,
                    source: mod_entry.source.clone(),
                    hash: mod_entry.hash.clone(),
                });
                state.save(&self.state_path)?;
            }
            completed_steps += 1u32;
            self.emit_progress(completed_steps as f32 / total_steps as f32);
        }
        // Resources
        self.emit_change_phase(Phase::DownloadResources);
        for resource_entry in &self.config.resources {
            let needs_download =
                state
                    .get_resource(resource_entry)
                    .map_or(true, |downloaded_resource| {
                        if !downloaded_resource.equals(resource_entry) {
                            log::warn!(
                                "Resource {} is downloaded, but uploaded file was changed.",
                                resource_entry.name
                            );
                            true
                        } else {
                            log::info!(
                                "Resource {} is already downloaded, skipping download.",
                                resource_entry.name
                            );
                            false
                        }
                    });
            if needs_download {
                let url = resource_entry.source.get_download_url();
                let target_dir = self.get_resource_dir(resource_entry);
                let file_name = self.ensure_download(
                    &url,
                    &resource_entry.name,
                    &resource_entry.hash,
                    &target_dir,
                    completed_steps,
                    total_steps,
                )?;
                state.add_resource(ResourceState {
                    file_name,
                    source: resource_entry.source.clone(),
                    hash: resource_entry.hash.clone(),
                    target_dir: resource_entry.target_dir.clone(),
                });
                state.save(&self.state_path)?;
            }
            completed_steps += 1u32;
            self.emit_progress(completed_steps as f32 / total_steps as f32);
        }
        debug_assert_eq!(completed_steps, total_steps);
        self.emit_progress(1.);
        // Add profile to launcher
        self.emit_change_phase(Phase::AddProfile);
        if let Err(e) = self.add_launcher_profile() {
            log::warn!("Failed to add launcher profile: {e:?}");
            self.emit_add_alert(AlertLevel::Warning, "alertOnFailedAddProfile");
        }
        // Auto-open mod loader if configured
        if self.config.mod_loader.auto_open {
            self.emit_change_phase(Phase::LaunchModLoader);
            if let Err(e) = self.launch_mod_loader() {
                log::warn!("Failed to launch mod loader: {e:?}");
                self.emit_add_alert(AlertLevel::Warning, "alertOnFailedLaunchModLoader");
            }
        }
        log::info!("Installation completed.");

        Ok(())
    }

    fn prepare_temp_dir(&self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).with_context(|| {
                format!("Failed to wipe temp directory {}", self.temp_dir.display())
            })?;
        }
        fs::create_dir_all(&self.temp_dir).with_context(|| {
            format!(
                "Failed to create temp directory {}",
                self.temp_dir.display()
            )
        })?;
        Ok(())
    }

    fn total_download_steps(&self) -> u32 {
        let mut steps = 0u32;
        steps += 1; // Mod Loader
        steps += self.config.mods.len() as u32;
        steps += self.config.resources.len() as u32;
        steps
    }

    fn ensure_download(
        &self,
        url: &str,
        name: &str,
        expected_hash: &str,
        final_dir: &Path,
        completed_steps: u32,
        total_steps: u32,
    ) -> Result<String> {
        log::info!("Downloading {name} from {url}...");
        self.emit_change_detail(name);
        let outcome = self.download_manager.download_to_dir(
            url,
            &self.temp_dir,
            Some(move |progress: DownloadProgress| -> Result<()> {
                if progress.total_bytes.is_none() {
                    return Ok(());
                }
                let total = progress.total_bytes.unwrap();
                let fraction = if total != 0 {
                    progress.received_bytes as f32 / total as f32
                } else {
                    0.0
                };
                self.emit_progress((completed_steps as f32 + fraction) / total_steps as f32);
                Ok(())
            }),
        )?;
        let file_name = outcome
            .path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Could not extract file name from downloaded file"))?;
        let final_path = final_dir.join(&file_name);
        verify_hash(expected_hash, &outcome.hash, &final_path)?;
        move_file(&outcome.path, &final_path)?;
        self.emit_progress((completed_steps + 1) as f32 / total_steps as f32);
        log::info!("Downloaded {name}.");

        Ok(file_name.to_string_lossy().to_string())
    }

    fn get_mods_dir(&self) -> PathBuf {
        self.install_dir.join("mods")
    }

    fn get_resource_dir(&self, entry: &ResourceEntry) -> PathBuf {
        self.install_dir.join(&entry.target_dir)
    }

    fn emit_change_phase(&self, phase: Phase) {
        emit_event(
            &self.app,
            InstallerEvent::ChangePhase(ChangePhasePayload { phase: phase }),
        );
    }

    fn emit_change_detail(&self, detail: &str) {
        emit_event(
            &self.app,
            InstallerEvent::ChangeDetail(ChangeDetailPayload {
                detail: detail.to_string(),
            }),
        );
    }

    fn emit_progress(&self, progress: f32) {
        emit_event(
            &self.app,
            InstallerEvent::UpdateProgress(UpdateProgressPayload { progress }),
        );
    }

    fn emit_add_alert(&self, level: AlertLevel, translation_key: &str) {
        emit_event(
            &self.app,
            InstallerEvent::AddAlert(AddAlertPayload {
                level,
                translation_key: translation_key.to_string(),
            }),
        );
    }

    fn add_launcher_profile(&self) -> Result<()> {
        if !cfg!(target_os = "windows") {
            bail!("Adding launcher profile is only supported on Windows.");
        }
        log::info!("Adding launcher profile...");
        let appdata = env::var("APPDATA").context("APPDATA environment variable not found")?;
        let profiles_path = PathBuf::from(appdata)
            .join(".minecraft")
            .join("launcher_profiles.json");
        if !profiles_path.exists() {
            bail!("Launcher profiles file not found. ");
        }
        // Load existing profiles
        let content =
            fs::read_to_string(&profiles_path).context("Failed to read launcher_profiles.json")?;
        let mut launcher_profiles: LauncherProfiles =
            serde_json::from_str(&content).context("Failed to parse launcher_profiles.json")?;
        // Check if profile already exists
        for profile in launcher_profiles.profiles.values() {
            if profile.name == self.config.profile.name {
                log::info!(
                    "Launcher profile '{}' already exists, skipping addition.",
                    profile.name
                );
                return Ok(());
            }
        }
        // Insert new profile
        let profile_id = uuid::Uuid::new_v4().simple().to_string();
        let now = chrono::Utc::now();
        let now_rounded =
            chrono::DateTime::from_timestamp_millis(now.timestamp_millis()).unwrap_or(now);
        let new_profile = LauncherProfile {
            created: Some(now_rounded),
            game_dir: Some(self.install_dir.clone()),
            icon: self.config.profile.icon.clone(),
            java_args: self.config.profile.jvm_args.clone(),
            java_dir: None,
            last_used: Some(now_rounded),
            last_version_id: self.config.profile.version.clone(),
            name: self.config.profile.name.clone(),
            resolution: None,
            skip_jre_version_check: None,
            profile_type: "custom".to_string(),
        };
        if launcher_profiles
            .profiles
            .insert(profile_id.clone(), new_profile)
            .is_some()
        {
            bail!("Profile ID '{profile_id}' already exists in launcher profiles");
        }
        // Backup original file
        let mut backup_path = profiles_path.with_extension("json.bak");
        let mut backup_index = 1;
        while backup_path.exists() {
            backup_path = profiles_path.with_extension(format!("json.bak{backup_index}"));
            backup_index += 1;
        }
        fs::rename(&profiles_path, &backup_path)
            .context("Failed to backup launcher_profiles.json")?;
        log::info!(
            "Backed up launcher_profiles.json to {}",
            backup_path.display()
        );
        // Save profiles
        let profiles_json = serde_json::to_string_pretty(&launcher_profiles)
            .context("Failed to serialize profiles")?;
        fs::write(&profiles_path, profiles_json)
            .context("Failed to write launcher_profiles.json")?;
        log::info!("Added profile '{}' to launcher.", self.config.profile.name);

        Ok(())
    }

    fn launch_mod_loader(&self) -> Result<()> {
        log::info!("Launching mod loader...");
        // Find mod loader jar file
        let jar_files: Vec<_> = fs::read_dir(&self.install_dir)
            .context("Failed to read install directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("jar"))
                    .unwrap_or(false)
            })
            .collect();
        if jar_files.is_empty() {
            bail!("Mod loader installer JAR file not found.");
        }
        let jar_path = &jar_files.first().unwrap().path();
        log::info!("Found mod loader: {}", jar_path.display());
        // Find Java executable
        let java_exe = find_java().ok_or_else(|| anyhow!("Java executable not found"))?;
        log::info!("Using Java: {}", java_exe.display());
        // Launch jar file
        let mut command = Command::new(java_exe);
        command
            .arg("-jar")
            .arg(jar_path)
            .current_dir(&self.install_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
        }
        command
            .spawn()
            .context("Failed to launch mod loader installer")?;
        self.emit_add_alert(AlertLevel::Info, "alertOnLaunchModLoader");
        log::info!("Launched mod loader installer.");

        Ok(())
    }
}

fn find_java() -> Option<PathBuf> {
    // 1. Check system java command
    log::info!("Searching for system java...");
    match Command::new(if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    })
    .arg("java")
    .output()
    {
        Ok(output) => {
            if output.status.success() {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let path = PathBuf::from(path_str.trim());
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
        Err(error) => {
            log::warn!("Failed to find system java: {error:?}");
        }
    }
    // 2. Check Minecraft Launcher App runtime
    if cfg!(target_os = "windows") {
        log::info!("Searching for java from minecraft...");
        match env::var("LOCALAPPDATA") {
            Ok(local_appdata) => {
                let runtimes_dir = PathBuf::from(local_appdata)
                    .join("Packages")
                    .join("Microsoft.4297127D64EC6_8wekyb3d8bbwe")
                    .join("LocalCache")
                    .join("Local")
                    .join("runtime");
                if let Some(java) = search_runtime_dir(&runtimes_dir) {
                    return Some(java);
                }
            }
            Err(error) => {
                log::warn!("LOCALAPPDATA environment variable not found: {error:?}");
            }
        }
    }

    None
}

fn search_runtime_dir(runtime_dir: &Path) -> Option<PathBuf> {
    if !runtime_dir.exists() {
        return None;
    }
    let mut dirs: Vec<_> = fs::read_dir(runtime_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    dirs.sort_by(|a, b| {
        let name_a = a.file_name();
        let name_b = b.file_name();
        let a_is_newer_java = name_a.to_string_lossy().starts_with("java-runtime-");
        let b_is_newer_java = name_b.to_string_lossy().starts_with("java-runtime-");
        match (a_is_newer_java, b_is_newer_java) {
            (true, false) => Ordering::Less,    // java-runtime-* comes first
            (false, true) => Ordering::Greater, // jre-* comes later
            _ => name_b.cmp(&name_a),           // Among same type, reverse order (newer first)
        }
    });
    for entry in dirs {
        let path = entry.path();
        let java_exe = if cfg!(target_os = "windows") {
            path.join("bin").join("javaw.exe")
        } else {
            path.join("bin").join("java")
        };
        if java_exe.exists() {
            return Some(java_exe);
        }
    }

    None
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum InstallerEvent {
    ChangePhase(ChangePhasePayload),
    ChangeDetail(ChangeDetailPayload),
    UpdateProgress(UpdateProgressPayload),
    AddAlert(AddAlertPayload),
}

#[derive(Clone, Debug, Serialize)]
struct ChangePhasePayload {
    phase: Phase,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum Phase {
    DownloadModLoader,
    DownloadMods,
    DownloadResources,
    AddProfile,
    LaunchModLoader,
}

#[derive(Clone, Debug, Serialize)]
struct ChangeDetailPayload {
    detail: String,
}

#[derive(Clone, Debug, Serialize)]
struct UpdateProgressPayload {
    progress: f32,
}

#[derive(Clone, Debug, Serialize)]
struct AddAlertPayload {
    level: AlertLevel,
    translation_key: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum AlertLevel {
    Info,
    Warning,
}

fn hash_matches(expected: &str, actual: &str) -> bool {
    expected.eq_ignore_ascii_case(actual)
}

fn verify_hash(expected: &str, actual: &str, final_path: &Path) -> Result<()> {
    if hash_matches(expected, actual) {
        Ok(())
    } else {
        bail!(
            "Hash mismatch for {}. Expected {expected}, got {actual}",
            final_path.display()
        );
    }
}

fn emit_event(app: &AppHandle, payload: InstallerEvent) {
    if let Err(e) = app.emit("installer://event", &payload) {
        log::warn!("Failed to emit installer event. payload: {payload:?}, error: {e:?}");
    }
}
