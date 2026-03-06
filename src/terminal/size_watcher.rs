use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, Clear, ClearType};
use unicode_width::UnicodeWidthStr;

use crate::app::i18n::t;

// 终端尺寸结构体
#[derive(Clone, Copy, Debug)]
pub struct SizeState {
    pub width: u16,
    pub height: u16,
    pub size_ok: bool,
}

// 检查终端尺寸大小
pub fn check_size(min_width: u16, min_height: u16) -> Result<SizeState> {
    let (width, height) = terminal::size()?;
    Ok(SizeState {
        width,
        height,
        size_ok: width >= min_width && height >= min_height,
    })
}

// 绘制终端警告
pub fn draw_size_warning(state: &SizeState, min_width: u16, min_height: u16) -> Result<()> {
    let mut out = stdout();
    let lines = [
        t("warning.size_title").to_string(),
        format!("{}: {}x{}", t("warning.required"), min_width, min_height),
        format!("{}: {}x{}", t("warning.current"), state.width, state.height),
        t("warning.enlarge_hint").to_string(),
    ];

    let top = state.height.saturating_sub(lines.len() as u16) / 2;
    queue!(out, Clear(ClearType::All))?;

    for (idx, line) in lines.iter().enumerate() {
        let width = UnicodeWidthStr::width(line.as_str()) as u16;
        let x = state.width.saturating_sub(width) / 2;
        let y = top + idx as u16;
        queue!(
            out,
            MoveTo(x, y),
            SetAttribute(Attribute::Bold),
            Print(line),
            SetAttribute(Attribute::Reset)
        )?;
    }

    out.flush()?;
    Ok(())
}
