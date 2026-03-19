use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::app::i18n;

const MAX_COLS: usize = 12;
const H_GAP: u16 = 1;
const TRIANGLE: &str = "\u{25B6} ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 设置页的子页面状态。
pub enum SettingsPage {
    Hub,
    Language,
}

#[derive(Clone, Debug)]
/// 设置页运行时状态。
pub struct SettingsState {
    pub page: SettingsPage,
    pub hub_selected: usize,
    pub lang_selected: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 设置页需要主循环执行的高层动作。
pub enum SettingsAction {
    None,
    BackToMenu,
}

#[derive(Clone, Copy, Debug)]
/// 语言网格的布局参数。
pub struct GridMetrics {
    pub cols: usize,
    pub inner_width: u16,
    pub outer_width: u16,
}

impl SettingsState {
    /// 创建默认设置页状态，初始停留在设置中转页。
    pub fn new() -> Self {
        Self {
            page: SettingsPage::Hub,
            hub_selected: 0,
            lang_selected: default_selected_index(),
        }
    }
}

/// 返回语言网格默认选中项，即当前正在使用的语言索引。
pub fn default_selected_index() -> usize {
    let languages = i18n::available_languages();
    let current = i18n::current_language_code();
    languages
        .iter()
        .position(|pack| pack.code == current)
        .unwrap_or(0)
}

/// 处理设置页键盘输入，并返回需要主循环执行的动作。
pub fn handle_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    match state.page {
        SettingsPage::Hub => handle_hub_key(state, code),
        SettingsPage::Language => {
            handle_language_key(state, code);
            SettingsAction::None
        }
    }
}

/// 返回当前设置子页面所需的最小终端尺寸。
pub fn minimum_size(state: &SettingsState) -> (u16, u16) {
    match state.page {
        SettingsPage::Hub => minimum_size_hub(),
        SettingsPage::Language => minimum_size_language(),
    }
}

/// 根据当前设置子页面状态渲染对应界面。
pub fn render(frame: &mut ratatui::Frame<'_>, state: &SettingsState) {
    match state.page {
        SettingsPage::Hub => render_hub(frame, state.hub_selected),
        SettingsPage::Language => render_language_selector(frame, state.lang_selected),
    }
}

/// 计算语言网格的列数与单元宽度，用于渲染和导航。
pub fn grid_metrics(term_width: u16, languages: &[i18n::LanguagePack]) -> GridMetrics {
    if languages.is_empty() {
        return GridMetrics {
            cols: 1,
            inner_width: 6,
            outer_width: 8,
        };
    }

    let max_name_width = languages
        .iter()
        .map(|pack| UnicodeWidthStr::width(pack.name.as_str()))
        .max()
        .unwrap_or(4);

    let inner_width = (max_name_width + 2) as u16;
    let outer_width = inner_width + 2;

    let cols_by_width = (((term_width as usize) + H_GAP as usize)
        / (outer_width as usize + H_GAP as usize))
        .max(1);
    let cols = languages.len().min(MAX_COLS).min(cols_by_width).max(1);

    GridMetrics {
        cols,
        inner_width,
        outer_width,
    }
}

/// 在语言网格内移动选中项，边缘位置不会循环跳转。
pub fn move_selection(selected: usize, key: KeyCode, metrics: GridMetrics, total: usize) -> usize {
    if total == 0 {
        return 0;
    }

    let cols = metrics.cols.max(1);
    let row = selected / cols;
    let col = selected % cols;

    match key {
        KeyCode::Left => {
            if col > 0 {
                selected - 1
            } else {
                selected
            }
        }
        KeyCode::Right => {
            if col + 1 < cols && selected + 1 < total {
                selected + 1
            } else {
                selected
            }
        }
        KeyCode::Up => {
            if row > 0 {
                selected.saturating_sub(cols)
            } else {
                selected
            }
        }
        KeyCode::Down => {
            if selected + cols < total {
                selected + cols
            } else {
                selected
            }
        }
        _ => selected,
    }
}

fn handle_hub_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    match code {
        KeyCode::Up | KeyCode::Char('k') => state.hub_selected = 0,
        KeyCode::Down | KeyCode::Char('j') => state.hub_selected = 0,
        KeyCode::Char('1') => state.hub_selected = 0,
        KeyCode::Enter => {
            state.page = SettingsPage::Language;
            state.lang_selected = default_selected_index();
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            return SettingsAction::BackToMenu;
        }
        _ => {}
    }

    SettingsAction::None
}

