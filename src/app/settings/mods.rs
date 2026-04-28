// Mod 管理页面，设置页面中最复杂的子页。提供双栏布局（左列表右详情）、Mod 启用/禁用、安全模式管理、排序、热重载、缩略图渲染、详情面板和 Mod 安全模式对话框

use std::time::Instant; // 安全模式倒计时

use ratatui::buffer::Buffer; // 缓冲区渲染
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect}; // 布局
use ratatui::style::{Color, Modifier, Style}; // 样式
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::{Block, Borders, Paragraph, Wrap}; // 组件

use crate::app::i18n; // 国际化
use crate::app::rich_text; // 富文本解析（Mod 介绍）
use crate::app::settings::common as settings_common; // 通用工具
use crate::app::settings::types::*; // 类型定义
use crate::mods::{self, ModPackage, ModSafeModeState}; // Mod 系统类型和函数

// 计算最小尺寸 (90, 25+提示行数)
pub fn minimum_size_mods() -> (u16, u16) {
    let min_width = 90u16;
    let hint_lines =
        settings_common::wrap_mod_hint_lines(&settings_common::build_mod_hint_segments(true), min_width.saturating_sub(2) as usize)
            .len()
            .max(1) as u16;
    (min_width, 25 + hint_lines)
}

// 	主渲染：验证分页 → 计算预览/提示布局 → 左列表 + 右详情 → 底部提示 → 安全模式对话框
pub fn render_mods(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if state.mod_packages.is_empty() {
        state.mod_selected = 0;
        state.mod_page = 0;
        state.mod_detail_scroll = 0;
        state.mod_detail_scroll_available = false;
    } else {
        let page_size = settings_common::current_mod_page_size(state.mod_list_view);
        let total_pages = settings_common::total_mod_pages(state.mod_packages.len(), page_size);
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
    let root_preview = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let columns_preview = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root_preview[0]);
    state.mod_detail_scroll_available = compute_mod_detail_scroll_available(columns_preview[1], state);

    let hint_lines = settings_common::wrap_mod_hint_lines(
        &settings_common::build_mod_hint_segments(state.mod_detail_scroll_available),
        area.width.max(1) as usize,
    );
    let hint_height = hint_lines.len().max(1).min(u16::MAX as usize) as u16;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(hint_height)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root[0]);

    render_mod_list(frame, columns[0], state);
    render_mod_detail(frame, columns[1], state);

    frame.render_widget(
        Paragraph::new(hint_lines)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        root[1],
    );
    if let Some(dialog) = &state.mod_safe_dialog {
        render_mod_safe_dialog(frame, dialog);
    }
}

