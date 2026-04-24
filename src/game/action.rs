use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ActionBinding {
    pub key: ActionKeys,
    pub key_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ActionKeys {
    Single(String),
    Multiple(Vec<String>),
}

impl ActionBinding {
    pub fn keys(&self) -> Vec<String> {
        match &self.key {
            ActionKeys::Single(key) => {
                if key.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![key.clone()]
                }
            }
            ActionKeys::Multiple(keys) => keys
                .iter()
                .filter(|key| !key.trim().is_empty())
                .cloned()
                .collect(),
        }
    }

    pub fn key_name(&self) -> &str {
        &self.key_name
    }

    pub fn slots(&self) -> Vec<String> {
        match &self.key {
            ActionKeys::Single(key) => vec![key.clone()],
            ActionKeys::Multiple(keys) => keys.clone(),
        }
    }
}
