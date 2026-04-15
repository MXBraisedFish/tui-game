use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crossterm::event::KeyCode;
use std::time::Instant;

use crate::app::i18n;
use crate::app::rich_text;
use crate::mods::{self, ModPackage};

const MAX_COLS: usize = 12;
const H_GAP: u16 = 1;
const TRIANGLE: &str = "\u{25B6} ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsPage {
    Hub,
    Language,
    Mods,
}

#[derive(Clone, Debug)]
pub struct SettingsState {
    pub page: SettingsPage,
    pub hub_selected: usize,
    pub lang_selected: usize,
    pub mod_selected: usize,
    pub mod_page: usize,
    pub mod_detail_scroll: usize,
    pub mod_detail_scroll_available: bool,
    pub mod_packages: Vec<ModPackage>,
    pub mod_safe_dialog: Option<ModSafeDialog>,
}

#[derive(Clone, Debug)]
pub struct ModSafeDialog {
    pub namespace: String,
    pub mod_name: String,
    pub opened_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    None,
    BackToMenu,
}

#[derive(Clone, Copy, Debug)]
pub struct GridMetrics {
    pub cols: usize,
    pub inner_width: u16,
    pub outer_width: u16,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            page: SettingsPage::Hub,
            hub_selected: 0,
            lang_selected: default_selected_index(),
            mod_selected: 0,
            mod_page: 0,
            mod_detail_scroll: 0,
            mod_detail_scroll_available: false,
            mod_packages: load_mod_packages(),
            mod_safe_dialog: None,
        }
    }

    pub fn refresh_mods(&mut self) {
        let previous_namespace = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_packages = load_mod_packages();
        if self.mod_packages.is_empty() {
            self.mod_selected = 0;
            self.mod_page = 0;
            self.mod_detail_scroll = 0;
            self.mod_detail_scroll_available = false;
            return;
        }
        self.mod_selected = previous_namespace
            .and_then(|namespace| {
                self.mod_packages
                    .iter()
                    .position(|package| package.namespace == namespace)
            })
            .unwrap_or(0)
            .min(self.mod_packages.len().saturating_sub(1));
        let page_size = current_mod_page_size();
        self.mod_page = (self.mod_selected / page_size)
            .min(total_mod_pages(self.mod_packages.len(), page_size).saturating_sub(1));
        self.mod_detail_scroll = 0;
        self.mod_detail_scroll_available = false;
        if let Some(dialog) = &self.mod_safe_dialog
            && !self
                .mod_packages
                .iter()
                .any(|package| package.namespace == dialog.namespace)
        {
            self.mod_safe_dialog = None;
        }
    }
}

pub fn default_selected_index() -> usize {
    let languages = i18n::available_languages();
    let current = i18n::current_language_code();
    languages
        .iter()
        .position(|pack| pack.code == current)
        .unwrap_or(0)
}

pub fn handle_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    match state.page {
        SettingsPage::Hub => handle_hub_key(state, code),
        SettingsPage::Language => {
            handle_language_key(state, code);
            SettingsAction::None
        }
        SettingsPage::Mods => {
            handle_mods_key(state, code);
            SettingsAction::None
        }
    }
}

pub fn minimum_size(state: &SettingsState) -> (u16, u16) {
    match state.page {
        SettingsPage::Hub => minimum_size_hub(),
        SettingsPage::Language => minimum_size_language(),
        SettingsPage::Mods => minimum_size_mods(),
    }
}

pub fn render(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    match state.page {
        SettingsPage::Hub => render_hub(frame, state.hub_selected),
        SettingsPage::Language => render_language_selector(frame, state.lang_selected),
        SettingsPage::Mods => render_mods(frame, state),
    }
}

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
    let cols_by_width =
        (((term_width as usize) + H_GAP as usize) / (outer_width as usize + H_GAP as usize)).max(1);
    let cols = languages.len().min(MAX_COLS).min(cols_by_width).max(1);

    GridMetrics {
        cols,
        inner_width,
        outer_width,
    }
}

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

