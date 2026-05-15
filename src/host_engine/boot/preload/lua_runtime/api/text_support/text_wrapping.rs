//! 终端字符宽度换行辅助

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::{
    WrapLimit, WrapOptions,
};

/// 按终端字符宽度拆分文本行。
pub fn wrap_text_lines(text: &str, wrap_width: WrapLimit) -> Vec<String> {
    let WrapLimit::Fixed(wrap_width) = wrap_width else {
        return no_wrap_line(text);
    };
    let wrap_width = usize::from(wrap_width);
    let mut lines = Vec::new();

    for source_line in text.split('\n') {
        if source_line.is_empty() {
            lines.push(String::new());
            continue;
        }
        push_wrapped_line(source_line, wrap_width, &mut lines);
    }

    lines
}

/// 按换行配置拆分文本行，并在超过最大行高时按字符数回退替换溢出标记。
pub fn wrap_text_lines_with_options(text: &str, wrap_options: &WrapOptions) -> Vec<String> {
    let mut lines = wrap_text_lines(text, wrap_options.wrap_width);
    apply_line_limit(&mut lines, wrap_options);
    lines
}

/// 将换行视为空格并返回单行。
pub fn no_wrap_line(text: &str) -> Vec<String> {
    vec![text.replace('\n', " ")]
}

fn push_wrapped_line(source_line: &str, wrap_width: usize, lines: &mut Vec<String>) {
    let mut current_line = String::new();
    let mut current_width = 0usize;

    for character in source_line.chars() {
        let character_width = character.width().unwrap_or(1).max(1);
        if current_width > 0 && current_width + character_width > wrap_width {
            lines.push(std::mem::take(&mut current_line));
            current_width = 0;
        }

        current_line.push(character);
        current_width += character_width;

        if current_width >= wrap_width {
            lines.push(std::mem::take(&mut current_line));
            current_width = 0;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }
}

/// 返回多行文本中的最大终端字符宽度。
pub fn max_line_width(lines: &[String]) -> usize {
    lines
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or_default()
}

fn apply_line_limit(lines: &mut Vec<String>, wrap_options: &WrapOptions) {
    let WrapLimit::Fixed(wrap_height) = wrap_options.wrap_height else {
        return;
    };
    let wrap_height = usize::from(wrap_height).max(1);
    if lines.len() <= wrap_height {
        return;
    }

    lines.truncate(wrap_height);
    if let Some(text_overflow) = wrap_options.text_overflow.as_deref() {
        if !text_overflow.is_empty() {
            if let Some(last_line) = lines.last_mut() {
                *last_line = replace_tail_by_char_count(last_line, text_overflow);
            }
        }
    }
}

fn replace_tail_by_char_count(text: &str, text_overflow: &str) -> String {
    let overflow_count = text_overflow.chars().count();
    let keep_count = text.chars().count().saturating_sub(overflow_count);
    let mut output = text.chars().take(keep_count).collect::<String>();
    output.push_str(text_overflow);
    output
}
