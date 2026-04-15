use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::action::ActionBinding;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct RuntimeManifest {
    #[serde(default)]
    pub target_fps: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PackageManifest {
    #[serde(default)]
    pub namespace: String,
    #[serde(default, alias = "package")]
    pub package_name: String,
    #[serde(default)]
    pub mod_name: Option<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub introduction: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    #[serde(alias = "thumbnail")]
    pub icon: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<serde_json::Value>,
    #[serde(default)]
    pub api_version: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GameManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub introduction: Option<String>,
    #[serde(default)]
    pub icon: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<serde_json::Value>,
    pub entry: String,
    #[serde(default)]
    pub save: bool,
    #[serde(default)]
    pub best_none: Option<String>,
    #[serde(default)]
    pub min_width: Option<u16>,
    #[serde(default)]
    pub min_height: Option<u16>,
    #[serde(default)]
    pub max_width: Option<u16>,
    #[serde(default)]
    pub max_height: Option<u16>,
    #[serde(default)]
    pub actions: BTreeMap<String, ActionBinding>,
    #[serde(default)]
    pub runtime: RuntimeManifest,
    #[serde(default)]
    pub api: Option<serde_json::Value>,
    #[serde(default)]
    pub write: bool,
}
