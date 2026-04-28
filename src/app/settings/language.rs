// 语言选择页面，以网格布局展示所有可用语言包。用户可通过方向键导航，Enter 确认切换语言，切换后自动刷新内容缓存和 Mod 列表

use ratatui::buffer::Buffer; // 直接操作终端缓冲区（绘制语言网格）
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect}; // 布局管理
use ratatui::style::{Color, Modifier, Style}; // 样式控制
use ratatui::widgets::Paragraph; // 段落渲染
use unicode_width::UnicodeWidthStr; // 文本宽度

use crate::app::i18n; // 国际化（获取语言包列表、切换语言）
use crate::app::settings::common as settings_common; // 通用工具（网格计算、提示换行）
use crate::app::settings::types::*; // 类型定义（SettingsState、SettingsPage 等）

// 计算语言页面的最小尺寸：网格宽度+提示宽度，至少 30×10
pub fn minimum_size_language() -> (u16, u16) {
    let languages = i18n::available_languages();
    if languages.is_empty() {
        return (40, 8);
    }

    let max_name_width = languages
        .iter()
        .map(|pack| UnicodeWidthStr::width(pack.name.as_str()))
        .max()
        .unwrap_or(4);

    let inner_width = (max_name_width + 2) as u16;
    let outer_width = inner_width + 2;
    let cols = languages.len().min(settings_common::MAX_COLS).max(1) as u16;
    let rows = ((languages.len() + cols as usize - 1) / cols as usize).max(1) as u16;

    let grid_width = cols * outer_width + cols.saturating_sub(1) * settings_common::H_GAP;
    let grid_height = rows * 3;
    let hint = build_language_hint_segments_for_code("");
    let hint_width = UnicodeWidthStr::width(hint.join("  ").as_str()) as u16;

    let min_w = grid_width.max(hint_width).max(30) + 2;
    let min_h = 1 + 1 + grid_height + 1 + 2;
    (min_w, min_h.max(10))
}

// 渲染语言选择器：顶部显示当前语言的本土化"语言"标题，中间渲染语言网格，底部显示操作提示
pub fn render_language_selector(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let languages = i18n::available_languages();
    if languages.is_empty() {
        let empty = Paragraph::new(i18n::t("settings.no_valid_languages"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(empty, sections[2]);
        return;
    }

    let selected_idx = selected.min(languages.len() - 1);
    let selected_pack = &languages[selected_idx];
    let title = i18n::t_for_code(&selected_pack.code, "language");
    let hint_lines = settings_common::wrap_language_hint_lines(
        &build_language_hint_segments_for_code(&selected_pack.code),
        sections[3].width.max(1) as usize,
    );

    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title_widget, sections[0]);

    draw_language_grid(frame.buffer_mut(), sections[2], &languages, selected_idx);

    let hint_widget = Paragraph::new(hint_lines)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    frame.render_widget(hint_widget, sections[3]);
}

// 核心绘制函数：计算网格布局，对每个语言格绘制文字，选中项绘制双线边框，当前使用语言以黄绿色粗体高亮
pub fn draw_language_grid(
    buffer: &mut Buffer,
    area: Rect,
    languages: &[i18n::LanguagePack],
    selected: usize,
) {
    let metrics = settings_common::grid_metrics(area.width, languages);
    let cols = metrics.cols;
    let rows = ((languages.len() + cols - 1) / cols).max(1);

    let grid_width = cols as u16 * metrics.outer_width + (cols.saturating_sub(1) as u16) * settings_common::H_GAP;
    let grid_height = rows as u16 * 3;

    let start_x = area.x + area.width.saturating_sub(grid_width) / 2;
    let start_y = area.y + area.height.saturating_sub(grid_height) / 2;

    let current_code = i18n::current_language_code();

    for (idx, pack) in languages.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let x = start_x + col as u16 * (metrics.outer_width + settings_common::H_GAP);
        let y = start_y + row as u16 * 3;

        let is_selected = idx == selected;
        let is_current = pack.code == current_code;

        let text_style = if is_current {
            Style::default()
                .fg(Color::Rgb(173, 255, 47))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = Style::default().fg(Color::White);
        let label = center_text(pack.name.as_str(), metrics.inner_width as usize);

        if is_selected {
            let top = format!(
                "\u{2554}{}\u{2557}",
                "\u{2550}".repeat(metrics.inner_width as usize)
            );
            let bottom = format!(
                "\u{255A}{}\u{255D}",
                "\u{2550}".repeat(metrics.inner_width as usize)
            );
            buffer.set_string(x, y, top, border_style);
            buffer.set_string(x, y + 1, "\u{2551}", border_style);
            buffer.set_string(x + 1, y + 1, label.clone(), text_style);
            buffer.set_string(x + 1 + metrics.inner_width, y + 1, "\u{2551}", border_style);
            buffer.set_string(x, y + 2, bottom, border_style);
        } else {
            let blank = " ".repeat(metrics.outer_width as usize);
            let mid = format!(" {} ", label);
            buffer.set_string(x, y, blank.clone(), Style::default());
            buffer.set_string(x, y + 1, mid, text_style);
            buffer.set_string(x, y + 2, blank, Style::default());
        }
    }
}

// 在指定宽度内水平居中文本
pub fn center_text(text: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(text);
    if current >= width {
        return text.to_string();
    }
    let remaining = width - current;
    let left = remaining / 2;
    let right = remaining - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

// 根据语言代码构建操作提示（空代码用默认语言，非空用指定语言翻译提示）
pub fn build_language_hint_segments_for_code(code: &str) -> Vec<String> {
    let translate = |key: &str, fallback: &str| {
        if code.is_empty() {
            settings_common::text(key, fallback)
        } else {
            i18n::t_for_code(code, key)
        }
    };

    vec![
        translate("language.hint.segment.confirm", "[Enter] Confirm language"),
        translate("language.hint.segment.back", "[ESC]/[Q] Return to main menu"),
    ]
}

// 处理语言页面的按键事件：方向键移动选择，Enter 切换语言并刷新缓存，Esc/Q 返回 Hub
pub fn handle_language_key(state: &mut SettingsState, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;
    let languages = i18n::available_languages();
    if languages.is_empty() {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')) {
            state.page = SettingsPage::Hub;
        }
        return;
    }

    if state.lang_selected >= languages.len() {
        state.lang_selected = languages.len() - 1;
    }

    let (term_width, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let metrics = settings_common::grid_metrics(term_width, &languages);

    match code {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
            state.lang_selected =
                settings_common::move_selection(state.lang_selected, code, metrics, languages.len());
        }
        KeyCode::Enter => {
            if let Some(pack) = languages.get(state.lang_selected) {
                let _ = i18n::set_language(&pack.code);
                crate::app::content_cache::reload();
                state.refresh_mods();
            }
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}