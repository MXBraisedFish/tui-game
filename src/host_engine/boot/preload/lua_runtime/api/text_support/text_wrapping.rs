//! 终端字符宽度换行辅助

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// 按终端字符宽度拆分文本行。
pub fn wrap_text_lines(text: &str, wrap_width: Option<u16>) -> Vec<String> {
    let Some(wrap_width) = wrap_width.filter(|width| *width > 0) else {
        return text.split('\n').map(ToString::to_string).collect();
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