fn load_mod_packages() -> Vec<ModPackage> {
    mods::scan_mods()
        .map(|output| output.packages)
        .unwrap_or_default()
}

fn text(key: &str, fallback: &str) -> String {
    i18n::t_or(key, fallback)
}

fn handle_hub_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    let item_count = 2;
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.hub_selected = state.hub_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.hub_selected = (state.hub_selected + 1).min(item_count - 1);
        }
        KeyCode::Char('1') => state.hub_selected = 0,
        KeyCode::Char('2') => state.hub_selected = 1,
        KeyCode::Enter => match state.hub_selected {
            0 => {
                state.page = SettingsPage::Language;
                state.lang_selected = default_selected_index();
            }
            1 => {
                state.page = SettingsPage::Mods;
                state.refresh_mods();
            }
            _ => {}
        },
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

fn handle_mods_key(state: &mut SettingsState, code: KeyCode) {
    if let Some(dialog) = &state.mod_safe_dialog {
        let countdown_done = dialog.opened_at.elapsed().as_secs() >= 5;
        match code {
            KeyCode::Esc | KeyCode::Char('1') => {
                state.mod_safe_dialog = None;
            }
            KeyCode::Char('2') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, false);
                state.mod_safe_dialog = None;
                state.refresh_mods();
            }
            KeyCode::Char('3') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, true);
                state.mod_safe_dialog = None;
                state.refresh_mods();
            }
            _ => {}
        }
        return;
    }

    let page_size = current_mod_page_size();
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.mod_selected = state.mod_selected.saturating_sub(1);
            state.mod_page = state.mod_selected / page_size;
            state.mod_detail_scroll = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.mod_packages.is_empty() {
                state.mod_selected =
                    (state.mod_selected + 1).min(state.mod_packages.len().saturating_sub(1));
                state.mod_page = state.mod_selected / page_size;
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            state.mod_detail_scroll = state.mod_detail_scroll.saturating_sub(1);
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            state.mod_detail_scroll = state.mod_detail_scroll.saturating_add(1);
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if state.mod_page > 0 {
                state.mod_page -= 1;
                let start = state.mod_page * page_size;
                state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
            if state.mod_page + 1 < total_pages {
                state.mod_page += 1;
                let start = state.mod_page * page_size;
                state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                let _ = mods::set_mod_enabled(&package.namespace, !package.enabled);
                state.refresh_mods();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                let _ = mods::set_mod_debug_enabled(&package.namespace, !package.debug_enabled);
                state.refresh_mods();
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                if package.safe_mode_enabled {
                    state.mod_safe_dialog = Some(ModSafeDialog {
                        namespace: package.namespace.clone(),
                        mod_name: package.package_name.clone(),
                        opened_at: Instant::now(),
                    });
                } else {
                    let _ = mods::set_mod_safe_mode(&package.namespace, true, true);
                    state.refresh_mods();
                }
            }
        }
        KeyCode::Esc => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn minimum_size_hub() -> (u16, u16) {
    let label_lang = text("settings.hub.language", "Language");
    let label_mods = text("settings.hub.mods", "Mods");
    let enter_key = i18n::t("menu.enter_shortcut");
    let back_hint = text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu");

    let widths = [
        UnicodeWidthStr::width(format!("{}[1] {}", TRIANGLE, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}[2] {}", TRIANGLE, label_mods).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_mods).as_str()),
        UnicodeWidthStr::width(back_hint.as_str()),
    ];

    let max_width = widths.into_iter().max().unwrap_or(30) as u16;
    (max_width + 4, 12)
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

fn minimum_size_mods() -> (u16, u16) {
    (90, 26)
}

fn render_hub(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let items = [
        ("[1]", text("settings.hub.language", "Language")),
        ("[2]", text("settings.hub.mods", "Mods")),
    ];
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
    let back_hint_width = UnicodeWidthStr::width(
        text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu").as_str(),
    ) as u16;

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
        text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu"),
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
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title_widget, sections[0]);

    draw_language_grid(frame.buffer_mut(), sections[2], &languages, selected_idx);

    let hint_widget = Paragraph::new(Line::from(hint))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    frame.render_widget(hint_widget, sections[3]);
}

fn render_mods(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if state.mod_packages.is_empty() {
        state.mod_selected = 0;
        state.mod_page = 0;
        state.mod_detail_scroll = 0;
        state.mod_detail_scroll_available = false;
    } else {
        let page_size = current_mod_page_size();
        let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
        if state.mod_page >= total_pages {
            state.mod_page = total_pages.saturating_sub(1);
        }
        let page_start = state.mod_page * page_size;
        let page_end = (page_start + page_size).min(state.mod_packages.len());
        if state.mod_selected < page_start || state.mod_selected >= page_end {
            state.mod_selected = page_start.min(state.mod_packages.len().saturating_sub(1));
            state.mod_detail_scroll = 0;
        }
    }

    let area = frame.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root[0]);

    render_mod_list(frame, columns[0], state);
    render_mod_detail(frame, columns[1], state);

    let mut hints = format!(
        "{}  {}  {}  {}  {}  {}",
        text("settings.mods.hint.toggle", "[Enter] Toggle"),
        text("settings.mods.hint.debug", "[D] Debug"),
        text("settings.mods.hint.safe_mode", "[R] Safe Mode"),
        text("settings.mods.hint.move", "[鈫慮/[鈫揮 Move"),
        text("settings.mods.hint.page", "[Q]/[E] Page"),
        text("settings.hub.back_hint", "[ESC] Return to main menu")
    );
    if state.mod_detail_scroll_available {
        hints.push_str("  ");
        hints.push_str(&text("settings.mods.hint.scroll", "[W]/[S] Scroll Details"));
    }
    frame.render_widget(
        Paragraph::new(hints)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        root[1],
    );
    if let Some(dialog) = &state.mod_safe_dialog {
        render_mod_safe_dialog(frame, dialog);
    }
}

fn render_mod_list(frame: &mut ratatui::Frame<'_>, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", text("settings.mods.title", "Mods")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if state.mod_packages.is_empty() {
        frame.render_widget(
            Paragraph::new(text("settings.mods.empty", "No mods found."))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            rows[0],
        );
        frame.render_widget(
            Paragraph::new("1/1")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            rows[1],
        );
        return;
    }

    let item_height = 5u16;
    let page_size = ((rows[0].height / item_height).max(1)) as usize;
    let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
    let page = state.mod_page.min(total_pages.saturating_sub(1));
    let start = page * page_size;

    for (index, package) in state
        .mod_packages
        .iter()
        .enumerate()
        .skip(start)
        .take(page_size)
    {
        let local = (index - start) as u16;
        let item_area = Rect::new(
            rows[0].x,
            rows[0].y + local * item_height,
            rows[0].width,
            item_height.min(rows[0].height.saturating_sub(local * item_height)),
        );
        render_mod_list_item(
            frame.buffer_mut(),
            item_area,
            package,
            index == state.mod_selected,
        );
    }

    let left = if page > 0 {
        i18n::t("game_selection.pager.prev")
    } else {
        String::new()
    };
    let center = format!("{}/{}", page + 1, total_pages.max(1));
    let right = if page + 1 < total_pages {
        i18n::t("game_selection.pager.next")
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(left)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(center)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(right)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Right),
        rows[1],
    );
}

