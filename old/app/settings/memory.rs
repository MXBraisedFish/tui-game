// 内存清理页面，提供清除缓存和清除全部数据两个操作。包含清理确认对话框（带倒计时 3 秒确认）、数据清除的实现逻辑

use std::time::Instant; // 倒计时计时

use ratatui::layout::{Alignment}; // 布局
use ratatui::style::{Color, Style}; // 样式
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::{Block, Borders, Paragraph, Wrap}; // 组件

use crossterm::event::KeyCode; // 按键码

use crate::app::i18n; // 国际化
use crate::app::settings::common as settings_common; // 通用工具
use crate::app::settings::types::*; // 类型定义
use crate::utils::path_utils; // 路径工具

// 返回最小尺寸 (72, 12)
pub fn minimum_size_memory() -> (u16, u16) { (72, 12) }

// 渲染内存清理页面：两个操作项（清除缓存/清除全部数据），底部提示
pub fn render_memory(frame: &mut ratatui::Frame<'_>, state: &SettingsState) {
    let lines = vec![
        Line::from(""),
        settings_common::selection_action_line(0, state.memory_selected, settings_common::text("settings.memory.clear_cache", "Clear cache")),
        settings_common::selection_action_line(1, state.memory_selected, settings_common::text("settings.memory.clear_all", "Clear all data")),
        Line::from(""),
    ];
    let rect = settings_common::render_settings_box(frame, settings_common::text("settings.memory.title", "Memory Cleanup"), 32, lines);
    settings_common::render_box_hint_line(frame, rect, 2, settings_common::text("settings.memory.operation_hint", "[↑]/[↓] Select Option  [Enter] Confirm"));
    settings_common::render_box_hint_line(frame, rect, 3, settings_common::text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"));
}

// 渲染清理确认对话框：标题、说明文字、取消[1] 和确认[2] 选项，未到 3 秒倒计时时确认按钮灰色并显示剩余秒数
pub fn render_cleanup_dialog(frame: &mut ratatui::Frame<'_>, dialog: &CleanupDialog) {
    use ratatui::widgets::Clear;
    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 3u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let (title, question, description) = match dialog.action {
        CleanupAction::ClearCache => (
            settings_common::text("settings.memory.confirm_clear_cache_title", "Clear Cache"),
            settings_common::text("settings.memory.confirm_clear_cache_question", "Confirm clearing cache?"),
            settings_common::text("settings.memory.confirm_clear_cache", "This will clear mod image cache and game save cache. This cannot be undone."),
        ),
        CleanupAction::ClearAllData => (
            settings_common::text("settings.memory.confirm_clear_all_title", "Clear All Storage"),
            settings_common::text("settings.memory.confirm_clear_all_question", "Confirm clearing all storage?"),
            settings_common::text("settings.memory.confirm_clear_all", "This will reset all contents inside tui-game-data. This cannot be undone."),
        ),
    };

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = settings_common::wrap_plain_text_lines(&question, content_width, Style::default().fg(Color::White));
    lines.push(Line::from(""));
    lines.extend(settings_common::wrap_plain_text_lines(&description, content_width, Style::default().fg(Color::White)));
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![Span::styled("[1] ", index_style), Span::styled(settings_common::text("settings.memory.confirm_cancel", "Cancel"), Style::default().fg(Color::LightGreen))]));
    let mut confirm_spans = vec![Span::styled("[2] ", index_style), Span::styled(settings_common::text("settings.memory.confirm_cleanup", "Confirm Cleanup"), Style::default().fg(if countdown_done { Color::Red } else { Color::DarkGray }))];
    if !countdown_done { confirm_spans.push(Span::styled(format!(" {}", settings_common::text("settings.memory.confirm_countdown", "{seconds}s").replace("{seconds}", &remaining.to_string())), Style::default().fg(Color::DarkGray))); }
    lines.push(Line::from(confirm_spans));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height.saturating_add(2).clamp(8, area.height.saturating_sub(2).max(8));
    let rect = settings_common::centered_rect(area, width, height);
    let block = Block::default().borders(Borders::ALL).title(Line::from(Span::styled(format!(" {} ", title), Style::default().fg(Color::White)))).border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).alignment(Alignment::Left), inner);
    settings_common::render_box_back_hint(frame, rect, settings_common::text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"));
}

