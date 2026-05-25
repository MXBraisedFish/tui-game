//! 视觉主题系统。

pub mod color_scheme;
pub mod layout_config;
pub mod theme_manager;

pub use color_scheme::{ColorScheme, NamedColor, ThemeColor};
pub use layout_config::LayoutConfig;
pub use theme_manager::{TextStyle, ThemeManager};
