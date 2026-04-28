// 按键绑定配置页面，提供游戏动作与物理按键的映射编辑功能。支持双栏布局（左栏游戏列表，右栏按键映射表），按键捕获、冲突检测、还原和持久化

use std::time::{Duration, Instant}; // 按键捕获延时和超时

use ratatui::buffer::Buffer; // 终端缓冲区直接写入
use ratatui::layout::{Constraint, Direction, Layout, Rect}; // 布局管理
use ratatui::style::{Color, Modifier, Style}; // 样式控制
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::{Block, Borders, Paragraph}; // 边框块和段落
use unicode_width::UnicodeWidthStr; // 文本宽度

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind}; // 按键事件和类型（判断按下/重复）

use crate::app::i18n; // 国际化
use crate::app::settings::common as settings_common; // 通用工具
use crate::app::settings::types::*; // 	类型定义
use crate::core::key::{display_semantic_key, semantic_key_source}; // 语义键显示和双源输入（crossterm+rdev）
use crate::core::save as runtime_save; // 官方游戏键位持久化
use crate::game::action::{ActionBinding, ActionKeys}; // 动作绑定类型
use crate::game::registry::GameDescriptor; // 游戏描述符
use crate::game::resources;  // 包级文本解析

const SHIFT_BIND_HOLD: Duration = Duration::from_secs(1); // Shift 键绑定的长按时间
const KEYBIND_ACTION_PADDING: u16 = 2; // 动作名称前的空白填充
const KEYBIND_CAPTURE_DELAY: Duration = Duration::from_millis(120); // 进入捕获模式后等待 120ms 再接收按键，防止触发按键本身被捕获

// 计算键位页面最小尺寸
pub fn minimum_size_keybind(state: &SettingsState) -> (u16, u16) {
    let max_name_w = state
        .keybind_games
        .iter()
        .map(|game| UnicodeWidthStr::width(game.display_name.as_str()))
        .max()
        .unwrap_or(12) as u16;
    let left_w = (max_name_w + 6).max(22);
    let right_w = 72u16;
    (left_w + right_w + 4, 16)
}

// 主渲染函数：准备分页 → 构建提示 → 左右分栏（40%/60%）→ 渲染游戏列表和按键映射面板
pub fn render_keybind(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if state.keybind_games.is_empty() {
        state.keybind_selected = 0;
        state.keybind_page = 0;
    } else {
        let page_size = settings_common::current_keybind_page_size();
        let total_pages = settings_common::total_keybind_pages(state.keybind_games.len(), page_size);
        if state.keybind_page >= total_pages {
            state.keybind_page = total_pages.saturating_sub(1);
        }
        let page_start = state.keybind_page * page_size;
        let page_end = (page_start + page_size).min(state.keybind_games.len());
        if state.keybind_selected < page_start || state.keybind_selected >= page_end {
            state.keybind_selected = page_start.min(state.keybind_games.len().saturating_sub(1));
        }
    }

    let area = frame.area();
    let hint_segments = build_keybind_hint_segments(state);
    let hint_lines = settings_common::wrap_keybind_hint_lines(&hint_segments, area.width.max(1) as usize);
    let hint_height = hint_lines.len().max(1) as u16;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(hint_height)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(root[0]);

    render_keybind_game_list(frame, columns[0], state);
    render_keybind_mapping_panel(frame, columns[1], state);
    frame.render_widget(
        Paragraph::new(hint_lines)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center),
        root[1],
    );
}

