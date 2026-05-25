//! Rust implementation of storage details page.

use std::fs;
use std::path::{Path, PathBuf};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::{
    draw_title, is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

pub struct StorageDetailsPage {
    pending_navigation: Option<UiNavigation>,
}

impl StorageDetailsPage {
    pub fn new() -> Self {
        Self {
            pending_navigation: None,
        }
    }
}

impl UiPage for StorageDetailsPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::StorageDetails
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }
        match name.as_str() {
            "back" | "return" | "esc" | "q" => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingMemory));
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        draw_title(canvas, ctx, &ctx.i18n.memory.show)?;
        render_table(canvas, ctx)?;
        render_tip(canvas, ctx)?;
        render_footer(canvas, ctx)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

fn render_table(canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
    let rows = storage_rows(ctx);
    let headers = [
        ctx.i18n.memory.info_dir.as_str(),
        ctx.i18n.memory.info_size.as_str(),
        ctx.i18n.memory.info_path.as_str(),
    ];
    let dir_width = rows
        .iter()
        .map(|row| UnicodeWidthStr::width(row.name.as_str()))
        .chain(std::iter::once(UnicodeWidthStr::width(headers[0])))
        .max()
        .unwrap_or(0) as u16
        + 6;
    let size_width = rows
        .iter()
        .map(|row| UnicodeWidthStr::width(row.size.as_str()))
        .chain(std::iter::once(UnicodeWidthStr::width(headers[1])))
        .max()
        .unwrap_or(0) as u16
        + 6;
    let path_width = rows
        .iter()
        .map(|row| UnicodeWidthStr::width(row.path.as_str()))
        .chain(std::iter::once(UnicodeWidthStr::width(headers[2])))
        .max()
        .unwrap_or(0) as u16;
    let table_width = dir_width
        .saturating_add(size_width)
        .saturating_add(path_width);
    let x = ctx.terminal_size.width.saturating_sub(table_width) / 2;
    let y = ctx.terminal_size.height.saturating_sub(8) / 2;

    draw_cell(
        canvas,
        x,
        y,
        headers[0],
        theme_color(ctx, "text.warning", "yellow"),
    )?;
    draw_cell(
        canvas,
        x.saturating_add(dir_width),
        y,
        headers[1],
        theme_color(ctx, "text.warning", "yellow"),
    )?;
    draw_cell(
        canvas,
        x.saturating_add(dir_width).saturating_add(size_width),
        y,
        headers[2],
        theme_color(ctx, "text.warning", "yellow"),
    )?;

    for (index, row) in rows.iter().enumerate() {
        let row_y = y.saturating_add(2 + index as u16);
        draw_cell(
            canvas,
            x,
            row_y,
            &row.name,
            theme_color(ctx, "text.primary", "white"),
        )?;
        draw_cell(
            canvas,
            x.saturating_add(dir_width),
            row_y,
            &row.size,
            theme_color(ctx, "text.primary", "white"),
        )?;
        draw_cell(
            canvas,
            x.saturating_add(dir_width).saturating_add(size_width),
            row_y,
            &row.path,
            theme_color(ctx, "text.muted", "dark_gray"),
        )?;
    }
    Ok(())
}

fn draw_cell(canvas: &mut Canvas, x: u16, y: u16, text: &str, fg: String) -> UiResult<()> {
    canvas.draw_text_styled(x, y, text, Some(fg), None, vec![STYLE_BOLD])
}

fn render_tip(canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
    let width = UnicodeWidthStr::width(ctx.i18n.memory.tip.as_str()) as u16;
    let x = ctx.terminal_size.width.saturating_sub(width) / 2;
    canvas.draw_text_styled(
        x,
        ctx.terminal_size.height.saturating_sub(2),
        &ctx.i18n.memory.tip,
        Some(theme_color(ctx, "text.muted", "dark_gray")),
        None,
        Vec::new(),
    )
}

fn render_footer(canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
    let footer = format!(
        "[{}] {}",
        key_hint(ctx, "back", "Esc"),
        ctx.i18n.key.storage_details_back
    );
    let width = UnicodeWidthStr::width(footer.as_str()) as u16;
    let x = ctx.terminal_size.width.saturating_sub(width) / 2;
    canvas.draw_text_styled(
        x,
        ctx.terminal_size.height.saturating_sub(1),
        footer,
        Some(theme_color(ctx, "text.muted", "dark_gray")),
        None,
        Vec::new(),
    )
}

#[derive(Clone, Debug)]
struct StorageRow {
    name: String,
    size: String,
    path: String,
}

fn storage_rows(ctx: &UiContext) -> Vec<StorageRow> {
    let root = data_dirs::root_dir();
    let data = root.join("data");
    let profiles = data.join("profiles");
    let cache = data.join("cache");
    let log = data.join("log");
    let mod_dir = data.join("mod");
    vec![
        storage_row(ctx.i18n.memory.info_name_root.clone(), root),
        storage_row(ctx.i18n.memory.info_name_data.clone(), data),
        storage_row(ctx.i18n.memory.info_name_cache.clone(), cache),
        storage_row(ctx.i18n.memory.info_name_profiles.clone(), profiles),
        storage_row(ctx.i18n.memory.info_name_log.clone(), log),
        storage_row(ctx.i18n.memory.info_name_mod.clone(), mod_dir),
    ]
}

fn storage_row(name: String, path: PathBuf) -> StorageRow {
    StorageRow {
        name,
        size: format_size(directory_size(&path)),
        path: path.display().to_string(),
    }
}

fn directory_size(path: &Path) -> u64 {
    if path.is_file() {
        return fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
    }
    if !path.is_dir() {
        return 0;
    }
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| directory_size(&entry.path()))
        .sum()
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = size as f64;
    let mut unit_index = 0;
    while value >= 1024.0 && unit_index + 1 < UNITS.len() {
        value /= 1024.0;
        unit_index += 1;
    }
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{value:.2} {}", UNITS[unit_index])
    }
}