fn handle_language_key(state: &mut SettingsState, code: KeyCode) {
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
    let metrics = grid_metrics(term_width, &languages);

    match code {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
            state.lang_selected =
                move_selection(state.lang_selected, code, metrics, languages.len());
        }
        KeyCode::Enter => {
            if let Some(pack) = languages.get(state.lang_selected) {
                let _ = i18n::set_language(&pack.code);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn minimum_size_hub() -> (u16, u16) {
    let label_lang = i18n::t("settings.hub.language");
    let enter_key = i18n::t("menu.enter_shortcut");
    let back_hint = i18n::t("settings.hub.back_hint");

    let widths = [
        UnicodeWidthStr::width(format!("{}[1] {}", TRIANGLE, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_lang).as_str()),
        UnicodeWidthStr::width(back_hint.as_str()),
    ];

    let max_width = widths.into_iter().max().unwrap_or(30) as u16;
    (max_width + 4, 10)
}

fn minimum_size_language() -> (u16, u16) {
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
    let cols = languages.len().min(MAX_COLS).max(1) as u16;
    let rows = ((languages.len() + cols as usize - 1) / cols as usize).max(1) as u16;

    let grid_width = cols * outer_width + cols.saturating_sub(1) * H_GAP;
    let grid_height = rows * 3;
    let hint = i18n::t("confirm_language");
    let hint_width = UnicodeWidthStr::width(hint.as_str()) as u16;

    let min_w = grid_width.max(hint_width).max(30) + 2;
    let min_h = 1 + 1 + grid_height + 1 + 2;
    (min_w, min_h.max(10))
}

fn render_hub(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let items = [("[1]", i18n::t("settings.hub.language"))];
    let enter_hint = i18n::t("menu.enter_shortcut");

    let content_width = items
        .iter()
        .map(|(shortcut, text)| {
            let normal = format!("{}{} {}", TRIANGLE, shortcut, text);
            let enter = format!("{}{} {}", TRIANGLE, enter_hint, text);
            UnicodeWidthStr::width(normal.as_str()).max(UnicodeWidthStr::width(enter.as_str()))
        })
        .max()
        .unwrap_or(1) as u16;
    let back_hint_width = UnicodeWidthStr::width(i18n::t("settings.hub.back_hint").as_str()) as u16;

    let width = area
        .width
        .saturating_sub(2)
        .max(1)
        .min(content_width.max(back_hint_width).max(1));
    let height = (items.len() + 2) as u16;
    let menu_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    let left_pad = menu_area.width.saturating_sub(content_width) / 2;
    let mut lines = Vec::new();

    for (idx, (shortcut, text)) in items.iter().enumerate() {
        let is_selected = idx == selected.min(items.len().saturating_sub(1));
        let base_style = if is_selected {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let key_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(if is_selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
        let key = if is_selected {
            enter_hint.as_str()
        } else {
            shortcut
        };

        lines.push(Line::from(vec![
            Span::raw(" ".repeat(left_pad as usize)),
            Span::styled(if is_selected { TRIANGLE } else { "  " }, base_style),
            Span::styled(key.to_string(), key_style),
            Span::styled(format!(" {}", text), base_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        i18n::t("settings.hub.back_hint"),
        Style::default().fg(Color::DarkGray),
    )));

    let widget = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(widget, menu_area);
}

fn render_language_selector(frame: &mut ratatui::Frame<'_>, selected: usize) {
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
    let hint = i18n::t("confirm_language");

    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title_widget, sections[0]);

    draw_language_grid(frame.buffer_mut(), sections[2], &languages, selected_idx);

    let hint_widget = Paragraph::new(Line::from(hint))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    frame.render_widget(hint_widget, sections[3]);
}

fn draw_language_grid(
    buffer: &mut Buffer,
    area: Rect,
    languages: &[i18n::LanguagePack],
    selected: usize,
) {
    let metrics = grid_metrics(area.width, languages);
    let cols = metrics.cols;
    let rows = ((languages.len() + cols - 1) / cols).max(1);

    let grid_width = cols as u16 * metrics.outer_width + (cols.saturating_sub(1) as u16) * H_GAP;
    let grid_height = rows as u16 * 3;

    let start_x = area.x + area.width.saturating_sub(grid_width) / 2;
    let start_y = area.y + area.height.saturating_sub(grid_height) / 2;

    let current_code = i18n::current_language_code();

    for (idx, pack) in languages.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let x = start_x + col as u16 * (metrics.outer_width + H_GAP);
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

fn center_text(text: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(text);
    if current >= width {
        return text.to_string();
    }

    let remaining = width - current;
    let left = remaining / 2;
    let right = remaining - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}