// 根据当前状态构建操作提示文本：捕获模式 / 动作编辑模式 / 游戏选择模式
fn build_keybind_hint_segments(state: &SettingsState) -> Vec<String> {
    if state.keybind_capture.is_some() {
        return vec![
            settings_common::text("settings.keybind.hint.segment.capture_any", "[Any Key] Bind to this slot"),
            settings_common::text("settings.keybind.hint.segment.capture_shift", "[Shift] Hold for 2s to bind Shift"),
        ];
    }

    if state.keybind_focus == KeybindFocus::Actions {
        let mut segments = vec![
            settings_common::text("settings.keybind.hint.segment.move", "[↑]/[↓] Move"),
            settings_common::text("settings.keybind.hint.segment.scroll", "[W]/[S] Scroll"),
            settings_common::text("settings.keybind.hint.segment.reset_action", "[Z] Reset Action"),
            settings_common::text("settings.keybind.hint.segment.reset_game", "[R] Reset Current Game"),
            settings_common::text("settings.keybind.hint.segment.toggle_mode", "[X] Toggle Mode"),
        ];
        segments.push(match state.keybind_edit_mode {
            KeybindEditMode::Add => settings_common::text("settings.keybind.hint.segment.add_key", "[1]-[5] Add/Rebind Key"),
            KeybindEditMode::Delete => settings_common::text("settings.keybind.hint.segment.delete_key", "[1]-[5] Delete Key"),
        });
        return segments;
    }

    vec![
        settings_common::text("settings.keybind.hint.segment.move", "[↑]/[↓] Move"),
        settings_common::text("settings.keybind.hint.segment.page", "[Q]/[E] Page"),
        settings_common::text("settings.keybind.hint.segment.jump", "[P] Jump Page"),
        settings_common::text("settings.keybind.hint.segment.sort_mode", "[Z] Sort Mode"),
        settings_common::text("settings.keybind.hint.segment.sort_order", "[X] Sort Order"),
        settings_common::text("settings.keybind.hint.segment.select", "[Enter] Select"),
        settings_common::text("settings.keybind.hint.segment.save_exit", "[Esc]/[Q] Save and Exit"),
    ]
}