fn render_mod_list_item(buffer: &mut Buffer, area: Rect, package: &ModPackage, selected: bool) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let thumb_width = 8u16;
    let text_x = area.x + thumb_width + 2;
    let content_height = area.height.min(4);
    let status = if package.enabled {
        text("settings.mods.enabled", "Enabled")
    } else {
        text("settings.mods.disabled", "Disabled")
    };
    let base_style = if selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };
    let title_style = if selected {
        Style::default()
            .fg(Color::LightCyan)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };
    let meta_style = if selected {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Gray)
    };
    let status_style = if package.enabled {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    }
    .bg(if selected {
        Color::DarkGray
    } else {
        Color::Reset
    });

    for dy in 0..content_height {
        buffer.set_string(
            area.x,
            area.y + dy,
            " ".repeat(area.width as usize),
            base_style,
        );
    }

    for (idx, line) in package.thumbnail.lines.iter().take(4).enumerate() {
        if idx as u16 >= content_height {
            break;
        }
        render_rich_line_to_buffer(
            buffer,
            area.x,
            area.y + idx as u16,
            thumb_width as usize,
            line,
            meta_style,
        );
    }

    buffer.set_stringn(
        text_x,
        area.y,
        &package.package_name,
        area.width.saturating_sub(thumb_width + 2) as usize,
        title_style,
    );
    if content_height > 1 {
        buffer.set_stringn(
            text_x,
            area.y + 1,
            &format!(
                "{} {}",
                text("settings.mods.author", "Author:"),
                package.author
            ),
            area.width.saturating_sub(thumb_width + 2) as usize,
            meta_style,
        );
    }
    if content_height > 2 {
        buffer.set_stringn(
            text_x,
            area.y + 2,
            &format!(
                "{} {}",
                text("settings.mods.version", "Version:"),
                package.version
            ),
            area.width.saturating_sub(thumb_width + 2) as usize,
            meta_style,
        );
    }
    if content_height > 3 {
        buffer.set_stringn(
            text_x,
            area.y + 3,
            &format!(
                "{} {} / {} {}",
                text("settings.mods.state", "State:"),
                status,
                text("settings.mods.safe_mode", "Safe Mode:"),
                if package.safe_mode_enabled {
                    text("settings.mods.safe_mode_on", "On")
                } else {
                    text("settings.mods.safe_mode_off", "Off")
                }
            ),
            area.width.saturating_sub(thumb_width + 2) as usize,
            status_style,
        );
    }

    if package.debug_enabled && content_height > 0 && area.width > 0 {
        let debug_x = area.x + area.width - 1;
        for dy in 0..content_height {
            buffer.set_string(
                debug_x,
                area.y + dy,
                "█",
                Style::default().fg(Color::Red).bg(if selected {
                    Color::DarkGray
                } else {
                    Color::Reset
                }),
            );
        }
    }
}

