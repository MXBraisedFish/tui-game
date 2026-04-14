use std::io::{Write, stdout};
use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};

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

#[derive(Default)]
struct StyleState {
    fg: Option<CColor>,
    bg: Option<CColor>,
}

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
        let mut text = String::new();

        while x < row.len() {
            let cell = &row[x];
            let same_style =
                parse_color(cell.fg.as_deref()) == segment_fg && parse_color(cell.bg.as_deref()) == segment_bg;
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
            apply_style(out, style_state, segment_fg, segment_bg)?;
            queue!(out, Print(&text))?;
        } else {
            x += 1;
        }
    }

    Ok(())
}

fn apply_style<W: Write>(
    out: &mut W,
    style_state: &mut StyleState,
    fg: Option<CColor>,
    bg: Option<CColor>,
) -> Result<()> {
    if style_state.fg != fg || style_state.bg != bg {
        queue!(out, ResetColor)?;
        if let Some(color) = fg {
            queue!(out, SetForegroundColor(color))?;
        }
        if let Some(color) = bg {
            queue!(out, SetBackgroundColor(color))?;
        }
        style_state.fg = fg;
        style_state.bg = bg;
    }
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
