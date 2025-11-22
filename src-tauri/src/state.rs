use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::config::{ModEntry, ModLoader, ResourceEntry, SourceType};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerState {
    installer_version: Version,
    #[serde(default)]
    mod_loader: Option<ModLoaderState>,
    #[serde(default)]
    mods: Vec<ModState>,
    #[serde(default)]
    resources: Vec<ResourceState>,

    #[serde(skip)]
    mod_index: HashMap<String, usize>,
    #[serde(skip)]
    resource_index: HashMap<(String, String), usize>,
}

impl InstallerState {
    fn new(version: &Version) -> Self {
        Self {
            installer_version: version.clone(),
            mod_loader: None,
            mods: Vec::new(),
            resources: Vec::new(),
            mod_index: HashMap::new(),
            resource_index: HashMap::new(),
        }
    }

    fn mod_key(source: &SourceType) -> String {
        match source {
            SourceType::Curseforge {
                project_id,
                file_id,
            } => format!("cf:{project_id}:{file_id}"),
            SourceType::Direct { url } => format!("direct:{url}"),
        }
    }

    fn resource_key(source: &SourceType, target_dir: &str) -> (String, String) {
        let source_key = match source {
            SourceType::Curseforge {
                project_id,
                file_id,
            } => format!("cf:{project_id}:{file_id}"),
            SourceType::Direct { url } => format!("direct:{url}"),
        };
        (source_key, target_dir.to_string())
    }

    pub fn get_mod_loader(&self) -> Option<&ModLoaderState> {
        self.mod_loader.as_ref()
    }

    pub fn set_mod_loader(&mut self, loader: ModLoaderState) {
        self.mod_loader = Some(loader);
    }

    pub fn get_mod(&self, mod_entry: &ModEntry) -> Option<&ModState> {
        let key = Self::mod_key(&mod_entry.source);
        self.mod_index.get(&key).map(|&index| &self.mods[index])
    }

    pub fn add_mod(&mut self, mod_state: ModState) {
        let key = Self::mod_key(&mod_state.source);
        let index = self.mods.len();
        self.mods.push(mod_state);
        self.mod_index.insert(key, index);
    }

    pub fn get_resource(&self, resource_entry: &ResourceEntry) -> Option<&ResourceState> {
        let key = Self::resource_key(&resource_entry.source, &resource_entry.target_dir);
        self.resource_index
            .get(&key)
            .map(|&index| &self.resources[index])
    }

    pub fn add_resource(&mut self, resource_state: ResourceState) {
        let key = Self::resource_key(&resource_state.source, &resource_state.target_dir);
        let index = self.resources.len();
        self.resources.push(resource_state);
        self.resource_index.insert(key, index);
    }

    pub fn load(path: &Path, version: &Version) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read installer state at {}", path.display()))?;
        let mut state: InstallerState =
            serde_json::from_str(&raw).context("Failed to deserialize installer-state.json")?;
        // Build indexes
        debug_assert!(state.mod_index.is_empty());
        state.mod_index.clear();
        for (i, mod_state) in state.mods.iter().enumerate() {
            let key = Self::mod_key(&mod_state.source);
            state.mod_index.insert(key, i);
        }
        debug_assert!(state.resource_index.is_empty());
        state.resource_index.clear();
        for (i, resource_state) in state.resources.iter().enumerate() {
            let key = Self::resource_key(&resource_state.source, &resource_state.target_dir);
            state.resource_index.insert(key, i);
        }
        // Migration
        if state.installer_version < *version {
            state.installer_version = version.clone();
        }

        Ok(state)
    }

    pub fn load_or_new(path: &Path, version: &Version) -> Result<Self> {
        if path.exists() {
            Self::load(path, version)
        } else {
            Ok(Self::new(version))
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize installer state to JSON")?;
        fs::write(path, json)
            .with_context(|| format!("Failed to write installer state to {}", path.display()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModLoaderState {
    pub file_name: String,
    pub url: String,
    pub hash: String,
}

impl ModLoaderState {
    pub fn equals(&self, config: &ModLoader) -> bool {
        self.url == config.url && self.hash == config.hash
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModState {
    pub file_name: String,
    #[serde(flatten)]
    pub source: SourceType,
    pub hash: String,
}

impl ModState {
    pub fn equals(&self, config: &ModEntry) -> bool {
        self.source == config.source && self.hash == config.hash
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceState {
    pub file_name: String,
    #[serde(flatten)]
    pub source: SourceType,
    pub hash: String,
    pub target_dir: String,
    pub decompress: bool,
}

impl ResourceState {
    pub fn equals(&self, config: &ResourceEntry) -> bool {
        self.source == config.source && self.hash == config.hash
    }
}
