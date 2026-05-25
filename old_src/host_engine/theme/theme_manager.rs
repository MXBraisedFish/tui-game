//! 主题加载与查询。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::{
    STYLE_BLINK, STYLE_BOLD, STYLE_DIM, STYLE_HIDDEN, STYLE_ITALIC, STYLE_NORMAL, STYLE_REVERSE,
    STYLE_STRIKE, STYLE_UNDERLINE,
};
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::storage::profile_store::ProfileStore;
use crate::host_engine::theme::color_scheme::ColorScheme;
use crate::host_engine::theme::layout_config::LayoutConfig;

type ThemeResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TextStyle {
    #[serde(default)]
    pub fg: Option<String>,
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_text_styles")]
    pub styles: Vec<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ThemeFile {
    #[serde(default = "default_theme_name")]
    active_theme: String,
    #[serde(default)]
    colors: ColorScheme,
    #[serde(default)]
    layout: LayoutConfig,
    #[serde(default)]
    styles: HashMap<String, TextStyle>,
}

#[derive(Clone, Debug)]
pub struct ThemeManager {
    pub active_theme: String,
    pub colors: ColorScheme,
    pub layout: LayoutConfig,
    pub styles: HashMap<String, TextStyle>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self {
            active_theme: default_theme_name(),
            colors: ColorScheme::default(),
            layout: LayoutConfig::default(),
            styles: default_styles(),
        }
    }
}

impl ThemeManager {
    pub fn load(profiles: &ProfileStore, _packages: &PackageManager) -> ThemeResult<Self> {
        let requested_theme = normalized_theme_name(profiles.display.theme.as_str());
        Self::load_theme(requested_theme.as_str())
    }

    pub fn set_theme(&mut self, name: &str) -> ThemeResult<()> {
        *self = Self::load_theme(name)?;
        Ok(())
    }

    pub fn current_colors(&self) -> &ColorScheme {
        &self.colors
    }

    pub fn color(&self, role: &str) -> Option<String> {
        self.colors.color(role)
    }

    pub fn color_or(&self, role: &str, fallback: &str) -> String {
        self.colors.color_or(role, fallback)
    }

    fn load_theme(name: &str) -> ThemeResult<Self> {
        let normalized_name = normalized_theme_name(name);
        let Some(theme_path) = theme_path(normalized_name.as_str()) else {
            return Ok(Self {
                active_theme: normalized_name,
                ..Self::default()
            });
        };

        let raw_json = fs::read_to_string(theme_path)?;
        let theme_file =
            serde_json::from_str::<ThemeFile>(raw_json.trim_start_matches('\u{feff}'))?;
        Ok(Self {
            active_theme: normalized_name,
            colors: theme_file.colors,
            layout: theme_file.layout,
            styles: merge_default_styles(theme_file.styles),
        })
    }
}

fn theme_path(name: &str) -> Option<PathBuf> {
    let file_name = if name == "system" { "default" } else { name };
    let path = data_dirs::root_dir()
        .join("assets/theme")
        .join(format!("{file_name}.json"));
    path.is_file().then_some(path)
}

fn normalized_theme_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        default_theme_name()
    } else {
        trimmed.to_string()
    }
}

fn default_theme_name() -> String {
    "system".to_string()
}

fn default_styles() -> HashMap<String, TextStyle> {
    [
        (
            "title",
            TextStyle {
                fg: Some("text.primary".to_string()),
                bg: None,
                styles: vec![STYLE_BOLD],
            },
        ),
        (
            "footer",
            TextStyle {
                fg: Some("text.muted".to_string()),
                bg: None,
                styles: vec![STYLE_BOLD],
            },
        ),
        (
            "selected",
            TextStyle {
                fg: Some("accent.primary".to_string()),
                bg: None,
                styles: vec![STYLE_BOLD],
            },
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

fn merge_default_styles(styles: HashMap<String, TextStyle>) -> HashMap<String, TextStyle> {
    let mut merged = default_styles();
    merged.extend(styles);
    merged
}

fn deserialize_text_styles<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<serde_json::Value>::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .filter_map(|value| match value {
            serde_json::Value::Number(number) => number.as_i64().and_then(normalize_style_value),
            serde_json::Value::String(name) => parse_style_name(name.as_str()),
            _ => None,
        })
        .collect())
}

fn normalize_style_value(value: i64) -> Option<i64> {
    (STYLE_NORMAL..=STYLE_DIM).contains(&value).then_some(value)
}

fn parse_style_name(name: &str) -> Option<i64> {
    match name.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(STYLE_NORMAL),
        "bold" => Some(STYLE_BOLD),
        "italic" => Some(STYLE_ITALIC),
        "underline" => Some(STYLE_UNDERLINE),
        "strike" => Some(STYLE_STRIKE),
        "blink" => Some(STYLE_BLINK),
        "reverse" => Some(STYLE_REVERSE),
        "hidden" => Some(STYLE_HIDDEN),
        "dim" => Some(STYLE_DIM),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_exposes_core_roles() {
        let manager = ThemeManager::default();
        assert_eq!(manager.color_or("text.primary", "missing"), "white");
        assert_eq!(manager.color_or("accent.primary", "missing"), "cyan");
        assert_eq!(
            manager.color_or("background.selected", "missing"),
            "#78a8da"
        );
    }

    #[test]
    fn default_styles_are_available() {
        let manager = ThemeManager::default();
        assert!(manager.styles.contains_key("title"));
        assert!(manager.styles.contains_key("footer"));
        assert!(manager.styles.contains_key("selected"));
    }

    #[test]
    fn text_style_deserializes_string_and_numeric_styles() {
        let style: TextStyle =
            serde_json::from_str(r#"{"fg":"text.primary","styles":["bold",3,99,"bad"]}"#).unwrap();
        assert_eq!(style.styles, vec![STYLE_BOLD, STYLE_UNDERLINE]);
    }

    #[test]
    fn bundled_default_theme_file_is_parseable() {
        let raw_json = include_str!("../../../assets/theme/default.json");
        let theme_file = serde_json::from_str::<ThemeFile>(raw_json).unwrap();
        assert!(theme_file.colors.color("text.primary").is_some());
        assert_eq!(theme_file.styles["title"].styles, vec![STYLE_BOLD]);
    }
}
