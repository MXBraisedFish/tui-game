// 提供简易占位页面渲染，用于 About、Settings（旧版）、Continue 等暂未完整实现的功能入口。显示对应的提示文本和返回操作提示

use ratatui::layout::{Alignment, Constraint, Direction, Layout}; // 垂直居中布局
use ratatui::style::{Color, Style}; // 文本样式
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::{Paragraph, Wrap}; // 段落渲染

use crate::app::i18n::t; // 国际化

// 占位页面类型枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceholderPage {
    Settings,
    About,
    Continue,
}

// 渲染占位页面：根据页面类型显示对应消息，About 页面显示版本号信息。消息垂直居中，底部显示返回提示
pub fn render_placeholder(
    frame: &mut ratatui::Frame<'_>,
    page: PlaceholderPage,
    runtime_version: &str,
    latest_version: Option<&str>,
) {
    let message = match page {
        PlaceholderPage::Settings => t("placeholder.settings").to_string(),
        PlaceholderPage::About => format!(
            "{}\n{} {}\n{} {}",
            t("placeholder.about"),
            t("placeholder.latest_version"),
            latest_version.unwrap_or(runtime_version),
            t("placeholder.runtime_version"),
            runtime_version
        ),
        PlaceholderPage::Continue => t("placeholder.continue").to_string(),
    };

    let back_hint = match page {
        PlaceholderPage::About => t("placeholder.about.back_hint"),
        _ => t("common.back_hint"),
    };
    let mut lines = message
        .lines()
        .map(|line| Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))))
        .collect::<Vec<_>>();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        back_hint,
        Style::default().fg(Color::DarkGray),
    )));
    let line_count = lines.len() as u16;
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(line_count),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, sections[1]);
}