fn render_mod_detail(frame: &mut ratatui::Frame<'_>, area: Rect, state: &mut SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", text("settings.mods.detail", "Mod Details")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(package) = state.mod_packages.get(state.mod_selected) else {
        frame.render_widget(
            Paragraph::new(text("settings.mods.empty", "No mods found."))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    };

    let build_detail_lines = |content_width: usize| {
        let mut lines = rich_lines_from_image(
            &package.banner,
            content_width,
            Style::default().fg(Color::White),
        );
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            package.package_name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "{} {}",
            text("settings.mods.author", "Author:"),
            package.author
        )));
        lines.push(Line::from(format!(
            "{} {}",
            text("settings.mods.version", "Version:"),
            package.version
        )));
        lines.push(Line::from(format!(
            "{} {}",
            text("settings.mods.namespace", "Namespace:"),
            package.namespace
        )));
        lines.push(Line::from(format!(
            "{} {}",
            text("settings.mods.safe_mode", "Safe Mode:"),
            if package.safe_mode_enabled {
                text("settings.mods.safe_mode_on", "On")
            } else {
                text("settings.mods.safe_mode_off", "Off")
            }
        )));
        lines.push(Line::from(format!(
            "{} {}",
            text("settings.mods.games", "Games:"),
            package.games.len()
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            text("settings.mods.description", "Description"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.extend(rich_text::parse_rich_text_wrapped(
            &package.description,
            content_width,
            Style::default().fg(Color::White),
        ));
        if !package.errors.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                text("settings.mods.errors", "Scan Errors"),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            for error in package.errors.iter().take(8) {
                lines.extend(rich_text::parse_rich_text_wrapped(
                    &format!(
                        "[{}] {}",
                        error.severity.to_ascii_uppercase(),
                        error.message
                    ),
                    content_width,
                    Style::default().fg(Color::White),
                ));
            }
        }
        lines
    };

    let viewport_h = inner.height as usize;
    let full_width = inner.width.max(1) as usize;
    let wide_lines = build_detail_lines(full_width);
    let needs_scroll = wide_lines.len() > viewport_h;

    let (lines, text_area) = if needs_scroll && inner.width > 2 {
        (
            build_detail_lines(inner.width.saturating_sub(2).max(1) as usize),
            Rect::new(inner.x, inner.y, inner.width - 2, inner.height),
        )
    } else {
        (wide_lines, inner)
    };

    let max_scroll = lines.len().saturating_sub(viewport_h);
    if state.mod_detail_scroll > max_scroll {
        state.mod_detail_scroll = max_scroll;
    }
    state.mod_detail_scroll_available = max_scroll > 0;

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((state.mod_detail_scroll as u16, 0)),
        text_area,
    );

    if state.mod_detail_scroll_available && inner.width > 2 {
        let scroll_x = inner.x + inner.width - 1;
        let can_up = state.mod_detail_scroll > 0;
        let can_down = state.mod_detail_scroll < max_scroll;

        frame.render_widget(
            Paragraph::new(if can_up { "↑" } else { " " }).style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, inner.y, 1, 1),
        );
        frame.render_widget(
            Paragraph::new(if can_up { "W" } else { " " }).style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, inner.y.saturating_add(1), 1, 1),
        );

        if inner.height > 4 {
            let track_start = inner.y.saturating_add(2);
            let track_len = inner.height.saturating_sub(4);
            let pos = if max_scroll == 0 {
                0
            } else {
                ((state.mod_detail_scroll * (track_len as usize - 1)) / max_scroll) as u16
            };
            frame.render_widget(
                Paragraph::new("█").style(Style::default().fg(Color::White)),
                Rect::new(scroll_x, track_start.saturating_add(pos), 1, 1),
            );
        }

        let d_y = inner.y + inner.height.saturating_sub(2);
        frame.render_widget(
            Paragraph::new(if can_down { "S" } else { " " })
                .style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, d_y, 1, 1),
        );
        frame.render_widget(
            Paragraph::new(if can_down { "↓" } else { " " })
                .style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, d_y.saturating_add(1), 1, 1),
        );
    }
}

