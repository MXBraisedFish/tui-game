//! 物理按键表示与解析。

use std::fmt;

/// 宿主内部使用的规范化物理键。
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Key {
    Char(char),
    F(u8),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Space,
    LeftCtrl,
    RightCtrl,
    LeftShift,
    RightShift,
    LeftAlt,
    RightAlt,
    LeftMeta,
    RightMeta,
    CapsLock,
    NumLock,
    ScrollLock,
    PrintScreen,
    Pause,
    Menu,
    Unknown(String),
}

impl Key {
    /// 从存储/事件字符串解析为规范键。
    pub fn from_string(value: &str) -> Option<Self> {
        let key = value.trim();
        if key.is_empty() {
            return None;
        }

        let normalized = key.to_ascii_lowercase();
        if let Some(number) = normalized.strip_prefix('f') {
            if let Ok(function_key) = number.parse::<u8>() {
                if (1..=24).contains(&function_key) {
                    return Some(Self::F(function_key));
                }
            }
        }

        Some(match normalized.as_str() {
            "up" | "arrowup" => Self::ArrowUp,
            "down" | "arrowdown" => Self::ArrowDown,
            "left" | "arrowleft" => Self::ArrowLeft,
            "right" | "arrowright" => Self::ArrowRight,
            "enter" | "return" => Self::Enter,
            "esc" | "escape" => Self::Esc,
            "tab" => Self::Tab,
            "back_tab" | "btab" | "shift_tab" => Self::BackTab,
            "backspace" | "bksp" => Self::Backspace,
            "del" | "delete" => Self::Delete,
            "ins" | "insert" => Self::Insert,
            "home" => Self::Home,
            "end" => Self::End,
            "pageup" | "pgup" => Self::PageUp,
            "pagedown" | "pgdn" => Self::PageDown,
            "space" => Self::Space,
            "left_ctrl" | "lctrl" => Self::LeftCtrl,
            "right_ctrl" | "rctrl" => Self::RightCtrl,
            "left_shift" | "lshift" => Self::LeftShift,
            "right_shift" | "rshift" => Self::RightShift,
            "left_alt" | "lalt" => Self::LeftAlt,
            "right_alt" | "ralt" => Self::RightAlt,
            "left_meta" | "lmeta" => Self::LeftMeta,
            "right_meta" | "rmeta" => Self::RightMeta,
            "capslock" | "caps" => Self::CapsLock,
            "numlock" | "num" => Self::NumLock,
            "scrolllock" | "scrl" => Self::ScrollLock,
            "printscreen" | "prtsc" => Self::PrintScreen,
            "pause" => Self::Pause,
            "menu" => Self::Menu,
            _ => parse_character_key(key)?,
        })
    }

    /// 转为当前 keybind.json 使用的规范字符串。
    pub fn to_string(&self) -> String {
        match self {
            Self::Char(value) => value.to_string(),
            Self::F(value) => format!("f{value}"),
            Self::ArrowUp => "up".to_string(),
            Self::ArrowDown => "down".to_string(),
            Self::ArrowLeft => "left".to_string(),
            Self::ArrowRight => "right".to_string(),
            Self::Enter => "enter".to_string(),
            Self::Esc => "esc".to_string(),
            Self::Tab => "tab".to_string(),
            Self::BackTab => "back_tab".to_string(),
            Self::Backspace => "backspace".to_string(),
            Self::Delete => "del".to_string(),
            Self::Insert => "ins".to_string(),
            Self::Home => "home".to_string(),
            Self::End => "end".to_string(),
            Self::PageUp => "pageup".to_string(),
            Self::PageDown => "pagedown".to_string(),
            Self::Space => "space".to_string(),
            Self::LeftCtrl => "left_ctrl".to_string(),
            Self::RightCtrl => "right_ctrl".to_string(),
            Self::LeftShift => "left_shift".to_string(),
            Self::RightShift => "right_shift".to_string(),
            Self::LeftAlt => "left_alt".to_string(),
            Self::RightAlt => "right_alt".to_string(),
            Self::LeftMeta => "left_meta".to_string(),
            Self::RightMeta => "right_meta".to_string(),
            Self::CapsLock => "capslock".to_string(),
            Self::NumLock => "numlock".to_string(),
            Self::ScrollLock => "scrolllock".to_string(),
            Self::PrintScreen => "printscreen".to_string(),
            Self::Pause => "pause".to_string(),
            Self::Menu => "menu".to_string(),
            Self::Unknown(value) => value.clone(),
        }
    }

    /// 系统保留键不能被包动作覆盖。
    pub fn is_system_key(&self) -> bool {
        matches!(self, Self::F(2) | Self::F(3) | Self::F(4))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_string())
    }
}

fn parse_character_key(key: &str) -> Option<Key> {
    let mut chars = key.chars();
    let value = chars.next()?;
    if chars.next().is_none() {
        Some(Key::Char(value))
    } else if key.starts_with("key(") && key.ends_with(')') {
        Some(Key::Unknown(key.to_string()))
    } else {
        None
    }
}
