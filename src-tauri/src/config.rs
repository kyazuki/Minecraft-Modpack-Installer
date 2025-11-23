use std::{
    fs,
    path::{Component, Path},
};

use anyhow::{bail, Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};

pub const LATEST_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModPackConfig {
    pub schema_version: u32,
    pub pack_version: Version,
    pub profile: Profile,
    pub mod_loader: ModLoader,
    #[serde(default)]
    pub mods: Vec<ModEntry>,
    #[serde(default)]
    pub resources: Vec<ResourceEntry>,
}

impl ModPackConfig {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {}", path.display()))?;
        let mut config: ModPackConfig =
            serde_yaml::from_str(&raw).context("Failed to parse config.yaml")?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&mut self) -> Result<()> {
        if self.schema_version > LATEST_SCHEMA_VERSION {
            bail!(
                "Unsupported config schema version '{}' (expected version {} or lower)",
                self.schema_version,
                LATEST_SCHEMA_VERSION
            );
        }
        self.profile.validate()?;
        self.mod_loader.validate()?;
        for entry in self.mods.iter_mut() {
            entry.validate()?;
        }
        for entry in self.resources.iter_mut() {
            entry.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub icon: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jvm_args: Option<String>,
}

impl Profile {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            bail!("profile.name must not be empty");
        }
        if self.version.trim().is_empty() {
            bail!("profile.version must not be empty");
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModLoader {
    pub name: String,
    pub url: String,
    pub hash: String,
    #[serde(default)]
    pub auto_open: bool,
}

impl ModLoader {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModEntry {
    pub name: String,
    #[serde(flatten)]
    pub source: SourceType,
    pub hash: String,
    pub side: SideType,
}

impl ModEntry {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceEntry {
    pub name: String,
    #[serde(flatten)]
    pub source: SourceType,
    pub hash: String,
    pub target_dir: String,
    #[serde(default)]
    pub decompress: bool,
    pub side: SideType,
}

impl ResourceEntry {
    fn validate(&self) -> Result<()> {
        validate_relative_dir(&self.target_dir, "resources.targetDir")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "type",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum SourceType {
    Curseforge { project_id: u32, file_id: u32 },
    Direct { url: String },
}

impl SourceType {
    pub fn get_download_url(&self) -> String {
        match self {
            SourceType::Curseforge {
                project_id,
                file_id,
            } => format!(
                "https://www.curseforge.com/api/v1/mods/{project_id}/files/{file_id}/download"
            ),
            SourceType::Direct { url } => url.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SideType {
    Both,
    Client,
    Server,
}

fn validate_relative_dir(dir: &str, field: &str) -> Result<()> {
    let path = Path::new(dir);
    if path.is_absolute() {
        bail!("{field} must be a relative path");
    }
    for component in path.components() {
        match component {
            Component::ParentDir => {
                bail!("{field} must not contain '..' segments");
            }
            Component::Normal(segment) => {
                if segment.to_string_lossy().contains(['\\', ':']) {
                    bail!("{field} contains invalid characters");
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("{field} must be a relative path");
            }
            Component::CurDir => {}
        }
    }
    Ok(())
}
