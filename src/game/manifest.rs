use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::action::ActionBinding;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PackageManifest {
    pub namespace: String,
    pub package_name: String,
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub thumbnail: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<serde_json::Value>,
    #[serde(default)]
    pub api_version: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GameManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detail: String,
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
}
