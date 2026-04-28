// 主菜单页面的布局计算，提供通用居中矩形工具和主菜单三个区域（Logo、菜单列表、版本信息）的定位

use ratatui::layout::{Constraint, Direction, Layout, Rect}; // 布局约束和矩形

pub const MENU_MIN_WIDTH: u16 = 60; // 主菜单最小终端宽度
pub const MENU_MIN_HEIGHT: u16 = 15; // 主菜单最小终端高度
pub const MAIN_CONTENT_WIDTH: u16 = 72; // 主菜单理想内容宽度
pub const MENU_LIST_WIDTH: u16 = 30; // 菜单列表区域的理想宽度

// 主菜单各区域的布局结果
pub struct MainMenuAreas {
    pub logo: Rect,
    pub menu: Rect,
    pub version: Rect,
}

// 计算主菜单的布局：整个内容区居中（72x15），纵向分为 logo(6) / 间距(1) / 菜单(5) / 版本(2) / 间距(1)，菜单区域再横向居中（宽度 30）
pub fn main_menu_areas(area: Rect) -> MainMenuAreas {
    let content = centered_rect(
        area,
        MAIN_CONTENT_WIDTH.min(area.width),
        15.min(area.height),
    );
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(content);

    let menu_width = MENU_LIST_WIDTH.min(rows[2].width);
    let menu_x = rows[2].x + rows[2].width.saturating_sub(menu_width) / 2;

    MainMenuAreas {
        logo: rows[0],
        menu: Rect {
            x: menu_x,
            y: rows[2].y,
            width: menu_width,
            height: rows[2].height,
        },
        version: rows[4],
    }
}

// 在父区域内部创建水平和垂直都居中的子矩形
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width.min(area.width)),
            Constraint::Min(0),
        ])
        .split(area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(horizontal[1]);

    vertical[1]
}