// 渲染左栏游戏列表，支持分页和 MOD 徽章。缺少按键的游戏以红色标记
fn render_keybind_game_list(frame: &mut ratatui::Frame<'_>, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(settings_common::keybind_game_list_title(state))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if state.keybind_games.is_empty() {
        frame.render_widget(
            Paragraph::new(i18n::t("game_selection.empty"))
                .style(Style::default().fg(Color::White))
                .alignment(ratatui::layout::Alignment::Center),
            rows[0],
        );
        frame.render_widget(
            Paragraph::new("1/1").style(Style::default().fg(Color::White)).alignment(ratatui::layout::Alignment::Center),
            rows[1],
        );
        return;
    }

    let page_size = rows[0].height.max(1) as usize;
    let total_pages = settings_common::total_keybind_pages(state.keybind_games.len(), page_size);
    let page = state.keybind_page.min(total_pages.saturating_sub(1));
    let start = page * page_size;
    let page_games = state.keybind_games.iter().skip(start).take(page_size).collect::<Vec<_>>();

    for (index, game) in page_games.iter().enumerate() {
        let y = rows[0].y + index as u16;
        if y >= rows[0].y + rows[0].height {
            break;
        }
        let selected = start + index == state.keybind_selected;
        let invalid = game_has_missing_keys(game);
        if selected {
            fill_buffer_row(frame.buffer_mut(), rows[0].x, y, rows[0].width, Style::default().bg(Color::LightBlue));
        }
        let style = if selected && invalid {
            Style::default().fg(Color::Red).bg(Color::LightBlue)
        } else if selected {
            Style::default().fg(Color::Black).bg(Color::LightBlue)
        } else if invalid {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(settings_common::keybind_game_list_line(game, rows[0].width as usize, style)),
            Rect::new(rows[0].x, y, rows[0].width, 1),
        );
    }

    let left = if page > 0 { i18n::t("game_selection.pager.prev") } else { String::new() };
    let right = if page + 1 < total_pages { i18n::t("game_selection.pager.next") } else { String::new() };
    let pager_line = if let Some(input) = &state.keybind_page_jump_input {
        let input_text = if input.is_empty() { "_".to_string() } else { input.clone() };
        Line::from(vec![
            Span::styled(input_text, Style::default().fg(if input.is_empty() { Color::Yellow } else { Color::Black }).bg(Color::Yellow)),
            Span::styled(format!("/{}", total_pages.max(1)), Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::styled(format!("{}/{}", page + 1, total_pages.max(1)), Style::default().fg(Color::White)))
    };

    frame.render_widget(Paragraph::new(left).style(Style::default().fg(Color::White)).alignment(ratatui::layout::Alignment::Left), rows[1]);
    frame.render_widget(Paragraph::new(pager_line).alignment(ratatui::layout::Alignment::Center), rows[1]);
    frame.render_widget(Paragraph::new(right).style(Style::default().fg(Color::White)).alignment(ratatui::layout::Alignment::Right), rows[1]);
}

// 渲染右栏映射表：标题、表头（Action + 5 个按键槽）、动作行列表、槽位编辑高亮
fn render_keybind_mapping_panel(frame: &mut ratatui::Frame<'_>, area: Rect, state: &mut SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(keybind_mapping_title(state.keybind_games.get(state.keybind_selected)))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    let viewport_h = inner.height.saturating_sub(2) as usize;
    sync_keybind_action_view(state, viewport_h);
    let selected_game = state.keybind_games.get(state.keybind_selected);
    frame.render_widget(block, area);

    let Some(game) = selected_game else {
        frame.render_widget(Paragraph::new(i18n::t("game_selection.empty")).style(Style::default().fg(Color::White)).alignment(ratatui::layout::Alignment::Center), inner);
        return;
    };

    let action_col_width = ((inner.width as f32) * 0.20).floor() as u16;
    let key_col_width = ((inner.width as f32) * 0.16).floor() as u16;
    let header_y = inner.y;
    let data_y = inner.y + 2;
    let mut x = inner.x;
    render_cell_text(frame.buffer_mut(), x.saturating_add(KEYBIND_ACTION_PADDING), header_y, action_col_width.saturating_sub(KEYBIND_ACTION_PADDING), &settings_common::text("settings.keybind.action", "Action"), Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    x += action_col_width;
    for slot in 0..5 {
        render_cell_text(frame.buffer_mut(), x, header_y, key_col_width, &format!("[{}] {}", slot + 1, settings_common::text("settings.keybind.key", "Key")), Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        x += key_col_width;
    }
    frame.buffer_mut().set_string(inner.x, inner.y + 1, "─".repeat(inner.width.max(1) as usize), Style::default().fg(Color::White));

    let actions = keybind_action_rows(game);
    let max_scroll = actions.len().saturating_sub(viewport_h);
    let scroll = state.keybind_action_scroll.min(max_scroll);
    let selected_row = state.keybind_action_selected.min(actions.len().saturating_sub(1));

    for (visible_idx, (action_name, _binding_key, binding)) in actions.iter().skip(scroll).take(viewport_h).enumerate() {
        let y = data_y + visible_idx as u16;
        let selected = state.keybind_focus == KeybindFocus::Actions && selected_row == scroll + visible_idx;
        let slots = binding.slots();
        let missing = slots.iter().all(|slot| slot.trim().is_empty());
        if selected {
            fill_buffer_row(frame.buffer_mut(), inner.x, y, inner.width, if state.keybind_edit_mode == KeybindEditMode::Delete { Style::default().bg(Color::LightRed) } else { Style::default().bg(Color::LightBlue) });
        }
        let row_style = if selected {
            let bg = if state.keybind_edit_mode == KeybindEditMode::Delete { Color::LightRed } else { Color::LightBlue };
            Style::default().fg(Color::Black).bg(bg)
        } else {
            Style::default().fg(Color::White)
        };
        if missing {
            frame.buffer_mut().set_string(inner.x, y, " ", Style::default().bg(Color::Red));
            frame.buffer_mut().set_string(inner.x + inner.width.saturating_sub(1), y, " ", Style::default().bg(Color::Red));
        }
        render_cell_text(frame.buffer_mut(), inner.x.saturating_add(KEYBIND_ACTION_PADDING), y, action_col_width.saturating_sub(KEYBIND_ACTION_PADDING), action_name, if selected { row_style } else { Style::default().fg(Color::White) });
        let mut x = inner.x + action_col_width;
        for slot in 0..5 {
            let value = slots.get(slot).cloned().unwrap_or_default();
            let formatted = if value.trim().is_empty() { String::new() } else { display_semantic_key(&value, game.case_sensitive) };
            render_key_slot(frame.buffer_mut(), x, y, key_col_width, formatted.as_str(), row_style);
            x += key_col_width;
        }
    }
}

// 构建映射面板标题，显示游戏名和大小写敏感提示
fn keybind_mapping_title(selected_game: Option<&GameDescriptor>) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("── {}: ", settings_common::text("settings.keybind.mapping_title", "Key Mapping")), Style::default().fg(Color::White))];
    if let Some(game) = selected_game {
        spans.push(Span::styled(game.display_name.clone(), Style::default().fg(Color::White)));
        if game.case_sensitive {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(settings_common::text("settings.keybind.case_sensitive_hint", "Letter keys are case-sensitive"), Style::default().fg(Color::Yellow)));
        }
    }
    Line::from(spans)
}

