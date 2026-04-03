use std::io::{Write, stdout};
use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::core::screen::Canvas;

static LAST_CANVAS: OnceLock<Mutex<Option<Canvas>>> = OnceLock::new();

fn canvas_cache() -> &'static Mutex<Option<Canvas>> {
    LAST_CANVAS.get_or_init(|| Mutex::new(None))
}

pub fn invalidate_canvas_cache() {
    if let Ok(mut cache) = canvas_cache().lock() {
        *cache = None;
    }
}

// 清理整个终端并把光标放到0，0的位置
pub fn clear() -> Result<()> {
    invalidate_canvas_cache();
    // 获取标准输出的笔(应该叫做句柄,但是我看不懂就写成笔了)
    let mut out = stdout();

    // 将命令加入队列
    // 清空并移动光标至0,0
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;

    // 刷新输出,真正执行命令
    out.flush()?;
    Ok(())
}

// 在终端指定位置绘制内容
// 绝对坐标
pub fn draw_text(x: u16, y: u16, text: &str) -> Result<()> {
    invalidate_canvas_cache();
    let mut out = stdout();

    // 移动光标到x,y并打印文本
    queue!(out, MoveTo(x, y), Print(text))?;

    out.flush()?;
    Ok(())
}

pub fn render_canvas(canvas: &Canvas) -> Result<()> {
    let mut out = stdout();
    let mut cache = canvas_cache()
        .lock()
        .map_err(|_| anyhow::anyhow!("canvas cache poisoned"))?;
    let previous = cache.as_ref();
    let full_redraw = previous
        .map(|prev| prev.width() != canvas.width() || prev.height() != canvas.height())
        .unwrap_or(true);

    if full_redraw {
        queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let index = usize::from(y) * usize::from(canvas.width()) + usize::from(x);
                queue_cell(&mut out, x, y, &canvas.cells[index])?;
            }
        }
    } else if let Some(prev) = previous {
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let index = usize::from(y) * usize::from(canvas.width()) + usize::from(x);
                let cell = &canvas.cells[index];
                if prev.cells.get(index) != Some(cell) {
                    queue_cell(&mut out, x, y, cell)?;
                }
            }
        }
    }

    out.flush()?;
    *cache = Some(canvas.clone());
    Ok(())
}

fn queue_cell<W: Write>(out: &mut W, x: u16, y: u16, cell: &crate::core::screen::Cell) -> Result<()> {
    queue!(out, MoveTo(x, y), ResetColor)?;
    if let Some(color) = parse_color(cell.fg.as_deref()) {
        queue!(out, SetForegroundColor(color))?;
    }
    if let Some(color) = parse_color(cell.bg.as_deref()) {
        queue!(out, SetBackgroundColor(color))?;
    }
    queue!(out, Print(cell.ch), ResetColor)?;
    Ok(())
}

// 根据文本宽度自动换行
// 会保留单词完整性避免跨单词换行
// 用到了unicode_width库
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    // 如果最大宽度位0,返回空字符串
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    // 处理每一行(保留原始换行)
    for raw_line in text.lines() {
        // 如果整行宽度小于最大宽度,直接保留
        if UnicodeWidthStr::width(raw_line) <= max_width {
            lines.push(raw_line.to_string());
            continue;
        }

        // 当前正在构建的行
        let mut current = String::new();

        // 当前行的显示宽度
        let mut width = 0;

        // 遍历每个字符
        for ch in raw_line.chars() {
            // 获取字符的显示宽度(这个库汉字=2,字母=1)
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);

            // 如果加上这个字符回超出宽度,且当行不为空
            if width + w > max_width && !current.is_empty() {
                lines.push(current.clone()); // 保存当前行
                current.clear(); // 开始新行
                width = 0;
            }

            // 添加字符到当前行
            current.push(ch);
            width += w;
        }

        // 添加最后一行
        if !current.is_empty() {
            lines.push(current);
        }
    }

    // 确保至少有一行
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn parse_color(name: Option<&str>) -> Option<CColor> {
    let raw = name.unwrap_or("").trim();
    if let Some(hex) = parse_hex_color(raw) {
        return Some(hex);
    }
    if let Some(rgb) = parse_rgb_color(raw) {
        return Some(rgb);
    }
    match raw.to_ascii_lowercase().as_str() {
        "black" => Some(CColor::Black),
        "white" => Some(CColor::White),
        "red" | "light_red" => Some(CColor::Red),
        "dark_red" => Some(CColor::DarkRed),
        "yellow" | "light_yellow" => Some(CColor::Yellow),
        "dark_yellow" | "orange" => Some(CColor::DarkYellow),
        "green" | "light_green" => Some(CColor::Green),
        "blue" | "light_blue" => Some(CColor::Blue),
        "cyan" | "light_cyan" => Some(CColor::Cyan),
        "magenta" | "light_magenta" => Some(CColor::Magenta),
        "grey" | "gray" => Some(CColor::Grey),
        "dark_grey" | "dark_gray" => Some(CColor::DarkGrey),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<CColor> {
    if raw.len() != 7 || !raw.starts_with('#') {
        return None;
    }
    let r = u8::from_str_radix(&raw[1..3], 16).ok()?;
    let g = u8::from_str_radix(&raw[3..5], 16).ok()?;
    let b = u8::from_str_radix(&raw[5..7], 16).ok()?;
    Some(CColor::Rgb { r, g, b })
}

fn parse_rgb_color(raw: &str) -> Option<CColor> {
    let lower = raw.to_ascii_lowercase();
    if !lower.starts_with("rgb(") || !lower.ends_with(')') {
        return None;
    }
    let inner = &lower[4..lower.len() - 1];
    let mut parts = inner.split(',').map(|part| part.trim().parse::<u8>().ok());
    let r = parts.next()??;
    let g = parts.next()??;
    let b = parts.next()??;
    if parts.next().is_some() {
        return None;
    }
    Some(CColor::Rgb { r, g, b })
}
