//! Rust implementation of the game list page.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::package::kind::GamePackage;
use crate::host_engine::runtime::ui::components::SplitPanel;
use crate::host_engine::runtime::ui::pages::common::{
    draw_footer, is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SortMode {
    Source,
    Name,
    Author,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SortOrder {
    Asc,
    Desc,
}

pub struct GameListPage {
    selected_index: usize,
    page: usize,
    user_page: usize,
    jump_mode: bool,
    detail_scroll: usize,
    sort_mode: SortMode,
    sort_order: SortOrder,
    pending_navigation: Option<UiNavigation>,
}

impl GameListPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            page: 1,
            user_page: 0,
            jump_mode: false,
            detail_scroll: 0,
            sort_mode: SortMode::Source,
            sort_order: SortOrder::Asc,
            pending_navigation: None,
        }
    }

    fn move_previous(&mut self, count: usize, capacity: usize) {
        if count == 0 {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            count - 1
        } else {
            self.selected_index - 1
        };
        self.sync_page_to_selection(capacity);
        self.detail_scroll = 0;
    }

    fn move_next(&mut self, count: usize, capacity: usize) {
        if count == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % count;
        self.sync_page_to_selection(capacity);
        self.detail_scroll = 0;
    }

    fn previous_page(&mut self, count: usize, capacity: usize) {
        let pages = pages(count, capacity);
        self.page = self.page.saturating_sub(1).max(1).min(pages);
        self.selected_index = page_start(self.page, capacity).min(count.saturating_sub(1));
        self.detail_scroll = 0;
    }

    fn next_page(&mut self, count: usize, capacity: usize) {
        let pages = pages(count, capacity);
        self.page = self.page.saturating_add(1).min(pages);
        self.selected_index = page_start(self.page, capacity).min(count.saturating_sub(1));
        self.detail_scroll = 0;
    }

    fn toggle_order(&mut self, count: usize, capacity: usize) {
        self.sort_order = match self.sort_order {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        };
        self.clamp_selection(count, capacity);
    }

    fn toggle_sort(&mut self, count: usize, capacity: usize) {
        self.sort_mode = match self.sort_mode {
            SortMode::Source => SortMode::Name,
            SortMode::Name => SortMode::Author,
            SortMode::Author => SortMode::Source,
        };
        self.clamp_selection(count, capacity);
    }

    fn start_jump(&mut self) {
        self.jump_mode = true;
        self.user_page = 0;
    }

    fn cancel_jump(&mut self) {
        self.jump_mode = false;
        self.user_page = 0;
    }

    fn push_jump_digit(&mut self, digit: usize) {
        if !self.jump_mode {
            return;
        }
        self.user_page = self
            .user_page
            .saturating_mul(10)
            .saturating_add(digit)
            .min(9999);
    }

    fn confirm_jump(&mut self, count: usize, capacity: usize) {
        if !self.jump_mode {
            return;
        }
        let pages = pages(count, capacity);
        self.page = self.user_page.max(1).min(pages);
        self.selected_index = page_start(self.page, capacity).min(count.saturating_sub(1));
        self.jump_mode = false;
        self.user_page = 0;
        self.detail_scroll = 0;
    }

    fn clamp_selection(&mut self, count: usize, capacity: usize) {
        if count == 0 {
            self.selected_index = 0;
            self.page = 1;
            self.detail_scroll = 0;
            return;
        }
        self.selected_index = self.selected_index.min(count - 1);
        self.sync_page_to_selection(capacity);
        self.detail_scroll = 0;
    }

    fn sync_page_to_selection(&mut self, capacity: usize) {
        self.page = self.selected_index / capacity.max(1) + 1;
    }
}

