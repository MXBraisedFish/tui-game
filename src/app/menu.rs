use crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::app::i18n::t;
use crate::app::layout;

pub const LOGO_ASCII: &str = r#"████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗
╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝
   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  
   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  
   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗
   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 主菜单可触发的高层动作。
pub enum MenuAction {
    Play,
    Continue,
    Settings,
    About,
    Quit,
}

#[derive(Clone, Debug)]
/// 单个主菜单项的数据描述。
pub struct MenuItem {
    pub key: &'static str,
    pub shortcut: KeyCode,
    pub action: MenuAction,
}

#[derive(Clone, Debug)]
/// 主菜单状态。
///
/// 保存菜单项列表、当前选中项以及“继续游戏”关联的存档信息。
pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
    continue_game_id: Option<String>,
    continue_game_name: Option<String>,
}

impl Menu {
    /// 创建默认主菜单，初始化各项菜单项及其快捷键。
    pub fn new() -> Self {
        Self {
            items: vec![
                MenuItem {
                    key: "menu.play",
                    shortcut: KeyCode::Char('1'),
                    action: MenuAction::Play,
                },
                MenuItem {
                    key: "menu.continue",
                    shortcut: KeyCode::Char('2'),
                    action: MenuAction::Continue,
                },
                MenuItem {
                    key: "menu.settings",
                    shortcut: KeyCode::Char('3'),
                    action: MenuAction::Settings,
                },
                MenuItem {
                    key: "menu.about",
                    shortcut: KeyCode::Char('4'),
                    action: MenuAction::About,
                },
                MenuItem {
                    key: "menu.quit",
                    shortcut: KeyCode::Esc,
                    action: MenuAction::Quit,
                },
            ],
            selected: 0,
            continue_game_id: None,
            continue_game_name: None,
        }
    }

    /// 返回当前菜单的全部菜单项。
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// 返回当前选中的菜单项下标。
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// 在下标合法时更新当前选中项。
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// 根据快捷键选中对应菜单项，匹配成功时返回 `true`。
    pub fn select_by_shortcut(&mut self, code: KeyCode) -> bool {
        if let Some(index) = self.items.iter().position(|item| item.shortcut == code) {
            self.selected = index;
            return true;
        }
        false
    }

    /// 选中下一个菜单项，超出末尾时循环到开头。
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.items.len();
    }

    /// 选中上一个菜单项，位于开头时循环到末尾。
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }

    /// 返回当前选中菜单项对应的高层动作。
    pub fn selected_action(&self) -> Option<MenuAction> {
        self.items.get(self.selected).map(|it| it.action)
    }

    /// 更新“继续游戏”对应的目标游戏信息。
    pub fn set_continue_target(&mut self, game_id: Option<String>, game_name: Option<String>) {
        self.continue_game_id = game_id;
        self.continue_game_name = game_name;
    }

    /// 判断“继续游戏”当前是否存在有效存档目标。
    pub fn can_continue(&self) -> bool {
        self.continue_game_id.is_some()
    }

    /// 返回当前“继续游戏”关联的游戏 ID。
    pub fn continue_game_id(&self) -> Option<&str> {
        self.continue_game_id.as_deref()
    }
}

/// 渲染主菜单界面，包括 Logo、菜单项和版本提示。
pub fn render_main_menu(
    frame: &mut ratatui::Frame<'_>,
    menu: &Menu,
    version: &str,
    update_hint: Option<&str>,
) {
    let areas = layout::main_menu_areas(frame.area());

    let logo_lines: Vec<Line<'_>> = LOGO_ASCII
        .lines()
        .map(|line| {
            let spans = line
                .chars()
                .map(|ch| {
                    let fg = if ch == '█' {
                        Color::Rgb(255, 165, 0)
                    } else {
                        Color::White
                    };
                    Span::styled(
                        ch.to_string(),
                        Style::default().fg(fg).add_modifier(Modifier::BOLD),
                    )
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect();
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, areas.logo);

    let enter_shortcut = t("menu.enter_shortcut");
    let content_width = menu
        .items()
        .iter()
        .map(|item| {
            let shortcut = match item.shortcut {
                KeyCode::Char(c) => format!("[{}]", c),
                KeyCode::Esc => "[ESC]".to_string(),
                _ => "[?]".to_string(),
            };
            let shortcut_width = UnicodeWidthStr::width(shortcut.as_str());
            let enter_width = UnicodeWidthStr::width(enter_shortcut.as_str());
            let effective_shortcut = if enter_width > shortcut_width {
                enter_shortcut.to_string()
            } else {
                shortcut
            };
            let content = format!("\u{25B6} {} {}", effective_shortcut, menu_item_label(menu, item));
            UnicodeWidthStr::width(content.as_str()) as u16
        })
        .max()
        .unwrap_or(0);
    let max_menu_width = frame.area().width.saturating_sub(2).max(1);
    let desired_menu_width = max_menu_width;
    let menu_area = Rect {
        x: frame.area().x + 1,
        y: areas.menu.y,
        width: desired_menu_width,
        height: areas.menu.height,
    };
    let left_pad = menu_area.width.saturating_sub(content_width) / 2;

    let mut lines = Vec::new();
    for (idx, item) in menu.items().iter().enumerate() {
        let selected = idx == menu.selected();
        let disabled_continue = item.action == MenuAction::Continue && !menu.can_continue();
        let base_style = if disabled_continue {
            Style::default().fg(Color::DarkGray)
        } else if selected {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let key_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(if selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });

        let raw_shortcut = match item.shortcut {
            KeyCode::Char(c) => format!("[{}]", c),
            KeyCode::Esc => "[ESC]".to_string(),
            _ => "[?]".to_string(),
        };
        let shortcut = if selected {
            t("menu.enter_shortcut")
        } else {
            raw_shortcut
        };

        let prefix = if selected { "▶ " } else { "  " };
        let left_spaces = " ".repeat(left_pad as usize);

        lines.push(Line::from(vec![
            Span::raw(left_spaces),
            Span::styled(prefix, base_style),
            Span::styled(shortcut, key_style),
            Span::styled(format!(" {}", menu_item_label(menu, item)), base_style),
        ]));
    }

    let menu_widget = Paragraph::new(lines);
    frame.render_widget(menu_widget, menu_area);

    let mut version_spans = vec![Span::styled(
        format!("v{}", version),
        Style::default().fg(Color::DarkGray),
    )];
    if update_hint.is_some() {
        version_spans.push(Span::styled(
            t("menu.version_update_hint"),
            Style::default().fg(Color::LightMagenta),
        ));
    }
    let version_line = Paragraph::new(Line::from(version_spans)).alignment(Alignment::Center);
    frame.render_widget(version_line, areas.version);
}

fn menu_item_label(menu: &Menu, item: &MenuItem) -> String {
    if item.action == MenuAction::Continue {
        if let Some(name) = &menu.continue_game_name {
            return format!("{}-{}", t(item.key), name);
        }
    }
    t(item.key)
}
