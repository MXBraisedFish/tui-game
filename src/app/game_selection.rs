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
use crate::game::registry::GameSourceKind;
use crate::game::registry::GameDescriptor;
use crate::game::resources;

/// 游戏选择页的完整状态。
pub struct GameSelection {
    games: Vec<GameDescriptor>,
    list_state: ListState,
    page_state: PageState,
    page_jump_input: Option<String>,
    launch_placeholder: bool,
    detail_scroll: usize,
    detail_scroll_available: bool,
    sort_mode: GameSortMode,
    sort_descending: bool,
}

#[derive(Clone, Copy)]
/// 列表分页状态。
struct PageState {
    current_page: usize,
    page_size: usize,
    total_pages: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameSortMode {
    Source,
    Name,
    Author,
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

        let mut this = Self {
            games,
            list_state,
            page_state: PageState {
                current_page: 0,
                page_size: initial_page_size,
                total_pages: 1,
            },
            page_jump_input: None,
            launch_placeholder: false,
            detail_scroll: 0,
            detail_scroll_available: false,
            sort_mode: GameSortMode::Source,
            sort_descending: false,
        };
        this.apply_sort();
        this
    }

    /// 刷新游戏列表和成绩数据，但尽量保留当前选中的游戏、分页和详情滚动位置。
    pub fn refresh_preserving_selection(&mut self, games: Vec<GameDescriptor>) {
        let selected_id = self.selected_game().map(|g| g.id.clone());
        let previous_global = self.selected_global_index().unwrap_or(0);
        let previous_scroll = self.detail_scroll;

        self.games = games;
        self.apply_sort();
        self.page_jump_input = None;
        self.launch_placeholder = false;

        if self.games.is_empty() {
            self.list_state.select(None);
            self.page_state.current_page = 0;
            self.page_state.total_pages = 1;
            self.page_jump_input = None;
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

        if let Some(input) = self.page_jump_input.as_mut() {
            match key.code {
                KeyCode::Esc => self.page_jump_input = None,
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) if ch.is_ascii_digit() => {
                    if input.len() < 4 {
                        input.push(ch);
                    }
                }
                KeyCode::Enter => {
                    if let Ok(page) = input.parse::<usize>()
                        && (1..=self.page_state.total_pages.max(1)).contains(&page)
                    {
                        self.page_state.current_page = page - 1;
                        self.list_state.select(Some(0));
                        self.reset_detail_scroll();
                    }
                    self.page_jump_input = None;
                }
                _ => {}
            }
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
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if self.page_state.total_pages > 1 {
                    self.page_jump_input = Some(String::new());
                }
                None
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                let next = match self.sort_mode {
                    GameSortMode::Source => GameSortMode::Name,
                    GameSortMode::Name => GameSortMode::Author,
                    GameSortMode::Author => GameSortMode::Source,
                };
                self.set_sort_mode(next);
                None
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.toggle_sort_order();
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

        let hint_lines = wrap_game_hint_lines(
            &build_game_hint_segments(self.detail_scroll_available),
            area.width.max(1) as usize,
        );
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(hint_lines.len().max(1) as u16)])
            .split(area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(root[0]);

        self.render_list_panel(frame, columns[0]);
        self.render_detail_panel(frame, columns[1]);

        let hint_widget = Paragraph::new(hint_lines)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint_widget, root[1]);
    }

    /// Return the minimum terminal size required for stable layout.
    pub fn minimum_size(&self) -> (u16, u16) {
        let list_title = i18n::t("game_selection.panel.games");
        let detail_title = i18n::t("game_selection.panel.details");
        let hint = build_game_hint_segments(true).join("  ");
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
            .title(self.list_title());

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
        let center = if let Some(input) = &self.page_jump_input {
            let input_text = if input.is_empty() {
                "_".to_string()
            } else {
                input.clone()
            };
            Line::from(vec![
                Span::styled(
                    input_text,
                    Style::default()
                        .fg(if input.is_empty() { Color::Yellow } else { Color::Black })
                        .bg(Color::Yellow),
                ),
                Span::styled(
                    format!("/{}", self.page_state.total_pages.max(1)),
                    Style::default().fg(Color::White),
                ),
            ])
        } else {
            Line::from(Span::styled(
                format!(
                    "{}/{}",
                    self.page_state.current_page + 1,
                    self.page_state.total_pages
                ),
                Style::default().fg(Color::White),
            ))
        };
        let right = if has_next {
            i18n::t("game_selection.pager.next")
        } else {
            String::new()
        };

        let left_widget = Paragraph::new(left)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        frame.render_widget(left_widget, rows[1]);

        let center_widget = Paragraph::new(center).alignment(Alignment::Center);
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
            if game.has_best_score {
                top_lines.extend(format_runtime_best_score_lines(
                    game,
                    inner.width.saturating_sub(1) as usize,
                ));
                top_lines.push(Line::from(separator.clone()));
            }
            if let Some((package_name, allow_rich)) = self.mod_package_name(game) {
                top_lines.extend(label_manifest_value_lines(
                    text("mods.info.package", "Package:"),
                    package_name,
                    allow_rich,
                    inner.width.saturating_sub(1) as usize,
                    Style::default().fg(Color::White),
                ));
            }
            if let Some(author) = self.mod_author(game) {
                top_lines.extend(label_manifest_value_lines(
                    text("mods.info.author", "Author:"),
                    author,
                    true,
                    inner.width.saturating_sub(1) as usize,
                    Style::default().fg(Color::White),
                ));
            }
            if let Some(version) = self.mod_version(game) {
                top_lines.extend(label_manifest_value_lines(
                    text("mods.info.version", "Version:"),
                    version,
                    true,
                    inner.width.saturating_sub(1) as usize,
                    Style::default().fg(Color::White),
                ));
            }
            top_lines.push(Line::from(separator.clone()));
        } else if game.has_best_score {
            top_lines.extend(format_runtime_best_score_lines(
                game,
                inner.width.saturating_sub(1) as usize,
            ));
        }

        if top_lines.len() > stat_lines_start && !game.is_mod_game() {
            top_lines.push(Line::from(separator.clone()));
        }
        top_lines.push(Line::from(
            Span::styled(
            i18n::t("game_selection.label.how_to_play"),
            Style::default().fg(Color::Yellow),
        )   
        ));

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
                .style(Style::default().fg(Color::Yellow))
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

    fn apply_sort(&mut self) {
        let selected_id = self.selected_game().map(|game| game.id.clone());
        let sort_mode = self.sort_mode;
        let descending = self.sort_descending;
        self.games.sort_by(|left, right| {
            let ordering = compare_games(left, right, sort_mode);
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
        self.restore_selected_game(selected_id.as_deref());
    }

    fn restore_selected_game(&mut self, id: Option<&str>) {
        if self.games.is_empty() {
            self.page_state.current_page = 0;
            self.list_state.select(None);
            return;
        }

        let target_global = id
            .and_then(|value| self.games.iter().position(|game| game.id == value))
            .or_else(|| self.selected_global_index())
            .unwrap_or(0)
            .min(self.games.len().saturating_sub(1));

        let page_size = self.page_state.page_size.max(1);
        self.page_state.total_pages =
            ((self.games.len() + page_size.saturating_sub(1)) / page_size).max(1);
        self.page_state.current_page =
            (target_global / page_size).min(self.page_state.total_pages.saturating_sub(1));

        let start = self.page_state.current_page * page_size;
        let page_len = (self.games.len() - start).min(page_size);
        let selected_in_page = (target_global - start).min(page_len.saturating_sub(1));
        self.list_state.select(Some(selected_in_page));
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

    fn set_sort_mode(&mut self, mode: GameSortMode) {
        self.sort_mode = mode;
        self.apply_sort();
        self.reset_detail_scroll();
    }

    fn toggle_sort_order(&mut self) {
        self.sort_descending = !self.sort_descending;
        self.apply_sort();
        self.reset_detail_scroll();
    }

    fn list_title(&self) -> Line<'static> {
        let order_text = if self.sort_descending {
            format!("\u{2191}{}", text("settings.mods.order.desc", "Descending"))
        } else {
            format!("\u{2193}{}", text("settings.mods.order.asc", "Ascending"))
        };

        Line::from(vec![
            Span::raw(" "),
            Span::styled(
                i18n::t("game_selection.panel.games"),
                Style::default().fg(Color::White),
            ),
            Span::styled(" *", Style::default().fg(Color::White)),
            Span::styled(
                game_sort_label(self.sort_mode),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().fg(Color::White)),
            Span::styled("[", Style::default().fg(Color::White)),
            Span::styled(order_text, Style::default().fg(Color::DarkGray)),
            Span::styled("]", Style::default().fg(Color::White)),
            Span::raw(" "),
        ])
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
        game.display_name.clone()
    }

    fn localized_game_description(&self, game: &GameDescriptor) -> String {
        game.display_description.clone()
    }

    fn localized_game_details(&self, game: &GameDescriptor) -> String {
        game.display_detail.clone()
    }

    fn mod_package_name(&self, game: &GameDescriptor) -> Option<(String, bool)> {
        Some((
            game.display_package_name.clone()?,
            game.display_package_name_allows_rich,
        ))
    }

    fn mod_author(&self, game: &GameDescriptor) -> Option<String> {
        game.display_package_author.clone()
    }

    fn mod_version(&self, game: &GameDescriptor) -> Option<String> {
        game.display_package_version.clone()
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

fn game_sort_label(mode: GameSortMode) -> String {
    match mode {
        GameSortMode::Source => text("game_selection.sort.source", "Official & Third-party"),
        GameSortMode::Name => text("game_selection.sort.name", "Name"),
        GameSortMode::Author => text("game_selection.sort.author", "Author"),
    }
}

fn compare_games(left: &GameDescriptor, right: &GameDescriptor, mode: GameSortMode) -> std::cmp::Ordering {
    match mode {
        GameSortMode::Source => source_rank(&left.source)
            .cmp(&source_rank(&right.source))
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| cmp_lowercase(&left.display_author, &right.display_author))
            .then_with(|| left.id.cmp(&right.id)),
        GameSortMode::Name => cmp_lowercase(&left.display_name, &right.display_name)
            .then_with(|| source_rank(&left.source).cmp(&source_rank(&right.source)))
            .then_with(|| cmp_lowercase(&left.display_author, &right.display_author))
            .then_with(|| left.id.cmp(&right.id)),
        GameSortMode::Author => cmp_lowercase(&left.display_author, &right.display_author)
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| source_rank(&left.source).cmp(&source_rank(&right.source)))
            .then_with(|| left.id.cmp(&right.id)),
    }
}

fn source_rank(source: &GameSourceKind) -> u8 {
    match source {
        GameSourceKind::Official => 0,
        GameSourceKind::Mod => 1,
    }
}

fn cmp_lowercase(left: &str, right: &str) -> std::cmp::Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn build_game_hint_segments(include_scroll: bool) -> Vec<String> {
    let mut segments = vec![
        text("game_selection.hint.confirm", "[Enter] Confirm"),
        text("game_selection.hint.jump", "[P] Jump"),
        text("game_selection.hint.sort_mode", "[Z] Sort"),
        text("game_selection.hint.sort_order", "[X] Order"),
        text("game_selection.hint.move", "[↑]/[↓] Select Game"),
        text("game_selection.hint.page", "[Q]/[E] Change Page"),
        text("game_selection.hint.back", "[ESC] Return to Menu"),
    ];
    if include_scroll {
        segments.push(i18n::t("game_selection.hint.detail_scroll"));
    }
    segments
}

fn wrap_game_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    if width == 0 || segments.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines = Vec::new();
    let mut current_segments: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for segment in segments {
        let segment_width = UnicodeWidthStr::width(segment.as_str());
        let separator_width = if current_segments.is_empty() { 0 } else { 2 };

        if !current_segments.is_empty() && current_width + separator_width + segment_width > width {
            lines.push(Line::from(std::mem::take(&mut current_segments)));
            current_width = 0;
        }

        if !current_segments.is_empty() {
            current_segments.push(Span::raw("  "));
            current_width += 2;
        }
        current_segments.push(Span::raw(segment.clone()));
        current_width += segment_width;
    }

    if !current_segments.is_empty() {
        lines.push(Line::from(current_segments));
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    lines
}

fn label_manifest_value_lines(
    label: String,
    value: String,
    allow_rich: bool,
    width: usize,
    value_style: Style,
) -> Vec<Line<'static>> {
    if !allow_rich || !value.starts_with("f%") {
        return vec![Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(value, value_style.add_modifier(Modifier::BOLD)),
        ])];
    }

    let mut parsed = rich_text::parse_rich_text_wrapped(&value, usize::MAX / 8, value_style);
    if parsed.is_empty() {
        return vec![Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White)),
            Span::raw(" "),
        ])];
    }

    let mut first_spans = vec![
        Span::styled(label.clone(), Style::default().fg(Color::White)),
        Span::raw(" "),
    ];
    first_spans.extend(parsed.remove(0).spans);

    let mut lines = vec![Line::from(first_spans)];
    let indent = " ".repeat(UnicodeWidthStr::width(label.as_str()) + 1);
    let continuation_width = width.saturating_sub(indent.len()).max(1);
    for line in parsed {
        let wrapped = crop_line_center_to_width(&line, continuation_width);
        let mut spans = vec![Span::styled(indent.clone(), Style::default().fg(Color::White))];
        spans.extend(wrapped.spans);
        lines.push(Line::from(spans));
    }
    lines
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

fn format_runtime_best_score_lines(game: &GameDescriptor, width: usize) -> Vec<Line<'static>> {
    if !game.has_best_score {
        return Vec::new();
    }
    let Some(score) = runtime_stats::read_runtime_best_score(&game.id) else {
        let fallback = resolved_best_none_text(game);
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

fn resolved_best_none_text(game: &GameDescriptor) -> String {
    match game.display_best_none.clone() {
        Some(value) if !value.trim().is_empty() => value,
        _ => "---".to_string(),
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
