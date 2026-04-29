// 将游戏的虚拟画布（Canvas）高效渲染到终端。核心优化是增量渲染——只重绘与实际上一帧不同的单元格，并维护样式状态以减少重复的 ANSI 控制序列

use std::io::{Write, stdout}; // 标准输出写入
use std::sync::{Mutex, OnceLock}; // 线程安全的画布缓存

use anyhow::Result; // 错误处理
use crossterm::cursor::MoveTo; // 移动终端光标
use crossterm::queue; // 批量排队 ANSI 指令
use crossterm::style::{ // 颜色、样式属性控制
    Attribute, Color as CColor, Print, ResetColor, SetAttribute, SetBackgroundColor,
    SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType}; // 清屏

use crate::core::screen::{ // 游戏的虚拟画布和样式定义
    Canvas, STYLE_BLINK, STYLE_BOLD, STYLE_DIM, STYLE_HIDDEN, STYLE_ITALIC, STYLE_REVERSE,
    STYLE_STRIKE, STYLE_UNDERLINE,
};

static LAST_CANVAS: OnceLock<Mutex<Option<Canvas>>> = OnceLock::new();

// 内部状态结构体，缓存当前终端的前景色、背景色、文本样式，避免重复设置相同的样式
#[derive(Default)]
struct StyleState {
    fg: Option<CColor>,
    bg: Option<CColor>,
    text_style: Option<i64>,
}

// 获取全局画布缓存的单例（用于增量渲染对比）
fn canvas_cache() -> &'static Mutex<Option<Canvas>> {
    LAST_CANVAS.get_or_init(|| Mutex::new(None))
}

// 清除缓存，强制下一帧全量重绘
pub fn invalidate_canvas_cache() {
    if let Ok(mut cache) = canvas_cache().lock() {
        *cache = None;
    }
}

// 主渲染函数，对比缓存决定全量/增量渲染，调用行渲染
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
        let mut style_state = StyleState::default();
        for y in 0..canvas.height() {
            queue_row_segments(&mut out, canvas, None, y, &mut style_state)?;
        }
        queue!(out, ResetColor)?;
    } else if let Some(prev) = previous {
        let mut style_state = StyleState::default();
        for y in 0..canvas.height() {
            if canvas.row(y) == prev.row(y) {
                continue;
            }
            queue_row_segments(&mut out, canvas, Some(prev), y, &mut style_state)?;
        }
        queue!(out, ResetColor)?;
    }

    out.flush()?;
    *cache = Some(canvas.clone());
    Ok(())
}

// 渲染指定行，跳过未变化的单元格，合并在同一渲染段的相同样式字符
fn queue_row_segments<W: Write>(
    out: &mut W,
    canvas: &Canvas,
    previous: Option<&Canvas>,
    y: u16,
    style_state: &mut StyleState,
) -> Result<()> {
    let Some(row) = canvas.row(y) else {
        return Ok(());
    };
    let previous_row = previous.and_then(|canvas| canvas.row(y));

    let mut x = 0usize;
    while x < row.len() {
        let current = &row[x];
        let unchanged = previous_row
            .and_then(|prev| prev.get(x))
            .map(|prev| prev == current)
            .unwrap_or(false);
        if unchanged {
            x += 1;
            continue;
        }

        if current.continuation {
            x += 1;
            continue;
        }

        let segment_start = x;
        let segment_fg = parse_color(current.fg.as_deref());
        let segment_bg = parse_color(current.bg.as_deref());
        let segment_style = current.style;
        let mut text = String::new();

        while x < row.len() {
            let cell = &row[x];
            let same_style = parse_color(cell.fg.as_deref()) == segment_fg
                && parse_color(cell.bg.as_deref()) == segment_bg
                && cell.style == segment_style;
            let changed = previous_row
                .and_then(|prev| prev.get(x))
                .map(|prev| prev != cell)
                .unwrap_or(true);

            if !changed || cell.continuation || !same_style {
                break;
            }

            text.push(cell.ch);
            x += 1;
        }

        if !text.is_empty() {
            queue!(out, MoveTo(segment_start as u16, y))?;
            apply_style(out, style_state, segment_fg, segment_bg, segment_style)?;
            queue!(out, Print(&text))?;
        } else {
            x += 1;
        }
    }

    Ok(())
}

// 仅在样式变化时发送 ANSI 控制序列，更新样式状态
fn apply_style<W: Write>(
    out: &mut W,
    style_state: &mut StyleState,
    fg: Option<CColor>,
    bg: Option<CColor>,
    text_style: Option<i64>,
) -> Result<()> {
    if style_state.fg != fg || style_state.bg != bg || style_state.text_style != text_style {
        queue!(out, SetAttribute(Attribute::Reset), ResetColor)?;
        if let Some(color) = fg {
            queue!(out, SetForegroundColor(color))?;
        }
        if let Some(color) = bg {
            queue!(out, SetBackgroundColor(color))?;
        }
        if let Some(attribute) = map_text_style(text_style) {
            queue!(out, SetAttribute(attribute))?;
        }
        style_state.fg = fg;
        style_state.bg = bg;
        style_state.text_style = text_style;
    }
    Ok(())
}

// 将游戏内样式常量映射为 crossterm 的 Attribute
fn map_text_style(style: Option<i64>) -> Option<Attribute> {
    match style {
        Some(STYLE_BOLD) => Some(Attribute::Bold),
        Some(STYLE_ITALIC) => Some(Attribute::Italic),
        Some(STYLE_UNDERLINE) => Some(Attribute::Underlined),
        Some(STYLE_STRIKE) => Some(Attribute::CrossedOut),
        Some(STYLE_BLINK) => Some(Attribute::SlowBlink),
        Some(STYLE_REVERSE) => Some(Attribute::Reverse),
        Some(STYLE_HIDDEN) => Some(Attribute::Hidden),
        Some(STYLE_DIM) => Some(Attribute::Dim),
        _ => None,
    }
}

// 将颜色字符串（命名颜色、#RGB 十六进制、rgb() 函数）解析为 crossterm 的 Color
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

// 解析 #RRGGBB 格式的十六进制颜色字符串
fn parse_hex_color(raw: &str) -> Option<CColor> {
    if raw.len() != 7 || !raw.starts_with('#') {
        return None;
    }
    let r = u8::from_str_radix(&raw[1..3], 16).ok()?;
    let g = u8::from_str_radix(&raw[3..5], 16).ok()?;
    let b = u8::from_str_radix(&raw[5..7], 16).ok()?;
    Some(CColor::Rgb { r, g, b })
}

// 解析 rgb(r,g,b) 格式的颜色字符串
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