impl UiPage for GameListPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::GameList
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }

        let count = ctx.packages.games().len();
        let panel = SplitPanel::new(ctx.terminal_size.width, ctx.terminal_size.height, 2);
        let capacity = list_capacity(&panel);

        if self.jump_mode {
            match name.as_str() {
                "confirm" | "enter" => self.confirm_jump(count, capacity),
                "back" | "return" | "esc" | "q" => self.cancel_jump(),
                digit if digit.len() == 1 => {
                    if let Some(value) = digit.chars().next().and_then(|ch| ch.to_digit(10)) {
                        self.push_jump_digit(value as usize);
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        match name.as_str() {
            "prev_option" | "up" | "arrowup" => self.move_previous(count, capacity),
            "next_option" | "down" | "arrowdown" => self.move_next(count, capacity),
            "prev_page" | "q" => self.previous_page(count, capacity),
            "next_page" | "e" => self.next_page(count, capacity),
            "jump" | "j" => self.start_jump(),
            "order" | "z" => self.toggle_order(count, capacity),
            "sort" | "x" => self.toggle_sort(count, capacity),
            "scroll_up" | "w" => self.detail_scroll = self.detail_scroll.saturating_sub(1),
            "scroll_down" | "s" => self.detail_scroll = self.detail_scroll.saturating_add(1),
            "back" | "return" | "esc" => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Home));
            }
            "confirm" | "enter" => {
                let games = sorted_games(ctx.packages.games(), self.sort_mode, self.sort_order);
                if let Some(game) = games.get(self.selected_index) {
                    self.pending_navigation = Some(UiNavigation::StartGame(game.uid.clone()));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        let panel = SplitPanel::new(ctx.terminal_size.width, ctx.terminal_size.height, 2);
        panel.render_borders_with_theme(canvas, ctx, "", &ctx.i18n.game_list.info_title)?;
        draw_left_header(canvas, ctx, &panel, self.sort_order, self.sort_mode)?;

        let games = sorted_games(ctx.packages.games(), self.sort_mode, self.sort_order);
        let capacity = list_capacity(&panel);
        let pages = pages(games.len(), capacity);
        let page = self.page.max(1).min(pages);
        let selected_index = self.selected_index.min(games.len().saturating_sub(1));
        render_game_names(canvas, ctx, &panel, &games, selected_index, page, capacity)?;
        render_page_line(
            canvas,
            ctx,
            &panel,
            page,
            pages,
            self.jump_mode,
            self.user_page,
        )?;
        render_game_detail(
            canvas,
            ctx,
            &panel,
            games.get(selected_index).copied(),
            self.detail_scroll,
        )?;
        draw_footer(
            canvas,
            ctx,
            action_hint_text(ctx, pages, self.jump_mode).as_str(),
        )?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

fn draw_left_header(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    order: SortOrder,
    sort: SortMode,
) -> UiResult<()> {
    let x = panel.left_x.saturating_add(2);
    let y = panel.left_y;
    let title = format!(" {} *", ctx.i18n.game_list.list_title);
    let order_text = match order {
        SortOrder::Asc => ctx.i18n.game_list.info_order_ascending.as_str(),
        SortOrder::Desc => ctx.i18n.game_list.info_order_descending.as_str(),
    };
    let sort_text = match sort {
        SortMode::Source => ctx.i18n.game_list.info_sort_mod_official.as_str(),
        SortMode::Name => ctx.i18n.game_list.info_sort_name.as_str(),
        SortMode::Author => ctx.i18n.game_list.info_sort_author.as_str(),
    };

    let mut cursor = x;
    cursor = draw_segment(
        canvas,
        cursor,
        y,
        title.as_str(),
        theme_color(ctx, "border.primary", "white"),
    )?;
    cursor = draw_segment(
        canvas,
        cursor,
        y,
        "[",
        theme_color(ctx, "border.primary", "white"),
    )?;
    cursor = draw_segment(
        canvas,
        cursor,
        y,
        order_text,
        theme_color(ctx, "state.success", "green"),
    )?;
    cursor = draw_segment(
        canvas,
        cursor,
        y,
        "] ",
        theme_color(ctx, "border.primary", "white"),
    )?;
    let _ = draw_segment(
        canvas,
        cursor,
        y,
        sort_text,
        theme_color(ctx, "state.warning", "yellow"),
    )?;
    Ok(())
}

fn draw_segment(canvas: &mut Canvas, x: u16, y: u16, text: &str, color: String) -> UiResult<u16> {
    canvas.draw_text_styled(x, y, text, Some(color), None, vec![STYLE_BOLD])?;
    Ok(x.saturating_add(UnicodeWidthStr::width(text) as u16))
}

fn render_game_names(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    games: &[&GamePackage],
    selected_index: usize,
    page: usize,
    capacity: usize,
) -> UiResult<()> {
    let inner_x = panel.left_x.saturating_add(1);
    let inner_width = panel.left_width.saturating_sub(2);
    let start = page_start(page, capacity);
    let end = start.saturating_add(capacity).min(games.len());

    if games.is_empty() {
        let text = &ctx.i18n.game_list.none_game;
        let x = inner_x.saturating_add(
            inner_width.saturating_sub(UnicodeWidthStr::width(text.as_str()) as u16) / 2,
        );
        let y = panel.left_y.saturating_add(panel.height / 2);
        canvas.draw_text_styled(
            x,
            y,
            text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
        return Ok(());
    }

    for (row, index) in (start..end).enumerate() {
        let game = games[index];
        let y = panel.left_y.saturating_add(1 + row as u16);
        let selected = index == selected_index;
        if selected {
            canvas.fill_rect(
                inner_x,
                y,
                inner_width,
                1,
                ' ',
                None,
                Some(theme_color(ctx, "background.selected", "#78a8da")),
            )?;
        }

        let is_mod = game.source_label.contains("mod");
        let mark = if is_mod {
            format!(" {}", ctx.i18n.game_list.mod_label)
        } else {
            String::new()
        };
        let mark_width = UnicodeWidthStr::width(mark.as_str()) as u16;
        let name_width = inner_width.saturating_sub(mark_width).saturating_sub(1);
        let name = truncate_to_width(game.game_name.as_str(), name_width as usize);
        let fg = if selected {
            theme_color(ctx, "text.on_selected", "black")
        } else {
            theme_color(ctx, "text.primary", "white")
        };
        canvas.draw_text_styled(inner_x, y, name, Some(fg.clone()), None, Vec::new())?;
        if is_mod {
            let mark_x = inner_x
                .saturating_add(inner_width)
                .saturating_sub(mark_width);
            canvas.draw_text_styled(
                mark_x,
                y,
                mark,
                Some(if selected {
                    fg
                } else {
                    theme_color(ctx, "state.warning", "yellow")
                }),
                None,
                Vec::new(),
            )?;
        }
    }
    Ok(())
}

fn render_page_line(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    page: usize,
    pages: usize,
    jump_mode: bool,
    user_page: usize,
) -> UiResult<()> {
    let y = panel.left_y.saturating_add(panel.height.saturating_sub(2));
    let inner_x = panel.left_x.saturating_add(1);
    let inner_width = panel.left_width.saturating_sub(2);
    let page_text = if jump_mode {
        user_page.max(0).to_string()
    } else {
        format!("{page}/{pages}")
    };
    let page_width = UnicodeWidthStr::width(page_text.as_str()) as u16;
    let page_x = inner_x.saturating_add(inner_width.saturating_sub(page_width) / 2);
    canvas.draw_text_styled(
        page_x,
        y,
        page_text,
        Some(if jump_mode {
            theme_color(ctx, "text.on_warning", "black")
        } else {
            theme_color(ctx, "text.muted", "dark_gray")
        }),
        if jump_mode {
            Some(theme_color(ctx, "state.warning", "yellow"))
        } else {
            None
        },
        vec![STYLE_BOLD],
    )?;

    if page > 1 {
        canvas.draw_text_styled(
            inner_x,
            y,
            format!("◀ [{}]", key_hint(ctx, "prev_page", "Q")),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
    }
    if page < pages {
        let text = format!("[{}] ▶", key_hint(ctx, "next_page", "E"));
        let text_width = UnicodeWidthStr::width(text.as_str()) as u16;
        canvas.draw_text_styled(
            inner_x
                .saturating_add(inner_width)
                .saturating_sub(text_width),
            y,
            text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
    }
    Ok(())
}

fn render_game_detail(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    game: Option<&GamePackage>,
    detail_scroll: usize,
) -> UiResult<()> {
    let x = panel.right_x.saturating_add(1);
    let y = panel.right_y.saturating_add(1);
    let content_width = panel.right_width.saturating_sub(2);
    let content_height = panel.height.saturating_sub(2);
    let Some(game) = game else {
        let text = &ctx.i18n.game_list.none_info;
        let text_width = UnicodeWidthStr::width(text.as_str()) as u16;
        canvas.draw_text_styled(
            x.saturating_add(content_width.saturating_sub(text_width) / 2),
            y.saturating_add(content_height / 2),
            text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
        return Ok(());
    };

    let separator = "─".repeat(content_width as usize);
    let mut rows = Vec::new();
    push_row(&mut rows, Row::Text(game.game_name.clone()));
    push_row(&mut rows, Row::Separator(separator.clone()));
    push_row(
        &mut rows,
        Row::LabelValue(
            ctx.i18n.game_list.info_mod.clone(),
            game.package_name.clone(),
        ),
    );
    push_row(
        &mut rows,
        Row::LabelValue(ctx.i18n.game_list.info_author.clone(), game.author.clone()),
    );
    push_row(
        &mut rows,
        Row::LabelValue(
            ctx.i18n.game_list.info_version.clone(),
            game.version.clone(),
        ),
    );
    if !game.description.trim().is_empty() {
        push_row(&mut rows, Row::Separator(separator.clone()));
        push_wrapped_text(&mut rows, game.description.as_str(), content_width as usize);
    }
    if !game.detail.trim().is_empty() {
        push_row(&mut rows, Row::Separator(separator));
        push_wrapped_text(&mut rows, game.detail.as_str(), content_width as usize);
    }

    let max_scroll = rows.len().saturating_sub(content_height as usize);
    let scroll = detail_scroll.min(max_scroll);
    for (offset, row) in rows
        .into_iter()
        .skip(scroll)
        .take(content_height as usize)
        .enumerate()
    {
        let row_y = y.saturating_add(offset as u16);
        match row {
            Row::Text(text) => {
                canvas.draw_rich_text_styled(
                    x,
                    row_y,
                    text,
                    Some(theme_color(ctx, "text.primary", "white")),
                    None,
                    Vec::new(),
                )?;
            }
            Row::Separator(text) => {
                canvas.draw_text_styled(
                    x,
                    row_y,
                    text,
                    Some(theme_color(ctx, "border.primary", "white")),
                    None,
                    Vec::new(),
                )?;
            }
            Row::LabelValue(label, value) => {
                let label_width = UnicodeWidthStr::width(label.as_str()) as u16;
                canvas.draw_text_styled(
                    x,
                    row_y,
                    label.as_str(),
                    Some(theme_color(ctx, "state.warning", "yellow")),
                    None,
                    vec![STYLE_BOLD],
                )?;
                canvas.draw_rich_text_styled(
                    x.saturating_add(label_width),
                    row_y,
                    value,
                    Some(theme_color(ctx, "text.primary", "white")),
                    None,
                    Vec::new(),
                )?;
            }
        }
    }

    render_scroll_hint(canvas, ctx, panel, scroll, max_scroll, content_height)?;
    Ok(())
}

fn render_scroll_hint(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    scroll: usize,
    max_scroll: usize,
    content_height: u16,
) -> UiResult<()> {
    if max_scroll == 0 {
        return Ok(());
    }
    let x = panel
        .right_x
        .saturating_add(panel.right_width.saturating_sub(2));
    let top_y = panel.right_y.saturating_add(2);
    let bottom_y = panel.right_y.saturating_add(panel.height.saturating_sub(3));
    let color = theme_color(ctx, "text.muted", "dark_gray");
    if scroll > 0 {
        canvas.draw_text_styled(x, top_y, "↑", Some(color.clone()), None, vec![STYLE_BOLD])?;
        canvas.draw_text_styled(
            x,
            top_y.saturating_add(1),
            key_hint(ctx, "scroll_up", "W"),
            Some(color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
    }
    if scroll < max_scroll {
        canvas.draw_text_styled(
            x,
            bottom_y.saturating_sub(1),
            key_hint(ctx, "scroll_down", "S"),
            Some(color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
        canvas.draw_text_styled(x, bottom_y, "↓", Some(color), None, vec![STYLE_BOLD])?;
    }

    let track_height = content_height.saturating_sub(8).max(1);
    let slider_y = top_y
        .saturating_add(4)
        .saturating_add(((scroll as u32 * track_height as u32) / max_scroll.max(1) as u32) as u16);
    canvas.draw_text_styled(
        x,
        slider_y,
        "█",
        Some(theme_color(ctx, "text.muted", "dark_gray")),
        None,
        vec![STYLE_BOLD],
    )?;
    Ok(())
}

#[derive(Clone)]
enum Row {
    Text(String),
    Separator(String),
    LabelValue(String, String),
}

fn push_row(rows: &mut Vec<Row>, row: Row) {
    rows.push(row);
}

fn push_wrapped_text(rows: &mut Vec<Row>, text: &str, width: usize) {
    for line in wrap_text(text, width) {
        rows.push(Row::Text(line));
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut rows = Vec::new();
    for raw_line in text.lines() {
        let mut line = String::new();
        let mut line_width = 0usize;
        for ch in raw_line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if line_width > 0 && line_width.saturating_add(ch_width) > width {
                rows.push(line);
                line = String::new();
                line_width = 0;
            }
            line.push(ch);
            line_width = line_width.saturating_add(ch_width);
        }
        rows.push(line);
    }
    rows
}

fn sorted_games<'a>(
    games: &'a [GamePackage],
    sort_mode: SortMode,
    sort_order: SortOrder,
) -> Vec<&'a GamePackage> {
    let mut games: Vec<&GamePackage> = games.iter().collect();
    games.sort_by(|a, b| {
        let primary = match sort_mode {
            SortMode::Source => compare_field(source_sort_key(a), source_sort_key(b)),
            SortMode::Name => compare_field(a.game_name.as_str(), b.game_name.as_str()),
            SortMode::Author => compare_field(a.author.as_str(), b.author.as_str()),
        };
        let fallback = compare_field(a.game_name.as_str(), b.game_name.as_str())
            .then_with(|| compare_field(a.author.as_str(), b.author.as_str()));
        primary.then(fallback)
    });
    if sort_order == SortOrder::Desc {
        games.reverse();
    }
    games
}

fn compare_field(left: &str, right: &str) -> std::cmp::Ordering {
    UnicodeWidthStr::width(left)
        .cmp(&UnicodeWidthStr::width(right))
        .then_with(|| left.cmp(right))
}

fn source_sort_key(game: &GamePackage) -> &str {
    if game.source_label.contains("mod") {
        "mod"
    } else {
        "official"
    }
}

fn action_hint_text(ctx: &UiContext, pages: usize, jump_mode: bool) -> String {
    if jump_mode {
        return format!(
            "[1]-[9] {}  [{}] {}  [{}] {}",
            ctx.i18n.game_list.info_sort_name,
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.game_list_confirm,
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.game_list_cancel
        );
    }

    let mut parts = vec![
        format!(
            "[{}/{}] {}",
            key_hint(ctx, "prev_option", "↑"),
            key_hint(ctx, "next_option", "↓"),
            ctx.i18n.key.game_list_select
        ),
        format!(
            "[{}] {}",
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.game_list_start
        ),
        format!(
            "[{}/{}] {}",
            key_hint(ctx, "scroll_up", "W"),
            key_hint(ctx, "scroll_down", "S"),
            ctx.i18n.key.game_list_scroll
        ),
        format!(
            "[{}] {}",
            key_hint(ctx, "order", "Z"),
            ctx.i18n.key.game_list_order
        ),
        format!(
            "[{}] {}",
            key_hint(ctx, "sort", "X"),
            ctx.i18n.key.game_list_sort
        ),
    ];
    if pages > 1 {
        parts.push(format!(
            "[{}] {}",
            key_hint(ctx, "jump", "J"),
            ctx.i18n.key.game_list_jump
        ));
        parts.push(format!(
            "[{}/{}] {}",
            key_hint(ctx, "prev_page", "Q"),
            key_hint(ctx, "next_page", "E"),
            ctx.i18n.key.game_list_flip
        ));
    }
    parts.push(format!(
        "[{}] {}",
        key_hint(ctx, "back", "Esc"),
        ctx.i18n.key.game_list_back
    ));
    parts.join("  ")
}

fn list_capacity(panel: &SplitPanel) -> usize {
    panel.height.saturating_sub(4).max(1) as usize
}

fn pages(count: usize, capacity: usize) -> usize {
    count.max(1).div_ceil(capacity.max(1))
}

fn page_start(page: usize, capacity: usize) -> usize {
    page.saturating_sub(1).saturating_mul(capacity.max(1))
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    let mut output = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width.saturating_add(ch_width) > max_width {
            break;
        }
        output.push(ch);
        width = width.saturating_add(ch_width);
    }
    output
}