// 获取动作的本地化名称（通过包级 i18n 解析）
fn localized_action_name(game: &GameDescriptor, binding: &crate::game::action::ActionBinding) -> String {
    if let Some(package) = game.package_info() { resources::resolve_package_text(package, binding.key_name()) } else { binding.key_name().to_string() }
}

// 获取游戏的动作列表（名称、键、绑定对象）元组
fn keybind_action_rows(game: &GameDescriptor) -> Vec<(String, String, ActionBinding)> {
    game.actions.iter().map(|(binding_key, binding)| (localized_action_name(game, binding), binding_key.clone(), binding.clone())).collect()
}

// 获取动作数量，至少为 1
fn keybind_action_count(game: &GameDescriptor) -> usize { game.actions.len().max(1) }

// 获取当前选中的游戏引用
fn selected_keybind_game(state: &SettingsState) -> Option<&GameDescriptor> { state.keybind_games.get(state.keybind_selected) }

// 获取当前选中的游戏可变引用
fn selected_keybind_game_mut(state: &mut SettingsState) -> Option<&mut GameDescriptor> { state.keybind_games.get_mut(state.keybind_selected) }

// 检查游戏是否有未设置按键的动作
fn game_has_missing_keys(game: &GameDescriptor) -> bool { game.actions.values().any(|binding| binding.keys().is_empty()) }

// 检查所有游戏是否都有完整的按键绑定
fn keybind_all_games_valid(state: &SettingsState) -> bool { state.keybind_games.iter().all(|game| !game_has_missing_keys(game)) }

// 获取当前选中动作的绑定键名
fn selected_action_binding_key(state: &SettingsState) -> Option<String> {
    let selected_index = state.keybind_action_selected;
    selected_keybind_game(state).and_then(|game| keybind_action_rows(game).get(selected_index).map(|(_, binding_key, _)| binding_key.clone()))
}

// 同步动作列表的滚动视口，确保选中项在可见范围内
fn sync_keybind_action_view(state: &mut SettingsState, viewport_h: usize) {
    let action_count = selected_keybind_game(state).map(keybind_action_count).unwrap_or(1);
    state.keybind_action_selected = state.keybind_action_selected.min(action_count.saturating_sub(1));
    if viewport_h == 0 { return; }
    if state.keybind_action_selected < state.keybind_action_scroll {
        state.keybind_action_scroll = state.keybind_action_selected;
    } else if state.keybind_action_selected >= state.keybind_action_scroll + viewport_h {
        state.keybind_action_scroll = state.keybind_action_selected.saturating_sub(viewport_h.saturating_sub(1));
    }
}

// 在缓冲区指定位置渲染文本
fn render_cell_text(buffer: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) { if width > 0 { buffer.set_stringn(x, y, text, width as usize, style); } }

// 用空格填充整行（用于高亮背景）
fn fill_buffer_row(buffer: &mut Buffer, x: u16, y: u16, width: u16, style: Style) { if width > 0 { buffer.set_string(x, y, " ".repeat(width as usize), style); } }

// 渲染单个按键槽位 [Key]，空槽位显示 [ ]
fn render_key_slot(buffer: &mut Buffer, x: u16, y: u16, width: u16, value: &str, row_style: Style) {
    if width == 0 { return; }
    let bracket_style = Style::default().fg(Color::White).bg(row_style.bg.unwrap_or(Color::Reset));
    let value_style = Style::default().fg(row_style.fg.unwrap_or(Color::White)).bg(row_style.bg.unwrap_or(Color::Reset));
    let value_width = UnicodeWidthStr::width(value);
    let slot_width = (if value.is_empty() { 2 } else { value_width.saturating_add(4) }).min(width as usize) as u16;
    let end_x = x + slot_width.saturating_sub(1);
    buffer.set_string(x, y, "[", bracket_style);
    if !value.is_empty() { buffer.set_stringn(x + 2, y, value, slot_width.saturating_sub(3) as usize, value_style); }
    buffer.set_string(end_x, y, "]", bracket_style);
}

