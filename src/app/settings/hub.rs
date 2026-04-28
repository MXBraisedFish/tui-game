// 设置中心的 Hub（导航）页面，提供语言、Mod 管理、安全设置、按键绑定、内存清理五个子页面的入口。用户通过方向键或数字键选择并进入

use ratatui::layout::{Alignment, Rect}; // 对齐布局和矩形区域
use ratatui::style::{Color, Modifier, Style}; // 样式控制
use ratatui::text::{Line, Span}; // 富文本
use ratatui::widgets::Paragraph; // 段落渲染
use unicode_width::UnicodeWidthStr; // 计算文本宽度

use crate::app::i18n; // 国际化
use crate::app::settings::common as settings_common; // 通用工具
use crate::app::settings::types::*; // 类型定义
use crate::app::settings::SettingsAction; // 设置页面动作枚举

/// 计算 Hub 页面的最小终端尺寸，需容纳 5 个菜单项的宽度
pub fn minimum_size_hub() -> (u16, u16) {
    let label_lang = settings_common::text("settings.hub.language", "Language");
    let label_mods = settings_common::text("settings.hub.mods", "Mods");
    let label_security = settings_common::text("settings.hub.security", "Security");
    let label_keybind = settings_common::text("settings.hub.keybind", "Keybinding");
    let label_memory = settings_common::text("settings.hub.memory", "Memory Cleanup");
    let enter_key = i18n::t("menu.enter_shortcut");
    let back_hint = settings_common::text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu");

    let widths = [
        UnicodeWidthStr::width(format!("{}[1] {}", settings_common::TRIANGLE, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", settings_common::TRIANGLE, enter_key, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}[2] {}", settings_common::TRIANGLE, label_mods).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", settings_common::TRIANGLE, enter_key, label_mods).as_str()),
        UnicodeWidthStr::width(format!("{}[3] {}", settings_common::TRIANGLE, label_security).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", settings_common::TRIANGLE, enter_key, label_security).as_str()),
        UnicodeWidthStr::width(format!("{}[4] {}", settings_common::TRIANGLE, label_keybind).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", settings_common::TRIANGLE, enter_key, label_keybind).as_str()),
        UnicodeWidthStr::width(format!("{}[5] {}", settings_common::TRIANGLE, label_memory).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", settings_common::TRIANGLE, enter_key, label_memory).as_str()),
        UnicodeWidthStr::width(back_hint.as_str()),
    ];

    let max_width = widths.into_iter().max().unwrap_or(30) as u16;
    (max_width + 4, 14)
}

// 渲染 Hub 页面：5 个入口项（Language/Mods/Security/Keybinding/Memory Cleanup），选中项高亮并显示 Enter 提示，底部有操作提示
pub fn render_hub(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let items = [
        ("[1]", settings_common::text("settings.hub.language", "Language")),
        ("[2]", settings_common::text("settings.hub.mods", "Mods")),
        ("[3]", settings_common::text("settings.hub.security", "Security")),
        ("[4]", settings_common::text("settings.hub.keybind", "Keybinding")),
        ("[5]", settings_common::text("settings.hub.memory", "Memory Cleanup")),
    ];
    let enter_hint = i18n::t("menu.enter_shortcut");

    let content_width = items
        .iter()
        .map(|(shortcut, text)| {
            let normal = format!("{}{} {}", settings_common::TRIANGLE, shortcut, text);
            let enter = format!("{}{} {}", settings_common::TRIANGLE, enter_hint, text);
            UnicodeWidthStr::width(normal.as_str()).max(UnicodeWidthStr::width(enter.as_str()))
        })
        .max()
        .unwrap_or(1) as u16;
    let operation_hint_width = UnicodeWidthStr::width(
        settings_common::text(
            "settings.hub.operation_hint",
            "[↑]/[↓] Select Option  [Enter] Confirm",
        )
        .as_str(),
    ) as u16;
    let back_hint_width = UnicodeWidthStr::width(
        settings_common::text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu").as_str(),
    ) as u16;

    let width = area
        .width
        .saturating_sub(2)
        .max(1)
        .min(content_width.max(operation_hint_width).max(back_hint_width).max(1));
    let height = (items.len() + 3) as u16;
    let menu_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    let item_area = Rect {
        x: menu_area.x,
        y: menu_area.y,
        width: menu_area.width,
        height: items.len() as u16,
    };
    let hint_area = Rect {
        x: menu_area.x,
        y: menu_area.y + items.len() as u16 + 1,
        width: menu_area.width,
        height: 2,
    };

    let left_pad = item_area.width.saturating_sub(content_width) / 2;
    let mut item_lines = Vec::new();

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

        item_lines.push(Line::from(vec![
            Span::raw(" ".repeat(left_pad as usize)),
            Span::styled(if is_selected { settings_common::TRIANGLE } else { "  " }, base_style),
            Span::styled(key.to_string(), key_style),
            Span::styled(format!(" {}", text), base_style),
        ]));
    }

    let item_widget = Paragraph::new(item_lines).alignment(Alignment::Left);
    frame.render_widget(item_widget, item_area);

    let hint_lines = vec![
        Line::from(Span::styled(
            settings_common::text(
                "settings.hub.operation_hint",
                "[↑]/[↓] Select Option  [Enter] Confirm",
            ),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            settings_common::text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu"),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let hint_widget = Paragraph::new(hint_lines).alignment(Alignment::Center);
    frame.render_widget(hint_widget, hint_area);
}

// 处理 Hub 页面的按键事件：上下/数字键选择，Enter 进入子页面并初始化对应页面
pub fn handle_hub_key(state: &mut SettingsState, code: crossterm::event::KeyCode) -> SettingsAction {
    use crossterm::event::KeyCode;

    let item_count = 5;
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.hub_selected = state.hub_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.hub_selected = (state.hub_selected + 1).min(item_count - 1);
        }
        KeyCode::Char('1') => state.hub_selected = 0,
        KeyCode::Char('2') => state.hub_selected = 1,
        KeyCode::Char('3') => state.hub_selected = 2,
        KeyCode::Char('4') => state.hub_selected = 3,
        KeyCode::Char('5') => state.hub_selected = 4,
        KeyCode::Enter => match state.hub_selected {
            0 => {
                state.page = SettingsPage::Language;
                state.lang_selected = crate::app::settings::default_selected_index();
            }
            1 => {
                state.page = SettingsPage::Mods;
                state.refresh_mods();
            }
            2 => {
                state.page = SettingsPage::Security;
                state.refresh_security_defaults();
            }
            3 => {
                state.page = SettingsPage::Keybind;
                state.refresh_keybind_games();
            }
            4 => {
                state.page = SettingsPage::Memory;
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