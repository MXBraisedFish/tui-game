//! 数据驱动主题颜色定义。

use std::collections::HashMap;
use std::fmt;

use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 终端 16 色命名表。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    DarkGray,
}

impl NamedColor {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "black" => Some(Self::Black),
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "blue" => Some(Self::Blue),
            "magenta" => Some(Self::Magenta),
            "cyan" => Some(Self::Cyan),
            "white" => Some(Self::White),
            "gray" | "grey" => Some(Self::Gray),
            "dark_red" => Some(Self::DarkRed),
            "dark_green" => Some(Self::DarkGreen),
            "dark_yellow" => Some(Self::DarkYellow),
            "dark_blue" => Some(Self::DarkBlue),
            "dark_magenta" => Some(Self::DarkMagenta),
            "dark_cyan" => Some(Self::DarkCyan),
            "dark_gray" | "dark_grey" => Some(Self::DarkGray),
            _ => None,
        }
    }

    pub fn as_canvas_value(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Blue => "blue",
            Self::Magenta => "magenta",
            Self::Cyan => "cyan",
            Self::White => "white",
            Self::Gray => "gray",
            Self::DarkRed => "dark_red",
            Self::DarkGreen => "dark_green",
            Self::DarkYellow => "dark_yellow",
            Self::DarkBlue => "dark_blue",
            Self::DarkMagenta => "dark_magenta",
            Self::DarkCyan => "dark_cyan",
            Self::DarkGray => "dark_gray",
        }
    }
}

/// 主题颜色值，支持命名色、RGB 与 Hex。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeColor {
    Named(NamedColor),
    Rgb(u8, u8, u8),
    Hex(String),
}

impl ThemeColor {
    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if let Some(named) = NamedColor::parse(trimmed) {
            return Some(Self::Named(named));
        }
        if is_hex_color(trimmed) {
            return Some(Self::Hex(trimmed.to_ascii_lowercase()));
        }
        parse_rgb_function(trimmed).map(|(r, g, b)| Self::Rgb(r, g, b))
    }

    pub fn as_canvas_value(&self) -> String {
        match self {
            Self::Named(named) => named.as_canvas_value().to_string(),
            Self::Rgb(red, green, blue) => format!("rgb({red},{green},{blue})"),
            Self::Hex(hex) => hex.clone(),
        }
    }
}

impl Serialize for ThemeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_canvas_value().as_str())
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = ThemeColor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a color string or {\"rgb\": [r,g,b]}")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                ThemeColor::parse(value)
                    .ok_or_else(|| E::custom(format!("invalid theme color: {value}")))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<String>()? {
                    if key == "rgb" {
                        let rgb = map.next_value::<[u8; 3]>()?;
                        return Ok(ThemeColor::Rgb(rgb[0], rgb[1], rgb[2]));
                    }
                    let _ = map.next_value::<serde_json::Value>()?;
                }
                Err(M::Error::custom("missing rgb field"))
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

/// 主题颜色表。键名是语义 role，例如 `text.primary`。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorScheme {
    #[serde(default = "default_colors")]
    pub colors: HashMap<String, ThemeColor>,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            colors: default_colors(),
        }
    }
}

impl ColorScheme {
    pub fn color(&self, role: &str) -> Option<String> {
        self.colors.get(role).map(ThemeColor::as_canvas_value)
    }

    pub fn color_or(&self, role: &str, fallback: &str) -> String {
        self.color(role).unwrap_or_else(|| fallback.to_string())
    }
}

fn default_colors() -> HashMap<String, ThemeColor> {
    [
        ("text.primary", ThemeColor::Named(NamedColor::White)),
        ("text.secondary", ThemeColor::Named(NamedColor::DarkGray)),
        ("text.muted", ThemeColor::Named(NamedColor::DarkGray)),
        ("text.on_selected", ThemeColor::Named(NamedColor::Black)),
        ("text.warning", ThemeColor::Named(NamedColor::Yellow)),
        ("text.danger", ThemeColor::Named(NamedColor::Red)),
        ("text.success", ThemeColor::Named(NamedColor::Green)),
        ("logo.primary", ThemeColor::Hex("#ffa500".to_string())),
        ("accent.primary", ThemeColor::Named(NamedColor::Cyan)),
        ("accent.selected", ThemeColor::Hex("#78a8da".to_string())),
        (
            "background.selected",
            ThemeColor::Hex("#78a8da".to_string()),
        ),
        ("border.primary", ThemeColor::Named(NamedColor::White)),
        ("panel.background", ThemeColor::Named(NamedColor::Black)),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.chars().skip(1).all(|ch| ch.is_ascii_hexdigit())
}

fn parse_rgb_function(value: &str) -> Option<(u8, u8, u8)> {
    let inner = value.strip_prefix("rgb(")?.strip_suffix(')')?;
    let parts = inner
        .split(',')
        .map(str::trim)
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (parts.len() == 3).then_some((parts[0], parts[1], parts[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_hex_and_rgb_colors() {
        assert_eq!(
            ThemeColor::parse("cyan"),
            Some(ThemeColor::Named(NamedColor::Cyan))
        );
        assert_eq!(
            ThemeColor::parse("#78A8DA"),
            Some(ThemeColor::Hex("#78a8da".to_string()))
        );
        assert_eq!(
            ThemeColor::parse("rgb(255, 128, 64)"),
            Some(ThemeColor::Rgb(255, 128, 64))
        );
    }

    #[test]
    fn deserializes_rgb_object() {
        let color: ThemeColor = serde_json::from_str(r#"{"rgb":[1,2,3]}"#).unwrap();
        assert_eq!(color, ThemeColor::Rgb(1, 2, 3));
    }

    #[test]
    fn rejects_invalid_colors() {
        assert_eq!(ThemeColor::parse("#abc"), None);
        assert_eq!(ThemeColor::parse("rgb(300,1,2)"), None);
        assert_eq!(ThemeColor::parse("unknown"), None);
    }
}