// 主按键处理：安全模式对话框模式 / 跳页输入模式 / 普通操作模式（上下选择、W/S滚动、Q/E翻页、Enter/空格切换启用、D调试、R安全模式、H热重载、L切换视图、Z排序模式、X升降序、P跳页、Esc退出）
fn compute_mod_detail_scroll_available(area: Rect, state: &SettingsState) -> bool {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", settings_common::text("settings.mods.detail", "Mod Details")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);

    let Some(package) = state.mod_packages.get(state.mod_selected) else {
        return false;
    };

    let build_detail_lines = |content_width: usize| {
        let mut lines = settings_common::rich_lines_from_image(
            &package.banner,
            content_width,
            Style::default().fg(Color::White),
        );
        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text(
            "settings.mods.section.basic_info",
            "Basic Info",
        )));
        lines.extend(settings_common::label_value_lines(
            settings_common::text("settings.mods.package_label", "Mod Package:"),
            package.package_name.clone(),
            package.package_name_allows_rich,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(settings_common::label_value_lines(
            settings_common::text("settings.mods.author", "Author:"),
            package.author.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(settings_common::label_value_lines(
            settings_common::text("settings.mods.version", "Version:"),
            package.version.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text(
            "settings.mods.section.storage",
            "Data Storage",
        )));
        lines.push(settings_common::label_value_line(
            settings_common::text("settings.mods.best_score", "Best Score:"),
            if package.has_best_score_storage {
                settings_common::text("settings.mods.storage_has", "Available")
            } else {
                settings_common::text("settings.mods.storage_none", "None")
            },
            if package.has_best_score_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(settings_common::label_value_line(
            settings_common::text("settings.mods.save_data", "Game Save:"),
            if package.has_save_storage {
                settings_common::text("settings.mods.storage_has", "Available")
            } else {
                settings_common::text("settings.mods.storage_none", "None")
            },
            if package.has_save_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text(
            "settings.mods.section.security",
            "Security",
        )));
        lines.push(settings_common::label_value_line(
            settings_common::text("settings.mods.write_request", "Direct Write Request:"),
            if package.has_write_request {
                settings_common::text("settings.mods.storage_has", "Available")
            } else {
                settings_common::text("settings.mods.storage_none", "None")
            },
            if package.has_write_request {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        let safe_mode_text = match package.safe_mode_state {
            ModSafeModeState::Enabled => settings_common::text("settings.mods.safe_mode_on", "On"),
            ModSafeModeState::DisabledSession => settings_common::text(
                "settings.mods.safe_mode_session_off",
                "Disabled (This Session)",
            ),
            ModSafeModeState::DisabledTrusted => settings_common::text(
                "settings.mods.safe_mode_trusted_off",
                "Disabled (Permanently Trusted)",
            ),
        };
        lines.push(settings_common::label_value_line(
            settings_common::text("settings.mods.safe_mode", "Safe Mode:"),
            safe_mode_text,
            if package.safe_mode_enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text(
            "settings.mods.section.introduction",
            "Mod Introduction",
        )));
        lines.extend(rich_text::parse_rich_text_wrapped(
            &package.introduction,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines
    };

    let viewport_h = inner.height as usize;
    let full_width = inner.width.max(1) as usize;
    let wide_lines = build_detail_lines(full_width);
    let needs_scroll = wide_lines.len() > viewport_h;
    let lines = if needs_scroll && inner.width > 2 {
        build_detail_lines(inner.width.saturating_sub(2).max(1) as usize)
    } else {
        wide_lines
    };
    lines.len().saturating_sub(viewport_h) > 0
}

// 预计算详情面板是否需要滚动条
fn render_mod_list(frame: &mut ratatui::Frame<'_>, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(settings_common::mod_list_title(state))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if state.mod_packages.is_empty() {
        frame.render_widget(
            Paragraph::new(settings_common::text("settings.mods.empty", "No mods found."))
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

    let item_height = settings_common::mod_item_height(state.mod_list_view);
    let page_size = ((rows[0].height / item_height).max(1)) as usize;
    let total_pages = settings_common::total_mod_pages(state.mod_packages.len(), page_size);
    let page = state.mod_page.min(total_pages.saturating_sub(1));
    let start = page * page_size;

    for (index, package) in state.mod_packages.iter().enumerate().skip(start).take(page_size) {
        let local = (index - start) as u16;
        let item_area = Rect::new(
            rows[0].x,
            rows[0].y + local * item_height,
            rows[0].width,
            item_height.min(rows[0].height.saturating_sub(local * item_height)),
        );
        render_mod_list_item(frame.buffer_mut(), item_area, package, index == state.mod_selected, state.mod_list_view);
    }

    let left = if page > 0 { i18n::t("game_selection.pager.prev") } else { String::new() };
    let right = if page + 1 < total_pages { i18n::t("game_selection.pager.next") } else { String::new() };
    let pager_line = if let Some(dialog) = &state.mod_page_jump_dialog {
        let input_text = if dialog.input.is_empty() { "_".to_string() } else { dialog.input.clone() };
        let input_style = Style::default()
            .fg(if dialog.input.is_empty() { Color::Yellow } else { Color::Black })
            .bg(Color::Yellow);
        Line::from(vec![
            Span::styled(input_text, input_style),
            Span::styled(format!("/{}", total_pages.max(1)), Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::styled(format!("{}/{}", page + 1, total_pages.max(1)), Style::default().fg(Color::White)))
    };

    frame.render_widget(Paragraph::new(left).style(Style::default().fg(Color::White)).alignment(Alignment::Left), rows[1]);
    frame.render_widget(Paragraph::new(pager_line).alignment(Alignment::Center), rows[1]);
    frame.render_widget(Paragraph::new(right).style(Style::default().fg(Color::White)).alignment(Alignment::Right), rows[1]);
}

// 渲染左栏 Mod 列表：分页、详细/简单视图、MOD徽章、缩略图、状态行、安全模式标记
fn render_mod_list_item(buffer: &mut Buffer, area: Rect, package: &ModPackage, selected: bool, list_view: ModListView) {
    if area.height == 0 || area.width == 0 { return; }

    let base_style = if selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
    let meta_style = if selected { Style::default().fg(Color::White).bg(Color::DarkGray) } else { Style::default().fg(Color::Gray) };

    for dy in 0..area.height {
        buffer.set_string(area.x, area.y + dy, " ".repeat(area.width as usize), Style::default());
    }
    let highlight_rows = match list_view {
        ModListView::Detailed if selected => area.height.min(4),
        ModListView::Simple if selected => area.height.min(1),
        _ => 0,
    };
    for dy in 0..highlight_rows {
        buffer.set_string(area.x, area.y + dy, " ".repeat(area.width as usize), base_style);
    }

    match list_view {
        ModListView::Detailed => {
            let thumb_width = 8u16;
            let text_x = area.x + thumb_width + 2;
            let content_height = area.height.min(4);
            let safe_marker_width = if package.safe_mode_enabled { 0 } else { 1 };
            let text_width = area.width.saturating_sub(thumb_width + 2 + safe_marker_width) as usize;
            if text_width == 0 { return; }

            for (idx, line) in package.thumbnail.rendered_lines.iter().take(content_height as usize).enumerate() {
                settings_common::render_compiled_line_to_buffer(buffer, area.x, area.y + idx as u16, thumb_width as usize, line);
            }

            settings_common::render_mod_debug_prefix(buffer, text_x, area.y, package.debug_enabled, selected);
            let name_x = text_x + if package.debug_enabled { 3 } else { 0 };
            let name_width = text_width.saturating_sub(if package.debug_enabled { 3 } else { 0 });
            settings_common::render_manifest_text_to_buffer(buffer, name_x, area.y, name_width, &package.package_name, package.package_name_allows_rich, base_style.add_modifier(Modifier::BOLD));

            if content_height > 1 {
                settings_common::render_label_manifest_value_to_buffer(buffer, text_x, area.y + 1, text_width, &settings_common::text("settings.mods.author", "Author:"), &package.author, true, meta_style);
            }
            if content_height > 2 {
                settings_common::render_label_manifest_value_to_buffer(buffer, text_x, area.y + 2, text_width, &settings_common::text("settings.mods.version", "Version:"), &package.version, true, meta_style);
            }
            if content_height > 3 {
                settings_common::render_mod_status_line(buffer, text_x, area.y + 3, text_width, package, selected);
            }
            if !package.safe_mode_enabled {
                settings_common::render_safe_mode_marker_column(buffer, area, content_height);
            }
        }
        ModListView::Simple => {
            let safe_marker_width = if package.safe_mode_enabled { 0 } else { 1 };
            let status_width = 6usize;
            let text_width = area.width.saturating_sub(safe_marker_width) as usize;
            let name_width = text_width.saturating_sub(status_width + 1);
            let text_x = area.x;

            settings_common::render_mod_debug_prefix(buffer, text_x, area.y, package.debug_enabled, selected);
            let name_x = text_x + if package.debug_enabled { 3 } else { 0 };
            let actual_name_width = name_width.saturating_sub(if package.debug_enabled { 3 } else { 0 });
            settings_common::render_manifest_text_to_buffer(buffer, name_x, area.y, actual_name_width, &package.package_name, package.package_name_allows_rich, base_style);

            let status_x = area.x + name_width as u16 + 1;
            settings_common::render_enabled_tag(buffer, status_x, area.y, package.enabled, selected);

            if !package.safe_mode_enabled {
                buffer.set_string(area.x + area.width - 1, area.y, " ", Style::default().bg(Color::Red));
            }
        }
    }
}

// 渲染单个 Mod 列表项
fn render_mod_detail(frame: &mut ratatui::Frame<'_>, area: Rect, state: &mut SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", settings_common::text("settings.mods.detail", "Mod Details")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(package) = state.mod_packages.get(state.mod_selected) else {
        frame.render_widget(Paragraph::new(settings_common::text("settings.mods.empty", "No mods found.")).style(Style::default().fg(Color::White)).alignment(Alignment::Center), inner);
        return;
    };

    let build_detail_lines = |content_width: usize| {
        let mut lines = settings_common::rich_lines_from_image(&package.banner, content_width, Style::default().fg(Color::White));
        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text("settings.mods.section.basic_info", "Basic Info")));
        lines.extend(settings_common::label_value_lines(settings_common::text("settings.mods.package_label", "Mod Package:"), package.package_name.clone(), package.package_name_allows_rich, content_width, Style::default().fg(Color::White)));
        lines.extend(settings_common::label_value_lines(settings_common::text("settings.mods.author", "Author:"), package.author.clone(), true, content_width, Style::default().fg(Color::White)));
        lines.extend(settings_common::label_value_lines(settings_common::text("settings.mods.version", "Version:"), package.version.clone(), true, content_width, Style::default().fg(Color::White)));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text("settings.mods.section.storage", "Data Storage")));
        lines.push(settings_common::label_value_line(settings_common::text("settings.mods.best_score", "Best Score:"), if package.has_best_score_storage { settings_common::text("settings.mods.storage_has", "Available") } else { settings_common::text("settings.mods.storage_none", "None") }, if package.has_best_score_storage { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) }));
        lines.push(settings_common::label_value_line(settings_common::text("settings.mods.save_data", "Game Save:"), if package.has_save_storage { settings_common::text("settings.mods.storage_has", "Available") } else { settings_common::text("settings.mods.storage_none", "None") }, if package.has_save_storage { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) }));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text("settings.mods.section.security", "Security")));
        lines.push(settings_common::label_value_line(settings_common::text("settings.mods.write_request", "Direct Write Request:"), if package.has_write_request { settings_common::text("settings.mods.storage_has", "Available") } else { settings_common::text("settings.mods.storage_none", "None") }, if package.has_write_request { Style::default().fg(Color::Red) } else { Style::default().fg(Color::DarkGray) }));

        let safe_mode_text = match package.safe_mode_state {
            ModSafeModeState::Enabled => settings_common::text("settings.mods.safe_mode_on", "On"),
            ModSafeModeState::DisabledSession => settings_common::text("settings.mods.safe_mode_session_off", "Disabled (This Session)"),
            ModSafeModeState::DisabledTrusted => settings_common::text("settings.mods.safe_mode_trusted_off", "Disabled (Permanently Trusted)"),
        };
        lines.push(settings_common::label_value_line(settings_common::text("settings.mods.safe_mode", "Safe Mode:"), safe_mode_text, if package.safe_mode_enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }));

        lines.push(Line::from(""));
        lines.push(settings_common::section_title_line(settings_common::text("settings.mods.section.introduction", "Mod Introduction")));
        lines.extend(rich_text::parse_rich_text_wrapped(&package.introduction, content_width, Style::default().fg(Color::White)));
        lines
    };

    let viewport_h = inner.height as usize;
    let full_width = inner.width.max(1) as usize;
    let wide_lines = build_detail_lines(full_width);
    let needs_scroll = wide_lines.len() > viewport_h;

    let (lines, text_area) = if needs_scroll && inner.width > 2 {
        (build_detail_lines(inner.width.saturating_sub(2).max(1) as usize), Rect::new(inner.x, inner.y, inner.width - 2, inner.height))
    } else {
        (wide_lines, inner)
    };

    let max_scroll = lines.len().saturating_sub(viewport_h);
    if state.mod_detail_scroll > max_scroll { state.mod_detail_scroll = max_scroll; }
    state.mod_detail_scroll_available = max_scroll > 0;

    frame.render_widget(Paragraph::new(lines).style(Style::default().fg(Color::White)).wrap(Wrap { trim: false }).scroll((state.mod_detail_scroll as u16, 0)), text_area);

    if state.mod_detail_scroll_available && inner.width > 2 {
        let scroll_x = inner.x + inner.width - 1;
        let can_up = state.mod_detail_scroll > 0;
        let can_down = state.mod_detail_scroll < max_scroll;
        frame.render_widget(Paragraph::new(if can_up { "↑" } else { " " }).style(Style::default().fg(Color::White)), Rect::new(scroll_x, inner.y, 1, 1));
        frame.render_widget(Paragraph::new(if can_up { "W" } else { " " }).style(Style::default().fg(Color::White)), Rect::new(scroll_x, inner.y.saturating_add(1), 1, 1));
        if inner.height > 4 {
            let track_start = inner.y.saturating_add(2);
            let track_len = inner.height.saturating_sub(4);
            let pos = if max_scroll == 0 { 0 } else { ((state.mod_detail_scroll * (track_len as usize - 1)) / max_scroll) as u16 };
            frame.render_widget(Paragraph::new("█").style(Style::default().fg(Color::White)), Rect::new(scroll_x, track_start.saturating_add(pos), 1, 1));
        }
        let d_y = inner.y + inner.height.saturating_sub(2);
        frame.render_widget(Paragraph::new(if can_down { "S" } else { " " }).style(Style::default().fg(Color::White)), Rect::new(scroll_x, d_y, 1, 1));
        frame.render_widget(Paragraph::new(if can_down { "↓" } else { " " }).style(Style::default().fg(Color::White)), Rect::new(scroll_x, d_y.saturating_add(1), 1, 1));
    }
}

// 渲染右栏详情：Banner 图、基本信息、存储支持、安全信息、Mod 介绍、滚动条
pub fn render_mod_safe_dialog(frame: &mut ratatui::Frame<'_>, dialog: &ModSafeDialog) {
    use ratatui::widgets::Clear;
    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 5u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let message = settings_common::text(
        "settings.mods.safe_mode_dialog.message",
        "Are you sure you want to disable Safe Mode for mod \"{mod_name}\"?\n\nSafe Mode is designed to protect your device. After disabling it, this mod may perform high-risk operations such as file writes or system calls, which may cause data loss or system instability.\nPlease make sure you fully trust the source and author of this mod.",
    ).replace("{mod_name}", &dialog.mod_name);

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = settings_common::wrap_plain_text_lines(&message, content_width, Style::default().fg(Color::White));
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(settings_common::text("settings.mods.safe_mode_dialog.cancel", "Cancel"), Style::default().fg(Color::LightGreen)),
    ]));
    let gated_style = Style::default().fg(if countdown_done { Color::Red } else { Color::DarkGray });
    lines.push(Line::from(vec![
        Span::styled("[2] ", index_style),
        Span::styled(format!("{} {}{}", settings_common::text("settings.mods.safe_mode_dialog.disable_once", "Confirm Disable"), settings_common::text("settings.mods.safe_mode_dialog.disable_once_sub", "(Only This Time)"), if countdown_done { String::new() } else { format!(" {}", settings_common::text("settings.mods.safe_mode_dialog.countdown", "{seconds}s").replace("{seconds}", &remaining.to_string())) }), gated_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("[3] ", index_style),
        Span::styled(format!("{} {}{}", settings_common::text("settings.mods.safe_mode_dialog.disable_forever", "Confirm Disable"), settings_common::text("settings.mods.safe_mode_dialog.disable_forever_sub", "(Permanently Trust This Mod)"), if countdown_done { String::new() } else { format!(" {}", settings_common::text("settings.mods.safe_mode_dialog.countdown", "{seconds}s").replace("{seconds}", &remaining.to_string())) }), gated_style),
    ]));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height.saturating_add(2).clamp(8, area.height.saturating_sub(2).max(8));
    let rect = settings_common::centered_rect(area, width, height);
    let block = Block::default().borders(Borders::ALL).title(format!(" {} ", settings_common::text("settings.mods.safe_mode_dialog.title", "Safe Mode"))).border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).alignment(Alignment::Left), inner);
    settings_common::render_box_back_hint(frame, rect, settings_common::text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"));
}

