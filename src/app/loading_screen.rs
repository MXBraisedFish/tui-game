// 渲染启动加载画面，显示进度百分比、进度条、当前步骤信息和底部提示。在 main.rs 的启动流程中被调用

use std::io::Stdout; // 标准输出类型（终端后端泛型参数）

use anyhow::Result; // 错误处理
use ratatui::Terminal; // 终端渲染器
use ratatui::backend::CrosstermBackend; // crossterm 后端
use ratatui::layout::{Alignment, Constraint, Direction, Layout}; // 布局约束和方向
use ratatui::style::{Color, Modifier, Style}; // 颜色、修饰、样式
use ratatui::text::{Line, Span}; // 富文本行和片段
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap}; // Block、Borders、Gauge（进度条）、Paragraph、Wrap

use crate::app::content_cache; // 加载进度数据结构
use crate::app::i18n; // 国际化文本

// 绘制加载画面：标题 + 百分比 + 进度条 + 当前步骤 + 底部提示。整体居中显示在有边框的块内
pub fn render_loading_screen(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    progress: &content_cache::LoadingProgress,
) -> Result<()> {
    let title = i18n::t_or("loading.startup.title", "Loading data");
    let hint = i18n::t_or(
        "loading.startup.hint",
        "The program is preparing game and mod resources. This is not a freeze.",
    );
    terminal.draw(|frame| {
        let area = frame.area();
        let block_width = area.width.saturating_sub(2).max(1);
        let hint_lines = estimate_wrapped_lines(&hint, block_width.saturating_sub(2));
        let block_height = 5u16.saturating_add(hint_lines.max(1));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(block_height),
                Constraint::Min(0),
            ])
            .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(title.clone(), Style::default().fg(Color::White)),
                Span::styled(" ", Style::default()),
            ]))
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(layout[1]);
        frame.render_widget(block, layout[1]);

        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(hint_lines.max(1)),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{}%", progress.percent),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            body[0],
        );

        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(Color::LightGreen).bg(Color::DarkGray))
                .percent(progress.percent.min(100))
                .label(""),
            body[1],
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                progress.message.clone(),
                Style::default().fg(Color::White),
            )))
            .alignment(Alignment::Center),
            body[2],
        );

        frame.render_widget(
            Paragraph::new(hint.clone())
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::DarkGray)),
            body[3],
        );
    })?;
    Ok(())
}

// 估算文本在给定宽度下换行后的总行数。按字符数除以每行宽度向上取整
pub fn estimate_wrapped_lines(text: &str, width: u16) -> u16 {
    let width = width.max(1) as usize;
    let mut total = 0usize;
    for raw_line in text.lines() {
        let len = raw_line.chars().count().max(1);
        total += len.div_ceil(width);
    }
    total.max(1) as u16
}