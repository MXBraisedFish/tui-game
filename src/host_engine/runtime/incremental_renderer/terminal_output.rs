//! 终端差量输出（按段渲染 + 样式状态缓存）

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};

use super::color_style::{parse_color, parse_style};
use super::diff::RenderSegment;

type TerminalOutputResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 缓存的终端样式状态，避免重复发送相同 ANSI 指令。
#[derive(Default)]
struct StyleState {
    fg: Option<Color>,
    bg: Option<Color>,
    attrs: Vec<i64>,
}

/// 清空整个终端屏幕。
pub fn clear_screen() -> TerminalOutputResult<()> {
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All))?;
    stdout.flush()?;
    Ok(())
}

/// 输出所有渲染段，仅在样式变化时发送样式指令。
pub fn write_changes(segments: &[RenderSegment]) -> TerminalOutputResult<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    let mut state = StyleState::default();

    for seg in segments {
        queue!(stdout, MoveTo(seg.x, seg.y))?;
        let seg_fg = seg.fg.as_deref().and_then(parse_color);
        let seg_bg = seg.bg.as_deref().and_then(parse_color);
        if state.fg != seg_fg || state.bg != seg_bg || state.attrs != seg.styles {
            queue!(stdout, SetAttribute(Attribute::Reset), ResetColor)?;
            if let Some(fg) = seg_fg {
                queue!(stdout, SetForegroundColor(fg))?;
            }
            if let Some(bg) = seg_bg {
                queue!(stdout, SetBackgroundColor(bg))?;
            }
            for attr in seg.styles.iter().filter_map(|s| parse_style(*s)) {
                queue!(stdout, SetAttribute(attr))?;
            }
            state.fg = seg_fg;
            state.bg = seg_bg;
            state.attrs = seg.styles.clone();
        }
        queue!(stdout, Print(seg.text.as_str()))?;
    }

    queue!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;
    stdout.flush()?;
    Ok(())
}
