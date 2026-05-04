//! 布局定位计算

use super::layout_anchor::{HorizontalAnchor, VerticalAnchor};
use super::layout_parser::{ResolveRectArgs, ResolveXArgs, ResolveYArgs};

/// 计算 X 起始坐标。
pub fn resolve_x(terminal_width: i64, args: ResolveXArgs) -> i64 {
    resolve_horizontal(args.x_anchor, terminal_width, args.width) + args.offset_x
}

/// 计算 Y 起始坐标。
pub fn resolve_y(terminal_height: i64, args: ResolveYArgs) -> i64 {
    resolve_vertical(args.y_anchor, terminal_height, args.height) + args.offset_y
}

/// 计算矩形起始坐标。
pub fn resolve_rect(
    terminal_width: i64,
    terminal_height: i64,
    args: ResolveRectArgs,
) -> (i64, i64) {
    let x = resolve_horizontal(args.x_anchor, terminal_width, args.width) + args.offset_x;
    let y = resolve_vertical(args.y_anchor, terminal_height, args.height) + args.offset_y;
    (x, y)
}

fn resolve_horizontal(anchor: HorizontalAnchor, terminal_width: i64, width: i64) -> i64 {
    match anchor {
        HorizontalAnchor::Left => 0,
        HorizontalAnchor::Center => (terminal_width - width) / 2,
        HorizontalAnchor::Right => terminal_width - width,
    }
}

fn resolve_vertical(anchor: VerticalAnchor, terminal_height: i64, height: i64) -> i64 {
    match anchor {
        VerticalAnchor::Top => 0,
        VerticalAnchor::Middle => (terminal_height - height) / 2,
        VerticalAnchor::Bottom => terminal_height - height,
    }
}