// 渲染安全模式禁用确认对话框（带 5 秒倒计时）：[1]取消、[2]仅本次、[3]永久信任
pub fn handle_mods_key(state: &mut SettingsState, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;

    if let Some(dialog) = &state.mod_safe_dialog {
        let countdown_done = dialog.opened_at.elapsed().as_secs() >= 5;
        match code {
            KeyCode::Esc | KeyCode::Char('1') => state.mod_safe_dialog = None,
            KeyCode::Char('2') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, false);
                state.mod_safe_dialog = None;
                crate::app::content_cache::reload();
                state.refresh_mods();
            }
            KeyCode::Char('3') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, true);
                state.mod_safe_dialog = None;
                crate::app::content_cache::reload();
                state.refresh_mods();
            }
            _ => {}
        }
        return;
    }

    if let Some(dialog) = state.mod_page_jump_dialog.as_mut() {
        let total_pages = settings_common::total_mod_pages(state.mod_packages.len(), settings_common::current_mod_page_size(state.mod_list_view));
        match code {
            KeyCode::Esc => state.mod_page_jump_dialog = None,
            KeyCode::Backspace => { dialog.input.pop(); }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if dialog.input.len() < 4 { dialog.input.push(ch); }
            }
            KeyCode::Enter => {
                if let Ok(page) = dialog.input.parse::<usize>() && (1..=total_pages.max(1)).contains(&page) {
                    state.mod_page = page - 1;
                    let start = state.mod_page * settings_common::current_mod_page_size(state.mod_list_view);
                    state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                    state.mod_detail_scroll = 0;
                }
                state.mod_page_jump_dialog = None;
            }
            _ => {}
        }
        return;
    }

    let page_size = settings_common::current_mod_page_size(state.mod_list_view);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.mod_selected = state.mod_selected.saturating_sub(1);
            state.mod_page = state.mod_selected / page_size;
            state.mod_detail_scroll = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.mod_packages.is_empty() {
                state.mod_selected = (state.mod_selected + 1).min(state.mod_packages.len().saturating_sub(1));
                state.mod_page = state.mod_selected / page_size;
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') => state.mod_detail_scroll = state.mod_detail_scroll.saturating_sub(1),
        KeyCode::Char('s') | KeyCode::Char('S') => state.mod_detail_scroll = state.mod_detail_scroll.saturating_add(1),
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if state.mod_page > 0 {
                state.mod_page -= 1;
                let start = state.mod_page * page_size;
                state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            let total_pages = settings_common::total_mod_pages(state.mod_packages.len(), page_size);
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
                crate::app::content_cache::reload();
                state.refresh_mods();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                let _ = mods::set_mod_debug_enabled(&package.namespace, !package.debug_enabled);
                crate::app::content_cache::reload();
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
                    crate::app::content_cache::reload();
                    state.refresh_mods();
                }
            }
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            crate::app::content_cache::reload();
            state.refresh_mods();
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.toggle_mod_list_view();
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            let next = match state.mod_sort_mode {
                ModSortMode::Name => ModSortMode::Enabled,
                ModSortMode::Enabled => ModSortMode::Author,
                ModSortMode::Author => ModSortMode::SafeMode,
                ModSortMode::SafeMode => ModSortMode::Name,
            };
            state.set_mod_sort_mode(next);
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            state.toggle_mod_sort_order();
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if settings_common::total_mod_pages(state.mod_packages.len(), page_size) > 1 {
                state.mod_page_jump_dialog = Some(ModPageJumpDialog { input: String::new() });
            }
        }
        KeyCode::Esc => state.page = SettingsPage::Hub,
        _ => {}
    }
}