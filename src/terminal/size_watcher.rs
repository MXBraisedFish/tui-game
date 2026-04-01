use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, Clear, ClearType};
use unicode_width::UnicodeWidthStr;

use crate::app::i18n::t;

#[derive(Clone, Copy, Debug, Default)]
pub struct SizeConstraints {
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

#[derive(Clone, Copy, Debug)]
pub struct SizeState {
    pub width: u16,
    pub height: u16,
    pub size_ok: bool,
}

impl SizeConstraints {
    pub fn with_min(min_width: u16, min_height: u16) -> Self {
        Self {
            min_width: Some(min_width),
            min_height: Some(min_height),
            max_width: None,
            max_height: None,
        }
    }

    pub fn is_satisfied_by(&self, width: u16, height: u16) -> bool {
        if let Some(min_width) = self.min_width
            && width < min_width
        {
            return false;
        }
        if let Some(min_height) = self.min_height
            && height < min_height
        {
            return false;
        }
        if let Some(max_width) = self.max_width
            && width > max_width
        {
            return false;
        }
        if let Some(max_height) = self.max_height
            && height > max_height
        {
            return false;
        }
        true
    }
}

pub fn check_size(min_width: u16, min_height: u16) -> Result<SizeState> {
    check_constraints(SizeConstraints::with_min(min_width, min_height))
}

pub fn check_constraints(constraints: SizeConstraints) -> Result<SizeState> {
    let (width, height) = terminal::size()?;
    Ok(SizeState {
        width,
        height,
        size_ok: constraints.is_satisfied_by(width, height),
    })
}

pub fn draw_size_warning(state: &SizeState, min_width: u16, min_height: u16) -> Result<()> {
    draw_size_warning_with_constraints(
        state,
        SizeConstraints::with_min(min_width, min_height),
        false,
    )
}

pub fn draw_size_warning_with_constraints(
    state: &SizeState,
    constraints: SizeConstraints,
    back_to_game_list: bool,
) -> Result<()> {
    let mut out = stdout();

    let mut lines = vec![if constraints.max_width.is_some() || constraints.max_height.is_some() {
        t("warning.size_invalid_title").to_string()
    } else {
        t("warning.size_title").to_string()
    }];

    if let (Some(min_width), Some(min_height)) = (constraints.min_width, constraints.min_height) {
        lines.push(format!(
            "{}: {}x{}",
            t("warning.required"),
            min_width,
            min_height
        ));
    }
    if let (Some(max_width), Some(max_height)) = (constraints.max_width, constraints.max_height) {
        lines.push(format!(
            "{}: {}x{}",
            t("warning.max_allowed"),
            max_width,
            max_height
        ));
    }

    lines.push(format!(
        "{}: {}x{}",
        t("warning.current"),
        state.width,
        state.height
    ));

    if constraints.max_width.is_some() || constraints.max_height.is_some() {
        lines.push(t("warning.adjust_hint").to_string());
    } else {
        lines.push(t("warning.enlarge_hint").to_string());
    }

    lines.push(if back_to_game_list {
        t("warning.back_to_game_list_hint").to_string()
    } else {
        t("warning.quit_hint").to_string()
    });

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
