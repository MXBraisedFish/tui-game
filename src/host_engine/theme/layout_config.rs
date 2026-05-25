//! 主题布局参数。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LayoutConfig {
    #[serde(default = "default_margin")]
    pub margin: u16,
    #[serde(default = "default_spacing")]
    pub spacing: u16,
    #[serde(default = "default_min_width")]
    pub min_width: u16,
    #[serde(default = "default_min_height")]
    pub min_height: u16,
    #[serde(default)]
    pub max_width: Option<u16>,
    #[serde(default)]
    pub max_height: Option<u16>,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            margin: default_margin(),
            spacing: default_spacing(),
            min_width: default_min_width(),
            min_height: default_min_height(),
            max_width: None,
            max_height: None,
        }
    }
}

fn default_margin() -> u16 {
    2
}

fn default_spacing() -> u16 {
    1
}

fn default_min_width() -> u16 {
    crate::host_engine::constant::ROOT_UI_MIN_WIDTH
}

fn default_min_height() -> u16 {
    crate::host_engine::constant::ROOT_UI_MIN_HEIGHT
}
