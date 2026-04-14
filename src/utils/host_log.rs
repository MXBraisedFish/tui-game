use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::i18n;

pub fn append_host_log_line(message: &str) {
    let Ok(log_dir) = crate::utils::path_utils::log_dir() else {
        return;
    };
    let path = log_dir.join("tui_log.txt");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let line = format!("[{timestamp}] {message}\n");
    let mut existing = std::fs::read_to_string(&path).unwrap_or_default();
    existing.push_str(&line);
    let _ = std::fs::write(path, existing);
}

pub fn append_host_error(key: &str, pairs: &[(&str, &str)]) {
    append_with_level("debug.title.error", "Error", key, pairs);
}

pub fn append_host_warning(key: &str, pairs: &[(&str, &str)]) {
    append_with_level("debug.title.warning", "Warning", key, pairs);
}

fn append_with_level(level_key: &str, level_fallback: &str, message_key: &str, pairs: &[(&str, &str)]) {
    let level = i18n::t_or(level_key, level_fallback);
    let template = i18n::t_or(message_key, message_key);
    let message = interpolate(&template, pairs);
    append_host_log_line(&format!("[{level}] {message}"));
}

fn interpolate(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in pairs {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}
