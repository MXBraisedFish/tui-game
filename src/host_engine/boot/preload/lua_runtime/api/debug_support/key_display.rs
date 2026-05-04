//! 按键语义值显示映射

use serde_json::Value as JsonValue;

/// 将按键语义值转换为 UI 展示文本。
pub fn display_key_value(key_value: &JsonValue, case_sensitive: bool) -> JsonValue {
    match key_value {
        JsonValue::String(key) => JsonValue::String(display_semantic_key(key, case_sensitive)),
        JsonValue::Array(keys) => JsonValue::Array(
            keys.iter()
                .map(|key| display_key_value(key, case_sensitive))
                .collect(),
        ),
        _ => JsonValue::Null,
    }
}

fn display_semantic_key(key: &str, case_sensitive: bool) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }

    if key.len() == 1 {
        let character = key.chars().next().unwrap_or_default();
        if character.is_ascii_lowercase() && !case_sensitive {
            return character.to_ascii_uppercase().to_string();
        }
        return character.to_string();
    }

    match key {
        "f1" => "F1",
        "f2" => "F2",
        "f3" => "F3",
        "f4" => "F4",
        "f5" => "F5",
        "f6" => "F6",
        "f7" => "F7",
        "f8" => "F8",
        "f9" => "F9",
        "f10" => "F10",
        "f11" => "F11",
        "f12" => "F12",
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PgUp",
        "pagedown" => "PgDn",
        "enter" => "Enter",
        "backspace" => "Bksp",
        "del" => "Del",
        "ins" => "Ins",
        "tab" => "Tab",
        "back_tab" => "BTab",
        "space" => "Space",
        "left_ctrl" => "LCtrl",
        "right_ctrl" => "RCtrl",
        "left_shift" => "LShift",
        "right_shift" => "RShift",
        "shift" => "Shift",
        "left_alt" => "LAlt",
        "right_alt" => "RAlt",
        "left_meta" => "LMeta",
        "right_meta" => "RMeta",
        "capslock" => "Caps",
        "numlock" => "Num",
        "scrolllock" => "Scrl",
        "esc" => "Esc",
        "printscreen" => "Prtsc",
        "pause" => "Pause",
        "menu" => "Menu",
        other => other,
    }
    .to_string()
}
