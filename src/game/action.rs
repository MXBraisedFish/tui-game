use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ActionBinding {
    Single(String),
    Multiple(Vec<String>),
}

impl ActionBinding {
    pub fn keys(&self) -> Vec<String> {
        match self {
            Self::Single(key) => vec![key.clone()],
            Self::Multiple(keys) => keys.clone(),
        }
    }
}
