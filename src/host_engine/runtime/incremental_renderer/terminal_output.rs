//! 终端差量输出

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::style::Attribute;
use crossterm::terminal::{Clear, ClearType};

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::CanvasCell;

use super::color_style::{parse_color, parse_style};
use super::diff::RenderChange;

type TerminalOutputResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 清空整个终端屏幕。
pub fn clear_screen() -> TerminalOutputResult<()> {
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All))?;
    stdout.flush()?;
    Ok(())
}

/// 输出所有变化单元格。
pub fn write_changes(changes: &[RenderChange]) -> TerminalOutputResult<()> {
    if changes.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    for change in changes {
        queue!(stdout, MoveTo(change.x, change.y))?;
        queue_cell_style(&mut stdout, &change.cell)?;
        queue!(
            stdout,
            Print(change.cell.text.as_str()),
            ResetColor,
            SetAttribute(Attribute::Reset)
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn queue_cell_style(stdout: &mut io::Stdout, cell: &CanvasCell) -> TerminalOutputResult<()> {
    if let Some(fg) = cell.fg.as_deref().and_then(parse_color) {
        queue!(stdout, SetForegroundColor(fg))?;
    }
    if let Some(bg) = cell.bg.as_deref().and_then(parse_color) {
        queue!(stdout, SetBackgroundColor(bg))?;
    }
    if let Some(style) = cell.style.and_then(parse_style) {
        queue!(stdout, SetAttribute(style))?;
    }
    Ok(())
}
