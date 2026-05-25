//! 布局锚点常量和解析

pub const ANCHOR_LEFT: i64 = 0;
pub const ANCHOR_CENTER: i64 = 1;
pub const ANCHOR_RIGHT: i64 = 2;

pub const ANCHOR_TOP: i64 = 0;
pub const ANCHOR_MIDDLE: i64 = 1;
pub const ANCHOR_BOTTOM: i64 = 2;

/// 水平锚点。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

/// 垂直锚点。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalAnchor {
    Top,
    Middle,
    Bottom,
}

/// 解析水平锚点。
pub fn parse_horizontal_anchor(value: i64) -> mlua::Result<HorizontalAnchor> {
    match value {
        ANCHOR_LEFT => Ok(HorizontalAnchor::Left),
        ANCHOR_CENTER => Ok(HorizontalAnchor::Center),
        ANCHOR_RIGHT => Ok(HorizontalAnchor::Right),
        _ => Err(mlua::Error::external("invalid anchor parameter")),
    }
}

/// 解析垂直锚点。
pub fn parse_vertical_anchor(value: i64) -> mlua::Result<VerticalAnchor> {
    match value {
        ANCHOR_TOP => Ok(VerticalAnchor::Top),
        ANCHOR_MIDDLE => Ok(VerticalAnchor::Middle),
        ANCHOR_BOTTOM => Ok(VerticalAnchor::Bottom),
        _ => Err(mlua::Error::external("invalid anchor parameter")),
    }
}