// 处理内存页按键：数字键选择，Enter 打开确认对话框，Esc/Q 返回 Hub
pub fn handle_memory_key(state: &mut SettingsState, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => state.memory_selected = state.memory_selected.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => state.memory_selected = (state.memory_selected + 1).min(1),
        KeyCode::Char('1') => state.memory_selected = 0,
        KeyCode::Char('2') => state.memory_selected = 1,
        KeyCode::Enter => {
            let action = if state.memory_selected == 0 { CleanupAction::ClearCache } else { CleanupAction::ClearAllData };
            state.cleanup_dialog = Some(CleanupDialog { action, opened_at: Instant::now() });
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => state.page = SettingsPage::Hub,
        _ => {}
    }
}

// 处理确认对话框按键：[1]/Esc/Q 取消，[2]/Enter 在倒计时结束后执行清理并刷新缓存和 Mod
pub fn handle_cleanup_dialog_key(state: &mut SettingsState, code: KeyCode) {
    let Some(dialog) = state.cleanup_dialog.as_ref() else { return; };
    let confirm_ready = dialog.opened_at.elapsed().as_secs() >= 3;
    match code {
        KeyCode::Esc | KeyCode::Char('1') | KeyCode::Char('q') | KeyCode::Char('Q') => state.cleanup_dialog = None,
        KeyCode::Enter | KeyCode::Char('2') if confirm_ready => {
            let action = dialog.action;
            state.cleanup_dialog = None;
            let _ = match action {
                CleanupAction::ClearCache => clear_cached_data(),
                CleanupAction::ClearAllData => clear_all_runtime_data(),
            };
            crate::app::content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
        }
        _ => {}
    }
}

// 清除缓存：删除 cache 目录和 mod_save 目录内容，清除游戏调试日志，重置 saves.json
fn clear_cached_data() -> anyhow::Result<()> {
    clear_directory_contents(&path_utils::cache_dir()?)?;
    clear_directory_contents(&path_utils::mod_save_dir()?)?;
    clear_game_debug_logs()?;
    std::fs::write(path_utils::saves_file()?, "{\n  \"continue\": {},\n  \"data\": {}\n}\n")?;
    Ok(())
}

// 清除全部数据：删除整个 app_data 目录内容，重新创建目录结构和默认文件
fn clear_all_runtime_data() -> anyhow::Result<()> {
    let app_data = path_utils::app_data_dir()?;
    clear_directory_contents(&app_data)?;
    std::fs::create_dir_all(app_data.join("official"))?;
    std::fs::create_dir_all(app_data.join("mod"))?;
    std::fs::create_dir_all(app_data.join("cache"))?;
    std::fs::create_dir_all(app_data.join("mod_save"))?;
    std::fs::create_dir_all(app_data.join("log"))?;
    std::fs::write(path_utils::language_file()?, format!("{}\n", i18n::current_language_code()))?;
    std::fs::write(path_utils::best_scores_file()?, "{}\n")?;
    std::fs::write(path_utils::saves_file()?, "{\n  \"continue\": {},\n  \"data\": {}\n}\n")?;
    std::fs::write(path_utils::updater_cache_file()?, "{}\n")?;
    Ok(())
}

// 递归清除目录内所有文件和子目录
fn clear_directory_contents(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() { return Ok(()); }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let target = entry.path();
        if target.is_dir() { std::fs::remove_dir_all(target)?; } else { std::fs::remove_file(target)?; }
    }
    Ok(())
}

// 清除游戏调试日志，但保留宿主日志 tui_log.txt
fn clear_game_debug_logs() -> anyhow::Result<()> {
    let log_dir = path_utils::log_dir()?;
    if !log_dir.exists() { return Ok(()); }
    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let target = entry.path();
        if target.is_file() && target.file_name().and_then(|name| name.to_str()).map(|name| name != "tui_log.txt").unwrap_or(false) { std::fs::remove_file(target)?; }
    }
    Ok(())
}