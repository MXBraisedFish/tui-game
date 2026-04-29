//! Ctrl+C 处理器

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 判断当前按键是否为 Ctrl+C
pub fn is_ctrl_c(key_event: &KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c') | KeyCode::Char('C'))
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
}
