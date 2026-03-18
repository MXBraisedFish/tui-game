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
    pub width: u16, // 当前终端宽度
    pub height: u16, // 当前终端高度
    pub size_ok: bool, // 是否满足最小尺寸要求
}

// 检查终端尺寸大小
pub fn check_size(min_width: u16, min_height: u16) -> Result<SizeState> {
    // 调用crossterm获取终端尺寸
    let (width, height) = terminal::size()?;
    Ok(SizeState {
        width,
        height,
        // 只有宽高都满足要求才是true
        size_ok: width >= min_width && height >= min_height,
    })
}

// 绘制终端警告
pub fn draw_size_warning(state: &SizeState, min_width: u16, min_height: u16) -> Result<()> {
    let mut out = stdout();

    // i18n文本
    let lines = [
        t("warning.size_title").to_string(),
        format!("{}: {}x{}", t("warning.required"), min_width, min_height),
        format!("{}: {}x{}", t("warning.current"), state.width, state.height),
        t("warning.enlarge_hint").to_string(),
        t("warning.quit_hint").to_string(),
    ];

    // 计算垂直居中位置
    // 计算警告框的顶部位置
    let top = state.height.saturating_sub(lines.len() as u16) / 2;

    // 清空屏幕
    queue!(out, Clear(ClearType::All))?;

    for (idx, line) in lines.iter().enumerate() {
        // 计算当前行的显示宽度
        let width = UnicodeWidthStr::width(line.as_str()) as u16;

        // 水平居中:x = (屏幕宽度 - 文本宽度) / 2
        let x = state.width.saturating_sub(width) / 2;

        // 垂直位置:x = 顶部偏移 + 行号
        let y = top + idx as u16;

        // 使用粗体绘制,然后重置样式
        queue!(
            out,
            MoveTo(x, y),
            SetAttribute(Attribute::Bold), // 设置为粗体
            Print(line), // 打印文本
            SetAttribute(Attribute::Reset) // 重置所有属性
        )?;
    }

    out.flush()?;
    Ok(())
}
