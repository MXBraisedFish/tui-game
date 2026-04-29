// 安全设置页面，提供四项操作——默认安全模式开关（带10秒倒计时确认）、默认Mod启用开关、一键启用所有安全模式、一键禁用所有Mod。显示绿色成功提示

use std::time::Instant; // 倒计时和成功提示计时

use ratatui::layout::{Alignment}; // 布局
use ratatui::style::{Color, Style}; // 样式
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::{Block, Borders, Paragraph, Wrap}; // 组件
use crate::app::settings::common as settings_common; // 通用工具
use crate::app::settings::types::*; // 类型定义

// 返回最小尺寸 (72, 14)
pub fn minimum_size_security() -> (u16, u16) {
    (72, 14)
}

// 渲染安全设置页面：四项选择项（默认安全模式、默认启用状态带开关状态，重置安全模式、重置所有Mod为操作按钮）。若有成功提示则额外渲染绿色提示（1秒后自动清除）
pub fn render_security(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if let Some(shown_at) = state.security_success_at
        && shown_at.elapsed().as_secs() >= 1
    {
        state.security_success_at = None;
    }

    let lines = vec![
        Line::from(""),
        settings_common::selection_option_with_value_line(
            0,
            state.security_selected,
            settings_common::text("settings.security.default_safe_mode", "Default mod safe mode"),
            state.default_safe_mode_enabled,
            "settings.security.enabled",
            "settings.security.disabled",
            "Enabled",
            "Disabled",
        ),
        settings_common::selection_option_with_value_line(
            1,
            state.security_selected,
            settings_common::text("settings.security.default_enabled", "Default mod enabled state"),
            state.default_mod_enabled,
            "settings.security.mod_enabled",
            "settings.security.mod_disabled",
            "Enabled",
            "Disabled",
        ),
        settings_common::selection_action_line(
            2,
            state.security_selected,
            settings_common::text("settings.security.reset_safe_mode", "Reset all mod safe modes to enabled"),
        ),
        settings_common::selection_action_line(
            3,
            state.security_selected,
            settings_common::text("settings.security.reset_enabled", "Reset all mods to disabled"),
        ),
        Line::from(""),
    ];
    let rect = settings_common::render_settings_box(
        frame,
        settings_common::text("settings.security.title", "Security Settings"),
        56,
        lines,
    );
    if state.security_success_at.is_some() {
        settings_common::render_box_success_hint(
            frame,
            rect,
            settings_common::text("settings.security.reset_success", "Reset successful"),
        );
    }
    settings_common::render_box_hint_line(
        frame,
        rect,
        2,
        settings_common::text("settings.security.operation_hint", "[↑]/[↓] Select Option  [Enter] Confirm/Toggle Option"),
    );
    settings_common::render_box_hint_line(
        frame,
        rect,
        3,
        settings_common::text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"),
    );
}

// 渲染关闭安全模式的 10 秒倒计时确认对话框
pub fn render_default_safe_mode_disable_dialog(
    frame: &mut ratatui::Frame<'_>,
    dialog: &DefaultSafeModeDisableDialog,
) {
    use ratatui::widgets::Clear;
    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 10u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let message = settings_common::text(
        "settings.security.default_safe_mode_disable_dialog.message",
        "Are you sure you want to disable Safe Mode by default for all mod packages?\n\nSafe Mode is designed to protect your device. After disabling it, mod packages may perform high-risk operations such as file writes, which may cause data loss or system instability.\n\nPlease make sure you fully trust the source and authors of mod packages.",
    );

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = settings_common::wrap_plain_text_lines(&message, content_width, Style::default().fg(Color::White));
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(settings_common::text("settings.security.default_safe_mode_disable_dialog.cancel", "Cancel"), Style::default().fg(Color::LightGreen)),
    ]));
    let mut confirm_spans = vec![
        Span::styled("[2] ", index_style),
        Span::styled(settings_common::text("settings.security.default_safe_mode_disable_dialog.confirm_disable", "Confirm Disable"), Style::default().fg(if countdown_done { Color::Red } else { Color::DarkGray })),
    ];
    if !countdown_done {
        confirm_spans.push(Span::styled(
            format!(" {}", settings_common::text("settings.security.default_safe_mode_disable_dialog.countdown", "{seconds}s").replace("{seconds}", &remaining.to_string())),
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(confirm_spans));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height.saturating_add(2).clamp(8, area.height.saturating_sub(2).max(8));
    let rect = settings_common::centered_rect(area, width, height);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", settings_common::text("settings.security.default_safe_mode_disable_dialog.title", "Safe Mode")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).alignment(Alignment::Left), inner);
    settings_common::render_box_back_hint(
        frame,
        rect,
        settings_common::text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"),
    );
}

// 处理安全设置按键：上下/数字选择，Enter 确认操作
pub fn handle_security_key(state: &mut SettingsState, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;

    match code {
        KeyCode::Up | KeyCode::Char('k') => state.security_selected = state.security_selected.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => state.security_selected = (state.security_selected + 1).min(3),
        KeyCode::Char('1') => state.security_selected = 0,
        KeyCode::Char('2') => state.security_selected = 1,
        KeyCode::Char('3') => state.security_selected = 2,
        KeyCode::Char('4') => state.security_selected = 3,
        KeyCode::Enter => apply_security_confirm(state),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => state.page = SettingsPage::Hub,
        _ => {}
    }
}

// 执行安全设置确认操作：切换安全模式（弹出确认对话框）/切换默认启用/重置所有安全模式/禁用所有Mod
fn apply_security_confirm(state: &mut SettingsState) {
    match state.security_selected {
        0 => {
            let next = !state.default_safe_mode_enabled;
            if next {
                let _ = crate::mods::set_default_safe_mode_enabled(true);
                state.refresh_security_defaults();
            } else {
                state.default_safe_mode_disable_dialog = Some(DefaultSafeModeDisableDialog {
                    opened_at: Instant::now(),
                });
            }
        }
        1 => {
            let next = !state.default_mod_enabled;
            let _ = crate::mods::set_default_mod_enabled(next);
            state.refresh_security_defaults();
        }
        2 => {
            let _ = crate::mods::reset_all_mod_safe_modes_enabled();
            crate::app::content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
            state.security_success_at = Some(Instant::now());
        }
        3 => {
            let _ = crate::mods::reset_all_mod_enabled_disabled();
            crate::app::content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
            state.security_success_at = Some(Instant::now());
        }
        _ => {}
    }
}

// 处理关闭安全模式对话框的按键
pub fn handle_default_safe_mode_disable_dialog_key(state: &mut SettingsState, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;
    let Some(dialog) = state.default_safe_mode_disable_dialog.as_ref() else { return; };
    let confirm_ready = dialog.opened_at.elapsed().as_secs() >= 10;
    match code {
        KeyCode::Esc | KeyCode::Char('1') | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.default_safe_mode_disable_dialog = None;
        }
        KeyCode::Char('2') | KeyCode::Enter if confirm_ready => {
            state.default_safe_mode_disable_dialog = None;
            let _ = crate::mods::set_default_safe_mode_enabled(false);
            state.refresh_security_defaults();
        }
        _ => {}
    }
}