fn rich_lines_from_image(image: &mods::ModImage, width: usize, base: Style) -> Vec<Line<'static>> {
    image
        .lines
        .iter()
        .take(13)
        .map(|line| rich_line_from_image(line, width, base))
        .collect()
}

fn rich_line_from_image(text: &str, width: usize, base: Style) -> Line<'static> {
    let line = rich_text::parse_rich_text_wrapped(text, usize::MAX / 8, base)
        .into_iter()
        .next()
        .unwrap_or_else(|| Line::from(""));
    crop_line_center_to_width(&line, width)
}

fn crop_line_center_to_width(line: &Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }

    let mut cells = Vec::<(char, Style, usize)>::new();
    for span in &line.spans {
        for ch in span.content.chars() {
            let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
            if ch_width == 0 {
                continue;
            }
            cells.push((ch, span.style, ch_width));
        }
    }

    let mut total_width: usize = cells.iter().map(|(_, _, w)| *w).sum();
    if total_width <= width {
        return line.clone();
    }

    let mut trim_left = true;
    while total_width > width && !cells.is_empty() {
        if trim_left {
            if let Some((_, _, w)) = cells.first().copied() {
                total_width = total_width.saturating_sub(w);
            }
            cells.remove(0);
        } else if let Some((_, _, w)) = cells.pop() {
            total_width = total_width.saturating_sub(w);
        }
        trim_left = !trim_left;
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style: Option<Style> = None;
    let mut current_text = String::new();
    for (ch, style, _) in cells {
        match current_style {
            Some(existing) if existing == style => current_text.push(ch),
            Some(existing) => {
                spans.push(Span::styled(current_text.clone(), existing));
                current_text.clear();
                current_text.push(ch);
                current_style = Some(style);
            }
            None => {
                current_text.push(ch);
                current_style = Some(style);
            }
        }
    }
    if let Some(style) = current_style {
        spans.push(Span::styled(current_text, style));
    }
    Line::from(spans)
}

