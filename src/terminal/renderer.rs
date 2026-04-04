use std::io::{Write, stdout};
use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};

use crate::core::screen::{Canvas, Cell};

static LAST_CANVAS: OnceLock<Mutex<Option<Canvas>>> = OnceLock::new();

fn canvas_cache() -> &'static Mutex<Option<Canvas>> {
    LAST_CANVAS.get_or_init(|| Mutex::new(None))
}

pub fn invalidate_canvas_cache() {
    if let Ok(mut cache) = canvas_cache().lock() {
        *cache = None;
    }
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

fn queue_cell<W: Write>(out: &mut W, x: u16, y: u16, cell: &Cell) -> Result<()> {
    if cell.continuation {
        return Ok(());
    }
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
