mod config;
mod downloader;
mod installer;
mod launcher;
mod state;

use std::{env, path::PathBuf, sync::Mutex};

use installer::{Installer, InstallerMode};
use serde::Serialize;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

pub const APP_FOLDER_NAME: &str = "mm-installer";
const LOG_FOLDER_NAME: &str = "logs";

pub struct AppState {
    cwd: PathBuf,
    log_dir: PathBuf,
    is_running: Mutex<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleStatus {
    pub is_exists_config: bool,
}

#[tauri::command]
fn initialize_title() -> TitleStatus {
    log::info!("Called initialize_title.");
    let config_path = PathBuf::from("config.yaml");
    TitleStatus {
        is_exists_config: config_path.exists(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeResult {
    pub is_accept: bool,
    pub error: Option<String>,
}

#[tauri::command]
fn select_mode(mode: InstallerMode) -> ModeResult {
    log::info!("Selected mode: {mode:?}");
    let error = match mode {
        InstallerMode::Install => {
            if Installer::can_install() {
                None
            } else {
                Some("Config file is missing.".to_string())
            }
        }
    };
    if let Some(ref err) = error {
        log::error!("Failed to change mode: {err:?}");
    }
    ModeResult {
        is_accept: error.is_none(),
        error,
    }
}

#[tauri::command]
fn open_log_folder(app: tauri::AppHandle) -> () {
    let state = app.state::<AppState>();
    if let Err(e) = app
        .opener()
        .open_path(state.log_dir.to_string_lossy().to_string(), None::<&str>)
    {
        log::error!("Failed to open log folder: {e:?}");
    }
}

#[tauri::command]
async fn run_installer(app: tauri::AppHandle, mode: InstallerMode) -> Result<(), String> {
    let state = app.state::<AppState>();
    {
        let mut is_running = state.is_running.lock().unwrap();
        if *is_running {
            log::warn!("Installer is already running, ignoring duplicate call.");
            return Err("Installer is already running".to_string());
        }
        *is_running = true;
    }
    let join_handle = tauri::async_runtime::spawn_blocking({
        let app_for_task = app.clone();
        let cwd = state.cwd.clone();
        move || Installer::new(app_for_task, cwd.join("config.yaml"), cwd)?.run(mode)
    });
    let result = match join_handle.await {
        Err(err) => {
            log::error!("Installer task join error: {err:?}");
            Err("Failed to start installing.".to_string())
        }
        Ok(Err(err)) => {
            log::error!("Installer execution failed: {err:?}");
            Err("Failed to install.".to_string())
        }
        Ok(Ok(())) => Ok(()),
    };
    {
        let mut is_running = state.is_running.lock().unwrap();
        *is_running = false;
    }

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cwd = env::current_dir().unwrap();
    let log_dir = cwd.join(APP_FOLDER_NAME).join(LOG_FOLDER_NAME);
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .level(log::LevelFilter::Info)
                .targets([tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: log_dir.clone(),
                        file_name: None,
                    },
                )])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initialize_title,
            select_mode,
            run_installer,
            open_log_folder,
        ])
        .setup(|app| {
            app.manage(AppState {
                cwd: cwd,
                log_dir: log_dir,
                is_running: false.into(),
            });
            log::info!("{}", "=".repeat(80));
            log::info!("App version: {}", app.package_info().version);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