fn render_rich_line_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    base: Style,
) {
    let lines = rich_text::parse_rich_text_wrapped(text, width.max(1), base);
    let Some(first_line) = lines.first() else {
        return;
    };
    let line = crop_line_center_to_width(first_line, width.max(1));

    let mut cursor_x = x;
    for span in &line.spans {
        let content = span.content.as_ref();
        if content.is_empty() {
            continue;
        }
        let remaining = width.saturating_sub(cursor_x.saturating_sub(x) as usize);
        if remaining == 0 {
            break;
        }
        buffer.set_stringn(cursor_x, y, content, remaining, span.style);
        cursor_x = cursor_x.saturating_add(UnicodeWidthStr::width(content) as u16);
        if cursor_x >= x.saturating_add(width as u16) {
            break;
        }
    }
}

fn current_mod_page_size() -> usize {
    let (_, term_height) = crossterm::terminal::size().unwrap_or((90, 26));
    let root_height = term_height.saturating_sub(1);
    let inner_height = root_height.saturating_sub(2);
    let content_height = inner_height.saturating_sub(1);
    (content_height / 5).max(1) as usize
}

fn total_mod_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        1
    } else {
        ((total_items + page_size.saturating_sub(1)) / page_size).max(1)
    }
}

fn render_mod_safe_dialog(frame: &mut ratatui::Frame<'_>, dialog: &ModSafeDialog) {
    use ratatui::widgets::Clear;

    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let height = area.height.saturating_sub(4).clamp(10, 14);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            text("settings.mods.safe_mode_dialog.title", "Safe Mode")
        ))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let remaining = 5u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let message = text(
        "settings.mods.safe_mode_dialog.message",
        "Are you sure you want to disable Safe Mode for mod \"{mod_name}\"?\n\nSafe Mode is designed to protect your device. After disabling it, this mod may perform high-risk operations such as file writes or system calls, which may cause data loss or system instability.\nPlease make sure you fully trust the source and author of this mod.",
    )
    .replace("{mod_name}", &dialog.mod_name);

    let mut lines = rich_text::parse_rich_text_wrapped(
        &message,
        inner.width.saturating_sub(2) as usize,
        Style::default().fg(Color::White),
    );
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(
            text("settings.mods.safe_mode_dialog.cancel", "Cancel"),
            Style::default().fg(Color::LightGreen),
        ),
    ]));
    let gated_style = Style::default().fg(if countdown_done {
        Color::Red
    } else {
        Color::DarkGray
    });
    lines.push(Line::from(vec![
        Span::styled("[2] ", index_style),
        Span::styled(
            format!(
                "{} {}{}",
                text(
                    "settings.mods.safe_mode_dialog.disable_once",
                    "Confirm Disable"
                ),
                text(
                    "settings.mods.safe_mode_dialog.disable_once_sub",
                    "(Only This Time)"
                ),
                if countdown_done {
                    String::new()
                } else {
                    format!(
                        " {}",
                        text("settings.mods.safe_mode_dialog.countdown", "{seconds}s")
                            .replace("{seconds}", &remaining.to_string())
                    )
                }
            ),
            gated_style,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("[3] ", index_style),
        Span::styled(
            format!(
                "{} {}{}",
                text(
                    "settings.mods.safe_mode_dialog.disable_forever",
                    "Confirm Disable"
                ),
                text(
                    "settings.mods.safe_mode_dialog.disable_forever_sub",
                    "(Permanently Trust This Mod)"
                ),
                if countdown_done {
                    String::new()
                } else {
                    format!(
                        " {}",
                        text("settings.mods.safe_mode_dialog.countdown", "{seconds}s")
                            .replace("{seconds}", &remaining.to_string())
                    )
                }
            ),
            gated_style,
        ),
    ]));

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left),
        inner,
    );
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
