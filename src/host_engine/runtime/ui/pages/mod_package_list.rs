//! Rust implementations of Mod package list pages.

use std::cmp::Ordering;

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::package::kind::{GamePackage, OverlayPackage};
use crate::host_engine::runtime::ui::components::SplitPanel;
use crate::host_engine::runtime::ui::pages::common::{
    is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

// ---------------------------------------------------------------------------
// Shared enums
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModListSortMode {
    Name,
    Author,
    SafeMode,
    Toggle,
    Debug,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModListSortOrder {
    Asc,
    Desc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModListDisplayMode {
    Full,
    Brief,
}

// ---------------------------------------------------------------------------
// Page state
// ---------------------------------------------------------------------------

struct ModListPageState {
    selected_index: usize,
    page: usize,
    sort_mode: ModListSortMode,
    sort_order: ModListSortOrder,
    list_mode: ModListDisplayMode,
    jump_mode: bool,
    jump_input: usize,
    info_scroll: usize,
}

impl ModListPageState {
    fn new() -> Self {
        Self {
            selected_index: 0,
            page: 1,
            sort_mode: ModListSortMode::Name,
            sort_order: ModListSortOrder::Asc,
            list_mode: ModListDisplayMode::Full,
            jump_mode: false,
            jump_input: 0,
            info_scroll: 0,
        }
    }

    fn normalize(&mut self, items: &[PackageItem]) {
        if items.is_empty() {
            self.selected_index = 0;
            self.page = 1;
            return;
        }
        if self.selected_index >= items.len() {
            self.selected_index = items.len() - 1;
        }
        // page is normalized during render when layout is available
    }
}

// ---------------------------------------------------------------------------
// Package item wrapper for uniform access
// ---------------------------------------------------------------------------

enum PackageItem<'a> {
    Game(&'a GamePackage),
    Overlay(&'a OverlayPackage),
}

impl<'a> PackageItem<'a> {
    fn uid(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.uid.as_str(),
            PackageItem::Overlay(pkg) => pkg.uid.as_str(),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.package_name.as_str(),
            PackageItem::Overlay(pkg) => pkg.display_name.as_str(),
        }
    }

    fn author(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.author.as_str(),
            PackageItem::Overlay(pkg) => pkg.author.as_str(),
        }
    }

    fn version(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.version.as_str(),
            PackageItem::Overlay(pkg) => pkg.version.as_str(),
        }
    }

    fn introduction(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.introduction.as_str(),
            PackageItem::Overlay(pkg) => pkg.introduction.as_str(),
        }
    }

    fn icon(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.icon.as_str(),
            PackageItem::Overlay(pkg) => pkg.icon.as_str(),
        }
    }

    fn banner(&self) -> &str {
        match self {
            PackageItem::Game(pkg) => pkg.banner.as_str(),
            PackageItem::Overlay(pkg) => pkg.banner.as_str(),
        }
    }

    fn write_permission(&self) -> bool {
        match self {
            PackageItem::Game(pkg) => pkg.write_permission,
            PackageItem::Overlay(_) => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Kind enum for differentiation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModListKind {
    Game,
    Screensaver,
    Boss,
}

impl ModListKind {
    fn has_safe_mode(self) -> bool {
        matches!(self, Self::Game)
    }

    fn sort_cycle(self, current: ModListSortMode) -> ModListSortMode {
        match self {
            Self::Game => match current {
                ModListSortMode::Name => ModListSortMode::Author,
                ModListSortMode::Author => ModListSortMode::SafeMode,
                ModListSortMode::SafeMode => ModListSortMode::Toggle,
                ModListSortMode::Toggle => ModListSortMode::Name,
                ModListSortMode::Debug => ModListSortMode::Name,
            },
            Self::Screensaver | Self::Boss => match current {
                ModListSortMode::Name => ModListSortMode::Author,
                ModListSortMode::Author => ModListSortMode::Toggle,
                ModListSortMode::Toggle => ModListSortMode::Debug,
                ModListSortMode::Debug => ModListSortMode::Name,
                ModListSortMode::SafeMode => ModListSortMode::Name,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: build sorted package list
// ---------------------------------------------------------------------------

fn collect_packages<'a>(ctx: &'a UiContext, kind: ModListKind) -> Vec<PackageItem<'a>> {
    match kind {
        ModListKind::Game => ctx
            .packages
            .games
            .all()
            .iter()
            .map(PackageItem::Game)
            .collect(),
        ModListKind::Screensaver => ctx
            .packages
            .screensavers
            .all()
            .iter()
            .map(PackageItem::Overlay)
            .collect(),
        ModListKind::Boss => ctx
            .packages
            .bosses
            .all()
            .iter()
            .map(PackageItem::Overlay)
            .collect(),
    }
}

fn is_package_enabled(ctx: &UiContext, kind: ModListKind, uid: &str) -> bool {
    match kind {
        ModListKind::Game => ctx.packages.games.is_enabled(uid),
        ModListKind::Screensaver => ctx.packages.screensavers.is_enabled(uid),
        ModListKind::Boss => ctx.packages.bosses.is_enabled(uid),
    }
}

fn sort_packages(items: &mut [PackageItem], mode: ModListSortMode) {
    items.sort_by(|a, b| compare_packages(a, b, mode));
}

fn compare_packages(a: &PackageItem, b: &PackageItem, mode: ModListSortMode) -> Ordering {
    let primary = match mode {
        ModListSortMode::Name => compare_text(a.display_name(), b.display_name()),
        ModListSortMode::Author => compare_text(a.author(), b.author()),
        ModListSortMode::SafeMode => Ordering::Equal, // safe_mode state not accessible here
        ModListSortMode::Toggle => Ordering::Equal,   // enabled comparison below
        ModListSortMode::Debug => Ordering::Equal,
    };
    primary
        .then_with(|| compare_text(a.display_name(), b.display_name()))
        .then_with(|| compare_text(a.author(), b.author()))
}

fn compare_text(a: &str, b: &str) -> Ordering {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_width = UnicodeWidthStr::width(a_lower.as_str());
    let b_width = UnicodeWidthStr::width(b_lower.as_str());
    a_width.cmp(&b_width).then_with(|| a_lower.cmp(&b_lower))
}

// ---------------------------------------------------------------------------
// Icon / Banner helpers
// ---------------------------------------------------------------------------

fn icon_lines(raw: &str) -> Vec<&str> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.lines().take(4).collect()
}

fn banner_lines(raw: &str) -> Vec<&str> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.lines().collect()
}

// ---------------------------------------------------------------------------
// Render: colored header (issue 2.3)
// ---------------------------------------------------------------------------

fn sort_name_i18n(mode: ModListSortMode, ctx: &UiContext) -> &str {
    match mode {
        ModListSortMode::Name => &ctx.i18n.mod_list.info_sort_name,
        ModListSortMode::Author => &ctx.i18n.mod_list.info_sort_author,
        ModListSortMode::SafeMode => &ctx.i18n.mod_list.info_sort_safe_mode,
        ModListSortMode::Toggle => &ctx.i18n.mod_list.info_sort_toggle,
        ModListSortMode::Debug => &ctx.i18n.mod_list.info_sort_debug,
    }
}

fn draw_colored_header(
    canvas: &mut Canvas,
    ctx: &UiContext,
    layout: &SplitPanel,
    state: &ModListPageState,
    _kind: ModListKind,
) -> UiResult<()> {
    let title = &ctx.i18n.mod_list.list_title;
    let order_text = match state.sort_order {
        ModListSortOrder::Asc => &ctx.i18n.mod_list.info_order_ascending,
        ModListSortOrder::Desc => &ctx.i18n.mod_list.info_order_descending,
    };
    let sort_text = sort_name_i18n(state.sort_mode, ctx);

    let title_color = theme_color(ctx, "text.primary", "white");
    let order_color = theme_color(ctx, "state.success", "green");
    let sort_color = theme_color(ctx, "text.warning", "yellow");

    let x = layout.left_x.saturating_add(2);
    let y = layout.left_y;

    // Erase the border area behind the header text
    canvas.eraser(layout.left_x, y, layout.left_width, 1)?;

    // Draw header text
    let header_prefix = format!(" {title} *");
    canvas.draw_text_styled(
        x,
        y,
        &header_prefix,
        Some(title_color.clone()),
        None,
        vec![STYLE_BOLD],
    )?;
    let mut cursor = x + UnicodeWidthStr::width(header_prefix.as_str()) as u16;

    canvas.draw_text_styled(
        cursor,
        y,
        "[",
        Some(title_color.clone()),
        None,
        vec![STYLE_BOLD],
    )?;
    cursor += 1;

    canvas.draw_text_styled(
        cursor,
        y,
        order_text,
        Some(order_color),
        None,
        vec![STYLE_BOLD],
    )?;
    cursor += UnicodeWidthStr::width(order_text.as_str()) as u16;

    canvas.draw_text_styled(
        cursor,
        y,
        "] ",
        Some(title_color.clone()),
        None,
        vec![STYLE_BOLD],
    )?;
    cursor += 2;

    canvas.draw_text_styled(
        cursor,
        y,
        sort_text,
        Some(sort_color),
        None,
        vec![STYLE_BOLD],
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Render: full list item (issue 2.1, 2.6)
// ---------------------------------------------------------------------------

fn draw_full_item(
    canvas: &mut Canvas,
    ctx: &UiContext,
    item: &PackageItem,
    x: u16,
    y: u16,
    width: u16,
    selected: bool,
    kind: ModListKind,
) -> UiResult<()> {
    let inner_x = x.saturating_add(1);
    let inner_width = width.saturating_sub(2).max(1);
    let bg = if selected {
        Some("dark_gray".to_string())
    } else {
        None
    };
    let fg = if selected {
        theme_color(ctx, "text.primary", "white")
    } else {
        theme_color(ctx, "text.primary", "white")
    };

    // Selected background fill (4 rows)
    if selected {
        canvas.fill_rect(
            inner_x,
            y,
            inner_width,
            4,
            ' ',
            None,
            Some("dark_gray".to_string()),
        )?;
    }

    // Icon area (8 cols x 4 rows)
    let icon_str = item.icon();
    if !icon_str.is_empty() {
        let lines = icon_lines(icon_str);
        for (row, line) in lines.iter().enumerate() {
            if row >= 4 {
                break;
            }
            canvas.draw_rich_text_styled(
                inner_x,
                y.saturating_add(row as u16),
                *line,
                Some(fg.clone()),
                bg.clone(),
                Vec::new(),
            )?;
        }
    }

    let icon_width: u16 = 8;
    let text_x = inner_x.saturating_add(icon_width).saturating_add(1);

    // Package name (bold)
    canvas.draw_text_styled(
        text_x,
        y,
        item.display_name(),
        Some(fg.clone()),
        bg.clone(),
        vec![STYLE_BOLD],
    )?;

    // Author line
    let author_label = &ctx.i18n.mod_list.info_author;
    canvas.draw_text_styled(
        text_x,
        y.saturating_add(1),
        author_label,
        Some(fg.clone()),
        bg.clone(),
        Vec::new(),
    )?;
    canvas.draw_rich_text_styled(
        text_x.saturating_add(UnicodeWidthStr::width(author_label.as_str()) as u16),
        y.saturating_add(1),
        item.author(),
        Some(fg.clone()),
        bg.clone(),
        Vec::new(),
    )?;

    // Version line
    let version_label = &ctx.i18n.mod_list.info_version;
    canvas.draw_text_styled(
        text_x,
        y.saturating_add(2),
        version_label,
        Some(fg.clone()),
        bg.clone(),
        Vec::new(),
    )?;
    canvas.draw_rich_text_styled(
        text_x.saturating_add(UnicodeWidthStr::width(version_label.as_str()) as u16),
        y.saturating_add(2),
        item.version(),
        Some(fg.clone()),
        bg.clone(),
        Vec::new(),
    )?;

    // Status line
    let status_label = &ctx.i18n.mod_list.status;
    let enabled = is_package_enabled(ctx, kind, item.uid());
    let (status_text, status_color) = if enabled {
        (
            ctx.i18n.mod_list.toggle_mod_on.as_str(),
            theme_color(ctx, "state.success", "green"),
        )
    } else {
        (
            ctx.i18n.mod_list.toggle_mod_off.as_str(),
            theme_color(ctx, "state.danger", "red"),
        )
    };
    canvas.draw_text_styled(
        text_x,
        y.saturating_add(3),
        status_label,
        Some(fg.clone()),
        bg.clone(),
        Vec::new(),
    )?;
    canvas.draw_text_styled(
        text_x.saturating_add(UnicodeWidthStr::width(status_label.as_str()) as u16),
        y.saturating_add(3),
        status_text,
        Some(status_color),
        bg.clone(),
        vec![STYLE_BOLD],
    )?;

    // Safe mode off red bar (only for Game kind)
    if kind.has_safe_mode() {
        canvas.fill_rect(
            x.saturating_add(width).saturating_sub(2),
            y,
            1,
            4,
            ' ',
            None,
            Some("red".to_string()),
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Render: brief list item (issue 2.2)
// ---------------------------------------------------------------------------

fn draw_brief_item(
    canvas: &mut Canvas,
    ctx: &UiContext,
    item: &PackageItem,
    x: u16,
    y: u16,
    width: u16,
    selected: bool,
    kind: ModListKind,
) -> UiResult<()> {
    let inner_x = x.saturating_add(1);
    let inner_width = width.saturating_sub(2).max(1);
    let bg = if selected {
        Some("dark_gray".to_string())
    } else {
        None
    };
    let fg = if selected {
        theme_color(ctx, "text.primary", "white")
    } else {
        theme_color(ctx, "text.primary", "white")
    };

    if selected {
        canvas.fill_rect(
            inner_x,
            y,
            inner_width,
            1,
            ' ',
            None,
            Some("dark_gray".to_string()),
        )?;
    }

    let cursor = inner_x.saturating_add(1);

    // Enabled status badge on the right
    let enabled = is_package_enabled(ctx, kind, item.uid());
    let (status_text, status_color) = if enabled {
        (
            ctx.i18n.mod_list.toggle_mod_on_brief.as_str(),
            theme_color(ctx, "state.success", "green"),
        )
    } else {
        (
            ctx.i18n.mod_list.toggle_mod_off_brief.as_str(),
            theme_color(ctx, "state.danger", "red"),
        )
    };
    let status_full = format!("[{status_text}]");
    let status_width = UnicodeWidthStr::width(status_full.as_str()) as u16;
    let bar_space: u16 = if kind.has_safe_mode() { 2 } else { 0 };
    let status_x = x
        .saturating_add(width)
        .saturating_sub(status_width)
        .saturating_sub(bar_space)
        .saturating_sub(2);

    // Package name (cropped)
    let _max_name_width = status_x.saturating_sub(cursor).saturating_sub(1).max(1);
    canvas.draw_text_styled(
        cursor,
        y,
        item.display_name(),
        Some(fg.clone()),
        bg.clone(),
        vec![STYLE_BOLD],
    )?;

    // Status badge
    canvas.draw_text_styled(status_x, y, "[", Some(fg.clone()), bg.clone(), Vec::new())?;
    canvas.draw_text_styled(
        status_x.saturating_add(1),
        y,
        status_text,
        Some(status_color),
        bg.clone(),
        vec![STYLE_BOLD],
    )?;
    canvas.draw_text_styled(
        status_x
            .saturating_add(1)
            .saturating_add(UnicodeWidthStr::width(status_text) as u16),
        y,
        "]",
        Some(fg),
        bg.clone(),
        Vec::new(),
    )?;

    // Safe mode off red bar
    if kind.has_safe_mode() {
        canvas.fill_rect(
            x.saturating_add(width).saturating_sub(2),
            y,
            1,
            1,
            ' ',
            None,
            Some("red".to_string()),
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Render: page line (issues 2.4, 2.5)
// ---------------------------------------------------------------------------

fn draw_page_line(
    canvas: &mut Canvas,
    ctx: &UiContext,
    layout: &SplitPanel,
    state: &ModListPageState,
    total_pages: usize,
) -> UiResult<()> {
    let y = layout
        .left_y
        .saturating_add(layout.height)
        .saturating_sub(2);
    let page_color = theme_color(ctx, "text.muted", "dark_gray");

    if state.jump_mode {
        let current = if state.jump_input == 0 {
            "_".to_string()
        } else {
            state.jump_input.to_string()
        };
        let page_text = format!("{current}/{total_pages}");
        let page_x = layout.left_x.saturating_add(
            (layout
                .left_width
                .saturating_sub(UnicodeWidthStr::width(page_text.as_str()) as u16))
                / 2,
        );
        canvas.draw_text_styled(
            page_x,
            y,
            &current,
            Some("black".to_string()),
            Some("yellow".to_string()),
            vec![STYLE_BOLD],
        )?;
        canvas.draw_text_styled(
            page_x.saturating_add(UnicodeWidthStr::width(current.as_str()) as u16),
            y,
            &format!("/{total_pages}"),
            Some(page_color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
    } else {
        let page_text = format!("{}/{}", state.page, total_pages);
        let page_x = layout.left_x.saturating_add(
            (layout
                .left_width
                .saturating_sub(UnicodeWidthStr::width(page_text.as_str()) as u16))
                / 2,
        );
        canvas.draw_text_styled(
            page_x,
            y,
            &page_text,
            Some(page_color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
    }

    // Left arrow + prev_page key
    if state.page > 1 {
        let prev_key = key_hint(ctx, "prev_page", "<");
        let left_text = format!("◀ [{prev_key}]");
        canvas.draw_text_styled(
            layout.left_x.saturating_add(2),
            y,
            &left_text,
            Some(page_color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
    }

    // Right arrow + next_page key
    if state.page < total_pages {
        let next_key = key_hint(ctx, "next_page", ">");
        let right_text = format!("[{next_key}] ▶");
        let right_width = UnicodeWidthStr::width(right_text.as_str()) as u16;
        canvas.draw_text_styled(
            layout
                .left_x
                .saturating_add(layout.left_width)
                .saturating_sub(right_width)
                .saturating_sub(2),
            y,
            &right_text,
            Some(page_color),
            None,
            vec![STYLE_BOLD],
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Render: info panel (issues 2.7, 2.10)
// ---------------------------------------------------------------------------

fn draw_info_panel(
    canvas: &mut Canvas,
    ctx: &UiContext,
    panel: &SplitPanel,
    item: &PackageItem,
    _state: &ModListPageState,
    kind: ModListKind,
) -> UiResult<()> {
    let label_color = theme_color(ctx, "text.warning", "yellow");
    let text_color = theme_color(ctx, "text.primary", "white");
    let muted_color = theme_color(ctx, "text.muted", "dark_gray");

    let content_x = panel.right_x.saturating_add(1);
    let content_width = panel.right_width.saturating_sub(2).max(1);
    let max_y = panel.right_y.saturating_add(panel.height).saturating_sub(1);
    let mut y = panel.right_y.saturating_add(1);

    let draw_line =
        |canvas: &mut Canvas, y: u16, text: &str, color: &str, bold: bool| -> UiResult<u16> {
            let _ = content_width; // used implicitly via content_x positioning
            if y >= max_y {
                return Ok(y);
            }
            let styles = if bold { vec![STYLE_BOLD] } else { Vec::new() };
            canvas.draw_text_styled(content_x, y, text, Some(color.to_string()), None, styles)?;
            Ok(y.saturating_add(1))
        };

    // Banner area (max 13 lines)
    let banner_str = item.banner();
    if !banner_str.is_empty() {
        let lines = banner_lines(banner_str);
        let banner_max = 13usize.min(lines.len());
        let mut banner_drawn = 0usize;
        for line in lines.iter().take(banner_max) {
            if y >= max_y {
                break;
            }
            let line_width = UnicodeWidthStr::width(*line) as u16;
            let pad = if line_width < content_width {
                (content_width.saturating_sub(line_width)) / 2
            } else {
                0
            };
            let padded = format!("{}{}", " ".repeat(pad as usize), line);
            canvas.draw_rich_text_styled(
                content_x,
                y,
                &padded,
                Some(text_color.clone()),
                None,
                Vec::new(),
            )?;
            y = y.saturating_add(1);
            banner_drawn += 1;
        }
        // Pad banner to 13 lines
        let mut add_top = true;
        while banner_drawn > 0 && banner_drawn < 13 && y < max_y {
            add_top = !add_top;
            banner_drawn += 1;
        }
        y = draw_line(canvas, y, "", &text_color, false)?; // blank separator
    }

    // Basic Info section
    y = draw_line(canvas, y, &ctx.i18n.mod_list.info_base, &label_color, true)?;
    y = draw_line(canvas, y, item.display_name(), &text_color, false)?;
    let author_line = format!("{}{}", ctx.i18n.mod_list.info_author, item.author());
    y = draw_line(canvas, y, &author_line, &text_color, false)?;
    let version_line = format!("{}{}", ctx.i18n.mod_list.info_version, item.version());
    y = draw_line(canvas, y, &version_line, &text_color, false)?;
    y = draw_line(canvas, y, "", &text_color, false)?;

    // Security Info section
    y = draw_line(canvas, y, &ctx.i18n.mod_list.info_safe, &label_color, true)?;

    let enabled = is_package_enabled(ctx, kind, item.uid());
    let (en_status, en_color) = if enabled {
        (
            ctx.i18n.mod_list.toggle_mod_on.as_str(),
            theme_color(ctx, "state.success", "green"),
        )
    } else {
        (
            ctx.i18n.mod_list.toggle_mod_off.as_str(),
            theme_color(ctx, "state.danger", "red"),
        )
    };
    let en_label_w = UnicodeWidthStr::width(ctx.i18n.mod_list.info_safe_switch.as_str()) as u16;
    canvas.draw_text_styled(
        content_x,
        y,
        &ctx.i18n.mod_list.info_safe_switch,
        Some(text_color.clone()),
        None,
        Vec::new(),
    )?;
    canvas.draw_text_styled(
        content_x.saturating_add(en_label_w),
        y,
        en_status,
        Some(en_color),
        None,
        vec![STYLE_BOLD],
    )?;
    y = y.saturating_add(1);

    // Debug line (state from game_state not accessible — default false)
    let dbg_label_w = UnicodeWidthStr::width(ctx.i18n.mod_list.info_safe_debug.as_str()) as u16;
    canvas.draw_text_styled(
        content_x,
        y,
        &ctx.i18n.mod_list.info_safe_debug,
        Some(text_color.clone()),
        None,
        Vec::new(),
    )?;
    canvas.draw_text_styled(
        content_x.saturating_add(dbg_label_w),
        y,
        &ctx.i18n.mod_list.toggle_debug_off,
        Some(muted_color.clone()),
        None,
        Vec::new(),
    )?;
    y = y.saturating_add(1);

    // Write permission (Game only)
    if kind.has_safe_mode() {
        let write_text = if item.write_permission() {
            ctx.i18n.mod_list.toggle_write_on.as_str()
        } else {
            ctx.i18n.mod_list.toggle_write_off.as_str()
        };
        let write_color = if item.write_permission() {
            theme_color(ctx, "state.danger", "red")
        } else {
            muted_color.clone()
        };
        let write_label_w =
            UnicodeWidthStr::width(ctx.i18n.mod_list.info_safe_write.as_str()) as u16;
        canvas.draw_text_styled(
            content_x,
            y,
            &ctx.i18n.mod_list.info_safe_write,
            Some(text_color.clone()),
            None,
            Vec::new(),
        )?;
        canvas.draw_text_styled(
            content_x.saturating_add(write_label_w),
            y,
            write_text,
            Some(write_color),
            None,
            Vec::new(),
        )?;
        y = y.saturating_add(1);

        // Safe Mode line
        let safe_text = &ctx.i18n.mod_list.toggle_safe_mode_on;
        let safe_color = theme_color(ctx, "state.success", "green");
        let safe_label_w =
            UnicodeWidthStr::width(ctx.i18n.mod_list.info_safe_safe_mode.as_str()) as u16;
        canvas.draw_text_styled(
            content_x,
            y,
            &ctx.i18n.mod_list.info_safe_safe_mode,
            Some(text_color.clone()),
            None,
            Vec::new(),
        )?;
        canvas.draw_text_styled(
            content_x.saturating_add(safe_label_w),
            y,
            safe_text,
            Some(safe_color),
            None,
            vec![STYLE_BOLD],
        )?;
        y = y.saturating_add(1);
    }

    y = draw_line(canvas, y, "", &text_color, false)?;

    // Introduction section
    y = draw_line(
        canvas,
        y,
        &ctx.i18n.mod_list.info_introduction,
        &label_color,
        true,
    )?;
    let intro = item.introduction();
    if !intro.is_empty() {
        for line in intro.lines() {
            if y >= max_y {
                break;
            }
            canvas.draw_rich_text_styled(
                content_x,
                y,
                line,
                Some(text_color.clone()),
                None,
                Vec::new(),
            )?;
            y = y.saturating_add(1);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Render: dynamic action line (issue 2.9)
// ---------------------------------------------------------------------------

fn build_action_segments(
    ctx: &UiContext,
    state: &ModListPageState,
    kind: ModListKind,
    total_pages: usize,
) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();

    if state.jump_mode {
        segments.push(format!("[1]-[9] {}", ctx.i18n.key.mod_list_select));
        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.mod_list_confirm
        ));
        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.mod_list_cancel
        ));
    } else {
        let up_key = key_hint(ctx, "prev_option", "↑");
        let down_key = key_hint(ctx, "next_option", "↓");
        segments.push(format!(
            "[{up_key}]/[{down_key}] {}",
            ctx.i18n.key.mod_list_select
        ));

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.mod_list_toggle_confirm
        ));

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "debug", "D"),
            ctx.i18n.key.mod_list_debug
        ));

        if kind.has_safe_mode() {
            segments.push(format!(
                "[{}] {}",
                key_hint(ctx, "safe_mode", "R"),
                ctx.i18n.key.mod_list_safe_mode
            ));
        }

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "list", "L"),
            ctx.i18n.key.mod_list_list
        ));

        let scroll_up = key_hint(ctx, "scroll_up", "W");
        let scroll_down = key_hint(ctx, "scroll_down", "S");
        segments.push(format!(
            "[{scroll_up}]/[{scroll_down}] {}",
            ctx.i18n.key.mod_list_scroll
        ));

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "order", "O"),
            ctx.i18n.key.mod_list_order
        ));

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "sort", "T"),
            ctx.i18n.key.mod_list_sort
        ));

        if total_pages > 1 {
            segments.push(format!(
                "[{}] {}",
                key_hint(ctx, "jump", "J"),
                ctx.i18n.key.mod_list_jump
            ));
            let prev_pg = key_hint(ctx, "prev_page", "<");
            let next_pg = key_hint(ctx, "next_page", ">");
            segments.push(format!(
                "[{prev_pg}]/[{next_pg}] {}",
                ctx.i18n.key.mod_list_flip
            ));
        }

        segments.push(format!(
            "[{}] {}",
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.mod_list_back
        ));
    }

    segments
}

fn wrap_action_lines(segments: &[String], term_width: usize) -> Vec<String> {
    let wrap_width = term_width.saturating_sub(2).max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    let sep = "  ";

    for seg in segments {
        if let Some(ref cur) = current {
            let candidate = format!("{cur}{sep}{seg}");
            if UnicodeWidthStr::width(candidate.as_str()) <= wrap_width {
                current = Some(candidate);
            } else {
                lines.push(cur.clone());
                current = Some(seg.clone());
            }
        } else {
            current = Some(seg.clone());
        }
    }
    if let Some(cur) = current {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn draw_action_line(
    canvas: &mut Canvas,
    ctx: &UiContext,
    state: &ModListPageState,
    kind: ModListKind,
    total_pages: usize,
) -> UiResult<usize> {
    let hint_color = theme_color(ctx, "text.muted", "dark_gray");
    let segments = build_action_segments(ctx, state, kind, total_pages);
    let lines = wrap_action_lines(&segments, ctx.terminal_size.width as usize);

    let terminal_height = ctx.terminal_size.height;
    let base_y = terminal_height.saturating_sub(lines.len() as u16);

    for (i, line) in lines.iter().enumerate() {
        let line_width =
            UnicodeWidthStr::width(line.as_str()).min(ctx.terminal_size.width as usize - 2) as u16;
        let x = (ctx.terminal_size.width.saturating_sub(line_width)) / 2;
        canvas.draw_text_styled(
            x,
            base_y.saturating_add(i as u16),
            line,
            Some(hint_color.clone()),
            None,
            Vec::new(),
        )?;
    }

    Ok(lines.len())
}

// ---------------------------------------------------------------------------
// Main render entry
// ---------------------------------------------------------------------------

fn render_mod_list_page(
    canvas: &mut Canvas,
    ctx: &UiContext,
    state: &ModListPageState,
    kind: ModListKind,
) -> UiResult<()> {
    canvas.clear()?;

    let mut items = collect_packages(ctx, kind);
    sort_packages(&mut items, state.sort_mode);
    if state.sort_order == ModListSortOrder::Desc {
        items.reverse();
    }

    // Compute footer line count dynamically from action segments
    let footer_lines: u16 = {
        let dummy_pages = ((items.len() + 7) / 8).max(1);
        let segments = build_action_segments(ctx, state, kind, dummy_pages);
        let lines = wrap_action_lines(&segments, ctx.terminal_size.width as usize);
        lines.len().max(1) as u16
    };

    let panel = SplitPanel::new(
        ctx.terminal_size.width,
        ctx.terminal_size.height,
        footer_lines,
    );
    let capacity = {
        let item_h: u16 = match state.list_mode {
            ModListDisplayMode::Full => 4,
            ModListDisplayMode::Brief => 1,
        };
        (panel.height.saturating_sub(4).saturating_sub(1) / item_h) as usize
    };
    let actual_pages = if capacity == 0 {
        1
    } else {
        ((items.len() + capacity - 1) / capacity).max(1)
    };

    // Normalize page
    let page = state.page.min(actual_pages).max(1);
    let start_idx = (page - 1) * capacity;
    let end_idx = (start_idx + capacity).min(items.len());

    // Render panels — left title is drawn by draw_colored_header instead
    let empty_title = "";
    let info_title = &ctx.i18n.mod_list.info_title;
    panel.render_borders_with_theme(canvas, ctx, empty_title, info_title)?;

    // Colored header
    let adjusted_state = ModListPageState {
        page,
        ..ModListPageState {
            selected_index: state.selected_index,
            page: state.page,
            sort_mode: state.sort_mode,
            sort_order: state.sort_order,
            list_mode: state.list_mode,
            jump_mode: state.jump_mode,
            jump_input: state.jump_input,
            info_scroll: state.info_scroll,
        }
    };
    draw_colored_header(canvas, ctx, &panel, &adjusted_state, kind)?;

    // List items
    let list_y = panel.left_y.saturating_add(1);
    if items.is_empty() {
        let empty_text = &ctx.i18n.mod_list.none_mod;
        let empty_w = UnicodeWidthStr::width(empty_text.as_str()) as u16;
        let ex = panel
            .left_x
            .saturating_add((panel.left_width.saturating_sub(empty_w)) / 2);
        let ey = panel.left_y.saturating_add(panel.height / 2);
        canvas.draw_text_styled(
            ex,
            ey,
            empty_text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
    } else {
        for (i, item) in items
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(end_idx - start_idx)
        {
            let row = i - start_idx;
            let item_h: u16 = match state.list_mode {
                ModListDisplayMode::Full => 4,
                ModListDisplayMode::Brief => 1,
            };
            let item_y = list_y.saturating_add(row as u16 * item_h);
            let selected = i == state.selected_index;
            match state.list_mode {
                ModListDisplayMode::Full => {
                    draw_full_item(
                        canvas,
                        ctx,
                        item,
                        panel.left_x,
                        item_y,
                        panel.left_width,
                        selected,
                        kind,
                    )?;
                }
                ModListDisplayMode::Brief => {
                    draw_brief_item(
                        canvas,
                        ctx,
                        item,
                        panel.left_x,
                        item_y,
                        panel.left_width,
                        selected,
                        kind,
                    )?;
                }
            }
        }
    }

    // Page line
    draw_page_line(canvas, ctx, &panel, &adjusted_state, actual_pages)?;

    // Info panel
    if let Some(item) = items.get(state.selected_index) {
        draw_info_panel(canvas, ctx, &panel, item, &adjusted_state, kind)?;
    }

    // Action line
    draw_action_line(canvas, ctx, &adjusted_state, kind, actual_pages)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Event handling
// ---------------------------------------------------------------------------

fn handle_mod_list_event(
    state: &mut ModListPageState,
    event: &UiEvent,
    ctx: &UiContext,
    kind: ModListKind,
    pending_navigation: &mut Option<UiNavigation>,
) -> UiResult<()> {
    let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
        return Ok(());
    };
    if !is_press(status) {
        return Ok(());
    }

    let items = collect_packages(ctx, kind);

    // Jump mode key handling
    if state.jump_mode {
        match name.as_str() {
            "confirm" | "enter" => {
                if state.jump_input >= 1 {
                    state.page = state.jump_input;
                }
                state.jump_mode = false;
                state.jump_input = 0;
                return Ok(());
            }
            "back" | "return" | "esc" | "q" => {
                state.jump_mode = false;
                state.jump_input = 0;
                return Ok(());
            }
            _ => {}
        }
        // Digit input
        if let Ok(digit) = name.parse::<usize>() {
            if (0..=9).contains(&digit) {
                state.jump_input = (state.jump_input * 10 + digit).min(9999);
            }
            return Ok(());
        }
        if name == "backspace" {
            state.jump_input /= 10;
            return Ok(());
        }
        return Ok(());
    }

    // Normal mode
    match name.as_str() {
        "prev_option" | "up" | "arrowup" => {
            if !items.is_empty() {
                state.selected_index = if state.selected_index == 0 {
                    items.len() - 1
                } else {
                    state.selected_index - 1
                };
            }
        }
        "next_option" | "down" | "arrowdown" => {
            if !items.is_empty() {
                state.selected_index = if state.selected_index + 1 >= items.len() {
                    0
                } else {
                    state.selected_index + 1
                };
            }
        }
        "prev_page" | "page_up" => {
            if state.page > 1 {
                state.page -= 1;
                state.selected_index = (state.page - 1) * 8; // approximate, normalized in render
            }
        }
        "next_page" | "page_down" => {
            state.page += 1;
            state.selected_index = (state.page - 1) * 8;
        }
        "scroll_up" | "w" => {
            state.info_scroll = state.info_scroll.saturating_sub(1);
        }
        "scroll_down" | "s" => {
            state.info_scroll = state.info_scroll.saturating_add(1);
        }
        "jump" => {
            state.jump_mode = true;
            state.jump_input = 0;
        }
        "confirm" | "enter" => {
            // Toggle enabled — action queued for upper layer
        }
        "debug" => {
            // Toggle debug — action queued for upper layer
        }
        "safe_mode" if kind.has_safe_mode() => {
            // Toggle safe_mode — action queued for upper layer
        }
        "list" | "l" => {
            state.list_mode = match state.list_mode {
                ModListDisplayMode::Full => ModListDisplayMode::Brief,
                ModListDisplayMode::Brief => ModListDisplayMode::Full,
            };
        }
        "order" => {
            state.sort_order = match state.sort_order {
                ModListSortOrder::Asc => ModListSortOrder::Desc,
                ModListSortOrder::Desc => ModListSortOrder::Asc,
            };
        }
        "sort" => {
            state.sort_mode = kind.sort_cycle(state.sort_mode);
        }
        "back" | "return" | "esc" | "q" => {
            *pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingMods));
        }
        _ => {}
    }

    state.normalize(&items);
    Ok(())
}

// ---------------------------------------------------------------------------
// Macro: generate page struct + UiPage impl
// ---------------------------------------------------------------------------

macro_rules! mod_page {
    ($name:ident, $key:expr, $kind:expr) => {
        pub struct $name {
            state: ModListPageState,
            pending_navigation: Option<UiNavigation>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    state: ModListPageState::new(),
                    pending_navigation: None,
                }
            }
        }

        impl UiPage for $name {
            fn page_key(&self) -> UiPageKey {
                $key
            }

            fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
                handle_mod_list_event(
                    &mut self.state,
                    event,
                    ctx,
                    $kind,
                    &mut self.pending_navigation,
                )
            }

            fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
                render_mod_list_page(canvas, ctx, &self.state, $kind)
            }

            fn take_navigation(&mut self) -> Option<UiNavigation> {
                take_navigation(&mut self.pending_navigation)
            }
        }
    };
}

mod_page!(ModGameListPage, UiPageKey::ModGameList, ModListKind::Game);
mod_page!(
    ModScreensaverListPage,
    UiPageKey::ModScreensaverList,
    ModListKind::Screensaver
);
mod_page!(ModBossListPage, UiPageKey::ModBossList, ModListKind::Boss);