// 从 crossterm 事件捕获按键，备用从 rdev 获取，过滤左右的 Shift
fn capture_key_name(key: KeyEvent) -> Option<String> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) { return None; }
    let mut names = semantic_key_source().record_crossterm_key(key);
    if names.is_empty() { names = semantic_key_source().drain_ready_rdev_keys(4); }
    names.into_iter().find(|name| !matches!(name.as_str(), "left_shift" | "right_shift"))
}

// 规范化按键名：合并左右 Shift 为 "shift"，非大小写敏感时转小写
fn normalize_bound_key_name(value: String, case_sensitive: bool) -> String {
    match value.as_str() { "left_shift" | "right_shift" => "shift".to_string(), _ if case_sensitive => value, _ => value.to_lowercase() }
}

// 检查两个键名是否冲突
fn key_names_conflict(left: &str, right: &str, case_sensitive: bool) -> bool { if case_sensitive { left == right } else { left.eq_ignore_ascii_case(right) } }

// 去除重复和空槽位，最多保留 5 个
fn compact_key_slots(slots: Vec<String>, case_sensitive: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for slot in slots.into_iter().filter(|slot| !slot.trim().is_empty()) {
        if out.iter().any(|existing| key_names_conflict(existing, &slot, case_sensitive)) { continue; }
        out.push(slot);
        if out.len() >= 5 { break; }
    }
    out
}

// 将槽位列表设置到绑定对象
fn set_binding_slots(binding: &mut ActionBinding, slots: Vec<String>, case_sensitive: bool) {
    let slots = compact_key_slots(slots, case_sensitive);
    binding.key = match slots.len() { 0 => ActionKeys::Multiple(Vec::new()), 1 => ActionKeys::Single(slots[0].clone()), _ => ActionKeys::Multiple(slots) };
}

// 将槽位应用到游戏，可选移除其他动作中冲突的键
fn apply_binding_slots_to_game(game: &mut GameDescriptor, binding_key: &str, slots: Vec<String>, remove_conflicts: bool) {
    let normalized_slots = compact_key_slots(slots, game.case_sensitive);
    if remove_conflicts {
        let conflict_keys = normalized_slots.clone();
        for (other_key, other_binding) in &mut game.actions {
            if other_key == binding_key { continue; }
            let mut other_slots = other_binding.slots();
            let mut changed = false;
            for slot in &mut other_slots {
                if conflict_keys.iter().any(|key| key_names_conflict(slot, key, game.case_sensitive)) { slot.clear(); changed = true; }
            }
            if changed { set_binding_slots(other_binding, other_slots, game.case_sensitive); }
        }
    }
    if let Some(binding) = game.actions.get_mut(binding_key) { set_binding_slots(binding, normalized_slots, game.case_sensitive); }
}

// 将捕获的按键应用到当前选中动作的指定槽位
fn apply_keybind_to_selected_game(state: &mut SettingsState, slot_index: usize, key_name: String) {
    let Some(binding_key) = selected_action_binding_key(state) else { return; };
    let Some(game) = selected_keybind_game_mut(state) else { return; };
    let mut slots = game.actions.get(binding_key.as_str()).map(ActionBinding::slots).unwrap_or_default();
    while slots.len() <= slot_index { slots.push(String::new()); }
    slots[slot_index] = key_name;
    apply_binding_slots_to_game(game, binding_key.as_str(), slots, true);
    persist_selected_game_keybindings(game);
}

// 删除指定槽位的按键
fn delete_keybind_slot(state: &mut SettingsState, slot_index: usize) {
    let Some(binding_key) = selected_action_binding_key(state) else { return; };
    let Some(game) = selected_keybind_game_mut(state) else { return; };
    if let Some(binding) = game.actions.get_mut(binding_key.as_str()) {
        let mut slots = binding.slots();
        if slot_index < slots.len() { slots[slot_index].clear(); set_binding_slots(binding, slots, game.case_sensitive); persist_selected_game_keybindings(game); }
    }
}

