// 提供线程安全的宿主日志系统，支持带“日志对象”标签的日志行写入文件，用于区分不同游戏或 Mod 的日志来源

use std::cell::RefCell; // 线程局部存储的可变借用，存储当前日志对象名
use std::time::{SystemTime, UNIX_EPOCH}; // 获取 Unix 时间戳，用于日志行的时间标记

use crate::app::i18n; // 国际化文本，用于错误/警告级别的本地化标签

thread_local! {
    static LOG_OBJECT: RefCell<String> = RefCell::new("宿主".to_string());
}

// 一个 RAII 守卫结构体，用于临时切换日志对象名。Drop 时自动恢复之前的对象名
pub struct LogObjectGuard {
    previous: String, // 之前保存的日志对象名
}

impl Drop for LogObjectGuard {
    fn drop(&mut self) {
        let previous = std::mem::take(&mut self.previous);
        LOG_OBJECT.with(|object| {
            *object.borrow_mut() = previous;
        });
    }
}

// 临时设置当前线程的日志对象名，返回守卫。守卫离开作用域时自动恢复原名
pub fn scoped_log_object(object: impl Into<String>) -> LogObjectGuard {
    let object = object.into();
    let previous = LOG_OBJECT.with(|current| {
        let mut current = current.borrow_mut();
        let previous = current.clone();
        *current = if object.trim().is_empty() {
            "宿主".to_string()
        } else {
            object
        };
        previous
    });
    LogObjectGuard { previous }
}

// 追加一行日志到 tui_log.txt 文件，带有时间戳、可读时间、日志对象前缀
pub fn append_host_log_line(message: &str) {
    let Ok(log_dir) = crate::utils::path_utils::log_dir() else {
        return;
    };
    let path = log_dir.join("tui_log.txt");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let time_text = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let object = LOG_OBJECT.with(|object| object.borrow().clone());
    let line = format!("[{timestamp}][{time_text}][{object}] {message}\n");
    let mut existing = std::fs::read_to_string(&path).unwrap_or_default();
    existing.push_str(&line);
    let _ = std::fs::write(path, existing);
}

// 记录错误级别日志，内部调用 append_with_level
pub fn append_host_error(key: &str, pairs: &[(&str, &str)]) {
    append_with_level("debug.title.error", "Error", key, pairs);
}

// 记录警告级别日志
pub fn append_host_warning(key: &str, pairs: &[(&str, &str)]) {
    append_with_level("debug.title.warning", "Warning", key, pairs);
}

// 根据级别键获取本地化级别标签，再获取本地化消息模板，插值后调用 append_host_log_line
fn append_with_level(level_key: &str, level_fallback: &str, message_key: &str, pairs: &[(&str, &str)]) {
    let level = i18n::t_or(level_key, level_fallback);
    let template = i18n::t_or(message_key, message_key);
    let message = interpolate(&template, pairs);
    append_host_log_line(&format!("[{level}] {message}"));
}

// 将模板中的 {key} 占位符替换为对应的值
fn interpolate(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in pairs {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}
