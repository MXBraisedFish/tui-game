use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::i18n::t;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 占位页面类型。
pub enum PlaceholderPage {
    Settings,
    About,
    Continue,
}

/// 渲染通用占位页面，用于暂未完成的功能入口。
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
