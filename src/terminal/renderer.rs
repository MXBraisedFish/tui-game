use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// 清理整个终端并把光标放到0，0的位置
pub fn clear() -> Result<()> {
    let mut out = stdout();
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;
    out.flush()?;
    Ok(())
}

// 在终端指定位置绘制内容
// 绝对坐标
pub fn draw_text(x: u16, y: u16, text: &str) -> Result<()> {
    let mut out = stdout();
    queue!(out, MoveTo(x, y), Print(text))?;
    out.flush()?;
    Ok(())
}

// 根据文本宽度自动换行
// 会保留单词完整性避免跨单词换行
// 用到了unicode_width库
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if UnicodeWidthStr::width(raw_line) <= max_width {
            lines.push(raw_line.to_string());
            continue;
        }

        let mut current = String::new();
        let mut width = 0;

        for ch in raw_line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + w > max_width && !current.is_empty() {
                lines.push(current.clone());
                current.clear();
                width = 0;
            }
            current.push(ch);
            width += w;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