// 将当前动作重置为默认绑定
fn reset_selected_action_keybind(state: &mut SettingsState) {
    let Some(binding_key) = selected_action_binding_key(state) else { return; };
    let Some(game) = selected_keybind_game_mut(state) else { return; };
    if let Some(default_binding) = game.default_actions.get(binding_key.as_str()).cloned() { apply_binding_slots_to_game(game, binding_key.as_str(), default_binding.slots(), true); persist_selected_game_keybindings(game); }
}

// 将当前游戏的所有动作重置为默认
fn reset_selected_game_keybinds(state: &mut SettingsState) {
    let Some(game) = selected_keybind_game_mut(state) else { return; };
    game.actions = game.default_actions.clone();
    persist_selected_game_keybindings(game);
}

// 持久化：Mod 游戏存到 mod_state，官方游戏存到 saves.json
fn persist_selected_game_keybindings(game: &GameDescriptor) {
    let bindings: std::collections::HashMap<_, _> = game.actions.iter().map(|(action, binding)| (action.clone(), binding.slots())).collect();
    if let Some(package) = game.package_info() && game.is_mod_game() {
        let _ = crate::mods::update_mod_keybindings(package.namespace.as_str(), game.id.as_str(), game.entry.as_str(), bindings);
    } else {
        let _ = runtime_save::save_keybindings(game.id.as_str(), &bindings);
    }
}

// 轮询按键捕获（Shift 长按 / rdev 按键），由 poll_mod_hot_reload 调用
pub fn poll_keybind_capture(state: &mut SettingsState) -> bool {
    if state.page != SettingsPage::Keybind || state.keybind_capture.is_none() { return false; }
    if state.keybind_capture.as_ref().map(|capture| Instant::now() < capture.accept_after).unwrap_or(false) { return false; }
    if semantic_key_source().is_shift_held_for(SHIFT_BIND_HOLD) {
        let slot_index = state.keybind_capture.as_ref().map(|capture| capture.slot_index).unwrap_or(0);
        apply_keybind_to_selected_game(state, slot_index, "shift".to_string());
        state.keybind_capture = None;
        return true;
    }
    let keys = semantic_key_source().drain_ready_rdev_keys(4);
    if let Some(key_name) = keys.into_iter().find(|key_name| !matches!(key_name.as_str(), "left_shift" | "right_shift")) {
        let slot_index = state.keybind_capture.as_ref().map(|capture| capture.slot_index).unwrap_or(0);
        let case_sensitive = selected_keybind_game(state).map(|game| game.case_sensitive).unwrap_or(false);
        apply_keybind_to_selected_game(state, slot_index, normalize_bound_key_name(key_name, case_sensitive));
        state.keybind_capture = None;
        return true;
    }
    false
}

