use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherProfiles {
    pub profiles: BTreeMap<String, LauncherProfile>,
    pub settings: BTreeMap<String, serde_json::Value>,
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_dir: Option<PathBuf>,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub last_version_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<ProfileResolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_jre_version_check: Option<bool>,
    #[serde(rename = "type")]
    pub profile_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResolution {
    pub height: u32,
    pub width: u32,
}
