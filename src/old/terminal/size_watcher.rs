// 监控终端尺寸是否满足当前页面的最小/最大尺寸要求。若不满足，绘制居中警告信息提示用户调整窗口

use std::io::{Write, stdout}; // 标准输出写入

use anyhow::Result; // 错误处理
use crossterm::cursor::MoveTo; // 光标定位
use crossterm::queue; // 批量排队 ANSI 指令
use crossterm::style::{Attribute, Print, SetAttribute}; // 文本加粗等样式
use crossterm::terminal::{self, Clear, ClearType}; // 清屏
use unicode_width::UnicodeWidthStr; // Unicode 字符串宽度计算（用于居中布局）

use crate::app::i18n::t; // 国际化文本模块

// 尺寸约束结构体。
#[derive(Clone, Copy, Debug, Default)]
pub struct SizeConstraints {
    pub min_width: Option<u16>, // 最小宽度要求
    pub min_height: Option<u16>, // 最小高度要求
    pub max_width: Option<u16>, // 最大宽度限制
    pub max_height: Option<u16>, // 最大高度限制
}

// 当前终端尺寸状态
#[derive(Clone, Copy, Debug)]
pub struct SizeState {
    pub width: u16, // 当前宽度
    pub height: u16, // 当前高度
    pub size_ok: bool, // 是否满足约束
}

impl SizeConstraints {
    // 构造仅有最小限制的约束
    pub fn with_min(min_width: u16, min_height: u16) -> Self {
        Self {
            min_width: Some(min_width),
            min_height: Some(min_height),
            max_width: None,
            max_height: None,
        }
    }

    // 检查给定宽高是否满足所有约束
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

// 便捷函数，使用最小约束检查当前终端尺寸
pub fn check_size(min_width: u16, min_height: u16) -> Result<SizeState> {
    check_constraints(SizeConstraints::with_min(min_width, min_height))
}

// 检查当前终端尺寸是否满足自定义约束
pub fn check_constraints(constraints: SizeConstraints) -> Result<SizeState> {
    let (width, height) = terminal::size()?;
    Ok(SizeState {
        width,
        height,
        size_ok: constraints.is_satisfied_by(width, height),
    })
}

// 使用最小约束绘制尺寸不足警告
pub fn draw_size_warning(state: &SizeState, min_width: u16, min_height: u16) -> Result<()> {
    draw_size_warning_with_constraints(
        state,
        SizeConstraints::with_min(min_width, min_height),
        false,
    )
}

// 绘制尺寸警告画面，区分"尺寸不足"和"超出限制"两种场景
pub fn draw_size_warning_with_constraints(
    state: &SizeState,
    constraints: SizeConstraints,
    back_to_game_list: bool,
) -> Result<()> {
    let mut out = stdout();

    let mut lines = vec![
        if constraints.max_width.is_some() || constraints.max_height.is_some() {
            t("warning.size_invalid_title").to_string()
        } else {
            t("warning.size_title").to_string()
        },
    ];

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