// 主按键处理函数，包含捕获模式、跳页输入、游戏选择、动作编辑四种状态的处理
pub fn handle_keybind_key(state: &mut SettingsState, key: KeyEvent) {
    let code = key.code;
    if let Some(capture) = state.keybind_capture.clone() {
        if Instant::now() < capture.accept_after { return; }
        if let Some(bound_key) = capture_key_name(key) { apply_keybind_to_selected_game(state, capture.slot_index, bound_key); state.keybind_capture = None; }
        return;
    }

    if let Some(input) = state.keybind_page_jump_input.as_mut() {
        let total_pages = settings_common::total_keybind_pages(state.keybind_games.len(), settings_common::current_keybind_page_size());
        match code {
            KeyCode::Esc => state.keybind_page_jump_input = None,
            KeyCode::Backspace => { input.pop(); }
            KeyCode::Char(ch) if ch.is_ascii_digit() => { if input.len() < 4 { input.push(ch); } }
            KeyCode::Enter => {
                if let Ok(page) = input.parse::<usize>() && (1..=total_pages.max(1)).contains(&page) {
                    state.keybind_page = page - 1;
                    let start = state.keybind_page * settings_common::current_keybind_page_size();
                    state.keybind_selected = start.min(state.keybind_games.len().saturating_sub(1));
                }
                state.keybind_page_jump_input = None;
            }
            _ => {}
        }
        return;
    }

    let page_size = settings_common::current_keybind_page_size();
    match state.keybind_focus {
        KeybindFocus::Games => match code {
            KeyCode::Up | KeyCode::Char('k') => { state.keybind_selected = state.keybind_selected.saturating_sub(1); state.keybind_page = state.keybind_selected / page_size.max(1); }
            KeyCode::Down | KeyCode::Char('j') => {
                if !state.keybind_games.is_empty() { state.keybind_selected = (state.keybind_selected + 1).min(state.keybind_games.len().saturating_sub(1)); state.keybind_page = state.keybind_selected / page_size.max(1); }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if state.keybind_page > 0 { state.keybind_page -= 1; let start = state.keybind_page * page_size.max(1); state.keybind_selected = start.min(state.keybind_games.len().saturating_sub(1)); }
                else if keybind_all_games_valid(state) { crate::app::content_cache::reload(); state.refresh_keybind_games(); state.page = SettingsPage::Hub; }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                let total_pages = settings_common::total_keybind_pages(state.keybind_games.len(), page_size);
                if state.keybind_page + 1 < total_pages { state.keybind_page += 1; let start = state.keybind_page * page_size.max(1); state.keybind_selected = start.min(state.keybind_games.len().saturating_sub(1)); }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => { if settings_common::total_keybind_pages(state.keybind_games.len(), page_size) > 1 { state.keybind_page_jump_input = Some(String::new()); } }
            KeyCode::Char('z') | KeyCode::Char('Z') => { let next = match state.keybind_sort_mode { KeybindGameSortMode::Source => KeybindGameSortMode::Name, KeybindGameSortMode::Name => KeybindGameSortMode::Author, KeybindGameSortMode::Author => KeybindGameSortMode::Source }; state.set_keybind_sort_mode(next); }
            KeyCode::Char('x') | KeyCode::Char('X') => { state.toggle_keybind_sort_order(); }
            KeyCode::Enter => { state.keybind_focus = KeybindFocus::Actions; state.keybind_action_selected = 0; state.keybind_action_scroll = 0; }
            KeyCode::Esc => { if keybind_all_games_valid(state) { crate::app::content_cache::reload(); state.refresh_keybind_games(); state.page = SettingsPage::Hub; } }
            _ => {}
        },
        KeybindFocus::Actions => match code {
            KeyCode::Up | KeyCode::Char('k') => { state.keybind_action_selected = state.keybind_action_selected.saturating_sub(1); sync_keybind_action_view(state, 0); }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_index = selected_keybind_game(state).map(keybind_action_count).unwrap_or(1).saturating_sub(1);
                state.keybind_action_selected = (state.keybind_action_selected + 1).min(max_index);
                sync_keybind_action_view(state, 0);
            }
            KeyCode::Char('w') | KeyCode::Char('W') => { state.keybind_action_scroll = state.keybind_action_scroll.saturating_sub(1); }
            KeyCode::Char('s') | KeyCode::Char('S') => { state.keybind_action_scroll = state.keybind_action_scroll.saturating_add(1); sync_keybind_action_view(state, 0); }
            KeyCode::Char(ch) if ('1'..='5').contains(&ch) => {
                let slot_index = (ch as u8 - b'1') as usize;
                match state.keybind_edit_mode {
                    KeybindEditMode::Add => { semantic_key_source().clear_pending_keys(); state.keybind_capture = Some(KeybindCaptureState { slot_index, accept_after: Instant::now() + KEYBIND_CAPTURE_DELAY }); }
                    KeybindEditMode::Delete => { delete_keybind_slot(state, slot_index); }
                }
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => { reset_selected_action_keybind(state); }
            KeyCode::Char('r') | KeyCode::Char('R') => { reset_selected_game_keybinds(state); }
            KeyCode::Char('x') | KeyCode::Char('X') => { state.keybind_edit_mode = match state.keybind_edit_mode { KeybindEditMode::Add => KeybindEditMode::Delete, KeybindEditMode::Delete => KeybindEditMode::Add }; }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => { state.keybind_focus = KeybindFocus::Games; state.keybind_capture = None; }
            _ => {}
        },
    }
}