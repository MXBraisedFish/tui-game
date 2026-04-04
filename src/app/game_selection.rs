use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{symbols, widgets::Wrap};
use unicode_width::UnicodeWidthStr;

use crate::app::i18n;
use crate::app::rich_text;
use crate::core::stats as runtime_stats;
use crate::game::registry::GameDescriptor;
use crate::game::resources;

/// 游戏选择页的完整状态。
pub struct GameSelection {
    games: Vec<GameDescriptor>,
    list_state: ListState,
    page_state: PageState,
    launch_placeholder: bool,
    detail_scroll: usize,
    detail_scroll_available: bool,
}

#[derive(Clone, Copy)]
/// 列表分页状态。
struct PageState {
    current_page: usize,
    page_size: usize,
    total_pages: usize,
}

/// 游戏选择页向主循环上报的高层动作。
pub enum GameSelectionAction {
    BackToMenu,
    LaunchGame(GameDescriptor),
}

impl GameSelection {
    /// 根据扫描到的游戏列表和本地成绩数据创建游戏选择页状态。
    pub fn new(games: Vec<GameDescriptor>) -> Self {
        let initial_page_size = games.len().max(1);

        let mut list_state = ListState::default();
        if !games.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            games,
            list_state,
            page_state: PageState {
                current_page: 0,
                page_size: initial_page_size,
                total_pages: 1,
            },
            launch_placeholder: false,
            detail_scroll: 0,
            detail_scroll_available: false,
        }
    }

    /// 刷新游戏列表和成绩数据，但尽量保留当前选中的游戏、分页和详情滚动位置。
    pub fn refresh_preserving_selection(&mut self, games: Vec<GameDescriptor>) {
        let selected_id = self.selected_game().map(|g| g.id.clone());
        let previous_global = self.selected_global_index().unwrap_or(0);
        let previous_scroll = self.detail_scroll;

        self.games = games;
        self.launch_placeholder = false;

        if self.games.is_empty() {
            self.list_state.select(None);
            self.page_state.current_page = 0;
            self.page_state.total_pages = 1;
            self.detail_scroll = 0;
            self.detail_scroll_available = false;
            return;
        }

        let target_global = selected_id
            .and_then(|id| self.games.iter().position(|g| g.id == id))
            .unwrap_or_else(|| previous_global.min(self.games.len().saturating_sub(1)));

        let page_size = self.page_state.page_size.max(1);
        self.page_state.total_pages =
            ((self.games.len() + page_size.saturating_sub(1)) / page_size).max(1);
        self.page_state.current_page =
            (target_global / page_size).min(self.page_state.total_pages.saturating_sub(1));

        let start = self.page_state.current_page * page_size;
        let page_len = (self.games.len() - start).min(page_size);
        let selected_in_page = (target_global - start).min(page_len.saturating_sub(1));
        self.list_state.select(Some(selected_in_page));
        self.detail_scroll = previous_scroll;
    }

    /// Handle game selection input and return the resulting high-level action.
    pub fn handle_event(&mut self, key: KeyEvent) -> Option<GameSelectionAction> {
        if self.launch_placeholder {
            self.launch_placeholder = false;
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(GameSelectionAction::BackToMenu),
            KeyCode::Char('w') | KeyCode::Char('W') => {
                self.scroll_detail_up();
                None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.scroll_detail_down();
                None
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.prev_page();
                None
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.next_page();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Enter => {
                if let Some(game) = self.selected_game_cloned() {
                    return Some(GameSelectionAction::LaunchGame(game));
                }
                None
            }
            _ => None,
        }
    }

    /// Render the game selection page, including the list and detail panel.
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if self.launch_placeholder {
            self.render_launch_placeholder(frame, area);
            return;
        }

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(root[0]);

        self.render_list_panel(frame, columns[0]);
        self.render_detail_panel(frame, columns[1]);

        let mut hints = i18n::t("game_selection.hint.controls");
        if self.detail_scroll_available {
            hints.push_str("  ");
            hints.push_str(&i18n::t("game_selection.hint.detail_scroll"));
        }
        let hint_widget = Paragraph::new(hints)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint_widget, root[1]);
    }

    /// Return the minimum terminal size required for stable layout.
    pub fn minimum_size(&self) -> (u16, u16) {
        let list_title = i18n::t("game_selection.panel.games");
        let detail_title = i18n::t("game_selection.panel.details");
        let hint = i18n::t("game_selection.hint.controls");
        let list_title_w = UnicodeWidthStr::width(list_title.as_str());
        let detail_title_w = UnicodeWidthStr::width(detail_title.as_str());
        let hint_w = UnicodeWidthStr::width(hint.as_str());

        let max_name_w = self
            .games
            .iter()
            .map(|g| UnicodeWidthStr::width(self.localized_game_name(g).as_str()))
            .max()
            .unwrap_or(12);

        let left_w = (max_name_w.max(list_title_w) as u16 + 8).max(26);
        let right_w = (detail_title_w as u16 + 36).max(46);
        let min_w = (left_w + right_w + 4).max(hint_w as u16 + 2).max(60);
        let min_h = 12;
        (min_w, min_h)
    }

    fn render_list_panel(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White))
            .title(format!(" {} ", i18n::t("game_selection.panel.games")));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        self.sync_paging(rows[0].height as usize);

        let page_games = self.current_page_games();
        if page_games.is_empty() {
            let p = Paragraph::new(i18n::t("game_selection.empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(p, rows[0]);
            return;
        }

        let list_width = rows[0].width.saturating_sub(1) as usize;
        let items: Vec<ListItem<'_>> = page_games
            .iter()
            .map(|game| ListItem::new(self.render_game_list_line(game, list_width)))
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
            .highlight_symbol("");
        frame.render_stateful_widget(list, rows[0], &mut self.list_state);

        let has_prev = self.page_state.current_page > 0;
        let has_next = self.page_state.current_page + 1 < self.page_state.total_pages;
        let left = if has_prev {
            i18n::t("game_selection.pager.prev")
        } else {
            String::new()
        };
        let center = format!(
            "{}/{}",
            self.page_state.current_page + 1,
            self.page_state.total_pages
        );
        let right = if has_next {
            i18n::t("game_selection.pager.next")
        } else {
            String::new()
        };

        let left_widget = Paragraph::new(left)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        frame.render_widget(left_widget, rows[1]);

        let center_widget = Paragraph::new(center)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(center_widget, rows[1]);

        let right_widget = Paragraph::new(right)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Right);
        frame.render_widget(right_widget, rows[1]);
    }

    fn render_detail_panel(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White))
            .title(format!(" {} ", i18n::t("game_selection.panel.details")));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(game) = self.selected_game() else {
            let p = Paragraph::new(i18n::t("game_selection.empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(p, inner);
            self.detail_scroll_available = false;
            self.detail_scroll = 0;
            return;
        };

        let sep_len = inner.width as usize;
        let separator = "─".repeat(sep_len.max(1));
        let name = self.localized_game_name(game);
        let description = self.localized_game_description(game);
        let details = self.localized_game_details(game);

        let mut top_lines = vec![Line::from(Span::styled(
            name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))];

        top_lines.push(Line::from(separator.clone()));
        let stat_lines_start = top_lines.len();
        if game.id == "tic_tac_toe" {
        } else if game.is_mod_game() {
            top_lines.extend(format_runtime_best_score_lines(
                game,
                inner.width.saturating_sub(1) as usize,
            ));
            top_lines.push(Line::from(separator.clone()));
            if let Some(package_name) = self.mod_package_name(game) {
                top_lines.push(Line::from(format!(
                    "{} {}",
                    text("mods.info.package", "Package:"),
                    package_name
                )));
            }
            if let Some(author) = self.mod_author(game) {
                top_lines.push(Line::from(format!(
                    "{} {}",
                    text("mods.info.author", "Author:"),
                    author
                )));
            }
            if let Some(version) = self.mod_version(game) {
                top_lines.push(Line::from(format!(
                    "{} {}",
                    text("mods.info.version", "Version:"),
                    version
                )));
            }
            top_lines.push(Line::from(separator.clone()));
        } else {
            top_lines.extend(format_runtime_best_score_lines(
                game,
                inner.width.saturating_sub(1) as usize,
            ));
        }

        if top_lines.len() > stat_lines_start && !game.is_mod_game() {
            top_lines.push(Line::from(separator.clone()));
        }
        top_lines.push(Line::from(i18n::t("game_selection.label.how_to_play")));

        let rich_lines = rich_text::parse_rich_text_wrapped(
            &description,
            inner.width.saturating_sub(1) as usize,
            Style::default().fg(Color::White),
        );
        top_lines.extend(rich_lines);

        let min_details_h = 3u16.min(inner.height.max(1));
        let top_content_h = top_lines.len() as u16;
        let top_h = top_content_h.min(inner.height.saturating_sub(min_details_h));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(top_h), Constraint::Min(min_details_h)])
            .split(inner);

        let top_paragraph = Paragraph::new(top_lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(top_paragraph, chunks[0]);

        let detail_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(chunks[1]);

        frame.render_widget(
            Paragraph::new("─".repeat(detail_rows[0].width as usize))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left),
            detail_rows[0],
        );

        frame.render_widget(
            Paragraph::new(i18n::t("game_selection.label.game_details"))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left),
            detail_rows[1],
        );

        let details_full_lines = rich_text::parse_rich_text_wrapped(
            &details,
            detail_rows[2].width.saturating_sub(2) as usize,
            Style::default().fg(Color::White),
        );

        let viewport_h = detail_rows[2].height as usize;
        let max_scroll = details_full_lines.len().saturating_sub(viewport_h);
        if self.detail_scroll > max_scroll {
            self.detail_scroll = max_scroll;
        }
        self.detail_scroll_available = max_scroll > 0;

        let text_area = if self.detail_scroll_available && detail_rows[2].width > 2 {
            Rect::new(
                detail_rows[2].x,
                detail_rows[2].y,
                detail_rows[2].width - 2,
                detail_rows[2].height,
            )
        } else {
            detail_rows[2]
        };

        let details_paragraph = Paragraph::new(details_full_lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((self.detail_scroll as u16, 0));
        frame.render_widget(details_paragraph, text_area);

        if self.detail_scroll_available && detail_rows[2].width > 2 {
            let scroll_x = detail_rows[2].x + detail_rows[2].width - 1;
            let can_up = self.detail_scroll > 0;
            let can_down = self.detail_scroll < max_scroll;

            frame.render_widget(
                Paragraph::new(if can_up { "↑" } else { " " })
                    .style(Style::default().fg(Color::White)),
                Rect::new(scroll_x, detail_rows[2].y, 1, 1),
            );
            frame.render_widget(
                Paragraph::new(if can_up { "W" } else { " " })
                    .style(Style::default().fg(Color::White)),
                Rect::new(scroll_x, detail_rows[2].y.saturating_add(1), 1, 1),
            );

            if detail_rows[2].height > 4 {
                let track_start = detail_rows[2].y.saturating_add(2);
                let track_len = detail_rows[2].height.saturating_sub(4);
                let pos = if max_scroll == 0 {
                    0
                } else {
                    ((self.detail_scroll * (track_len as usize - 1)) / max_scroll) as u16
                };
                frame.render_widget(
                    Paragraph::new("█").style(Style::default().fg(Color::White)),
                    Rect::new(scroll_x, track_start.saturating_add(pos), 1, 1),
                );
            }

            let d_y = detail_rows[2].y + detail_rows[2].height.saturating_sub(2);
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

    fn render_launch_placeholder(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let width = 32u16.min(area.width.saturating_sub(2));
        let height = 5u16.min(area.height.saturating_sub(2));
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let rect = Rect::new(x, y, width.max(1), height.max(1));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let msg = Paragraph::new(format!(
            "{}\n{}",
            i18n::t("game_selection.placeholder.title"),
            i18n::t("game_selection.placeholder.back")
        ))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
    }

    fn selected_game(&self) -> Option<&GameDescriptor> {
        let selected_in_page = self.list_state.selected()?;
        let global = self.page_state.current_page * self.page_state.page_size + selected_in_page;
        self.games.get(global)
    }

    fn selected_game_cloned(&self) -> Option<GameDescriptor> {
        self.selected_game().cloned()
    }

    fn current_page_games(&self) -> &[GameDescriptor] {
        let start = self.page_state.current_page * self.page_state.page_size;
        let end = (start + self.page_state.page_size).min(self.games.len());
        &self.games[start..end]
    }

    fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    fn reset_detail_scroll(&mut self) {
        self.detail_scroll = 0;
    }

    fn select_prev(&mut self) {
        let page_len = self.current_page_games().len();
        if page_len == 0 {
            self.list_state.select(None);
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        if selected > 0 {
            self.list_state.select(Some(selected - 1));
            self.reset_detail_scroll();
        }
    }

    fn select_next(&mut self) {
        let page_len = self.current_page_games().len();
        if page_len == 0 {
            self.list_state.select(None);
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        if selected + 1 < page_len {
            self.list_state.select(Some(selected + 1));
            self.reset_detail_scroll();
        }
    }

    fn prev_page(&mut self) {
        if self.page_state.current_page > 0 {
            self.page_state.current_page -= 1;
            self.list_state.select(Some(0));
            self.reset_detail_scroll();
        }
    }

    fn next_page(&mut self) {
        if self.page_state.current_page + 1 < self.page_state.total_pages {
            self.page_state.current_page += 1;
            self.list_state.select(Some(0));
            self.reset_detail_scroll();
        }
    }

    fn render_game_list_line(&self, game: &GameDescriptor, width: usize) -> Line<'static> {
        let name = self.localized_game_name(game);
        if !game.is_mod_game() || width == 0 {
            return Line::from(truncate_with_ellipsis(&name, width));
        }

        let badge = text("mods.badge", "MOD");
        let badge_width = UnicodeWidthStr::width(badge.as_str());
        if width <= badge_width + 1 {
            return Line::from(truncate_with_ellipsis(&name, width));
        }

        let left_width = width - badge_width - 1;
        let left = truncate_with_ellipsis(&name, left_width);
        let pad = width.saturating_sub(UnicodeWidthStr::width(left.as_str()) + badge_width);
        Line::from(vec![
            Span::raw(left),
            Span::raw(" ".repeat(pad)),
            Span::styled(
                badge.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    }

    fn localized_game_name(&self, game: &GameDescriptor) -> String {
        if let Some(package) = game.package_info() {
            return resources::resolve_package_text(package, &game.name);
        }
        i18n::t_or(&format!("game.{}.name", game.id), &game.name)
    }

    fn localized_game_description(&self, game: &GameDescriptor) -> String {
        if let Some(package) = game.package_info() {
            return resources::resolve_package_text(package, &game.description);
        }
        i18n::t_or(&format!("game.{}.description", game.id), &game.description)
    }

    fn localized_game_details(&self, game: &GameDescriptor) -> String {
        if let Some(package) = game.package_info() {
            return resources::resolve_package_text(package, &game.detail);
        }
        i18n::t_or(&format!("game.{}.details", game.id), &game.detail)
    }

    fn mod_package_name(&self, game: &GameDescriptor) -> Option<String> {
        let package = game.package_info()?;
        Some(resources::resolve_package_text(
            package,
            package.package_name.as_str(),
        ))
    }

    fn mod_author<'a>(&self, game: &'a GameDescriptor) -> Option<&'a str> {
        game.package_info().map(|package| package.author.as_str())
    }

    fn mod_version<'a>(&self, game: &'a GameDescriptor) -> Option<&'a str> {
        game.package_info().map(|package| package.version.as_str())
    }
    fn selected_global_index(&self) -> Option<usize> {
        let selected_in_page = self.list_state.selected()?;
        let global = self.page_state.current_page * self.page_state.page_size + selected_in_page;
        if global < self.games.len() {
            Some(global)
        } else {
            None
        }
    }

    fn sync_paging(&mut self, visible_rows: usize) {
        let page_size = visible_rows.max(1);
        let selected_global = self.selected_global_index().unwrap_or(0);

        self.page_state.page_size = page_size;
        self.page_state.total_pages =
            ((self.games.len() + page_size.saturating_sub(1)) / page_size).max(1);

        if self.games.is_empty() {
            self.page_state.current_page = 0;
            self.list_state.select(None);
            return;
        }

        let clamped_global = selected_global.min(self.games.len() - 1);
        self.page_state.current_page =
            (clamped_global / page_size).min(self.page_state.total_pages.saturating_sub(1));

        let start = self.page_state.current_page * page_size;
        let page_len = (self.games.len() - start).min(page_size);
        let selected_in_page = (clamped_global - start).min(page_len.saturating_sub(1));
        self.list_state.select(Some(selected_in_page));
    }
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut result = String::new();
    for ch in text.chars() {
        let next = format!("{result}{ch}");
        if UnicodeWidthStr::width(next.as_str()) + 3 > max_width {
            break;
        }
        result.push(ch);
    }
    result.push_str("...");
    result
}

fn text(key: &str, fallback: &str) -> String {
    i18n::t_or(key, fallback)
}

fn format_runtime_best_score_lines(game: &GameDescriptor, width: usize) -> Vec<Line<'static>> {
    let Some(score) = runtime_stats::read_runtime_best_score(&game.id) else {
        let fallback = if let Some(package) = game.package_info() {
            game.best_none
                .as_ref()
                .map(|raw| resources::resolve_package_text(package, raw))
                .unwrap_or_else(|| "--".to_string())
        } else {
            game.best_none.clone().unwrap_or_else(|| "--".to_string())
        };
        return vec![Line::from(fallback)];
    };

    let rendered = match score {
        serde_json::Value::Object(map) => {
            if let Some(best_string_raw) = map.get("best_string").and_then(|value| value.as_str()) {
                let mut rendered = if let Some(package) = game.package_info() {
                    resources::resolve_package_text(package, best_string_raw)
                } else {
                    i18n::t_or(best_string_raw, best_string_raw)
                };
                for (key, value) in &map {
                    if key == "best_string" {
                        continue;
                    }
                    rendered =
                        rendered.replace(&format!("{{{key}}}"), &json_value_to_inline_text(value));
                }
                rendered
            } else {
                "--".to_string()
            }
        }
        serde_json::Value::String(value) => value,
        other => json_value_to_inline_text(&other),
    };

    let lines = rich_text::parse_rich_text_wrapped(
        &rendered,
        width.max(1),
        Style::default().fg(Color::White),
    );
    if lines.is_empty() {
        vec![Line::from("--")]
    } else {
        lines
    }
}

fn json_value_to_inline_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "--".to_string(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => v.clone(),
        serde_json::Value::Array(values) => values
            .iter()
            .map(json_value_to_inline_text)
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, value)| format!("{key}: {}", json_value_to_inline_text(value)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}
