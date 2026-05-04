//! 布局 API 参数解析

use mlua::{Value, Variadic};

use super::layout_anchor::{self, HorizontalAnchor, VerticalAnchor};
use crate::host_engine::boot::preload::lua_runtime::api::validation::argument;

/// resolve_x 参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolveXArgs {
    pub x_anchor: HorizontalAnchor,
    pub width: i64,
    pub offset_x: i64,
}

/// resolve_y 参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolveYArgs {
    pub y_anchor: VerticalAnchor,
    pub height: i64,
    pub offset_y: i64,
}

/// resolve_rect 参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolveRectArgs {
    pub x_anchor: HorizontalAnchor,
    pub y_anchor: VerticalAnchor,
    pub width: i64,
    pub height: i64,
    pub offset_x: i64,
    pub offset_y: i64,
}

/// 解析 resolve_x 参数。
pub fn parse_resolve_x_args(args: &Variadic<Value>) -> mlua::Result<ResolveXArgs> {
    argument::expect_arg_count_range(args, 2, 3)?;
    let x_anchor = layout_anchor::parse_horizontal_anchor(argument::expect_i64_arg(args, 0)?)?;
    let width = argument::expect_i64_arg(args, 1)?;
    ensure_non_negative_width(width)?;
    let offset_x = argument::expect_optional_i64_arg(args, 2)?.unwrap_or(0);

    Ok(ResolveXArgs {
        x_anchor,
        width,
        offset_x,
    })
}

/// 解析 resolve_y 参数。
pub fn parse_resolve_y_args(args: &Variadic<Value>) -> mlua::Result<ResolveYArgs> {
    argument::expect_arg_count_range(args, 2, 3)?;
    let y_anchor = layout_anchor::parse_vertical_anchor(argument::expect_i64_arg(args, 0)?)?;
    let height = argument::expect_i64_arg(args, 1)?;
    ensure_non_negative_height(height)?;
    let offset_y = argument::expect_optional_i64_arg(args, 2)?.unwrap_or(0);

    Ok(ResolveYArgs {
        y_anchor,
        height,
        offset_y,
    })
}

/// 解析 resolve_rect 参数。
pub fn parse_resolve_rect_args(args: &Variadic<Value>) -> mlua::Result<ResolveRectArgs> {
    argument::expect_arg_count_range(args, 4, 6)?;
    let x_anchor = layout_anchor::parse_horizontal_anchor(argument::expect_i64_arg(args, 0)?)?;
    let y_anchor = layout_anchor::parse_vertical_anchor(argument::expect_i64_arg(args, 1)?)?;
    let width = argument::expect_i64_arg(args, 2)?;
    let height = argument::expect_i64_arg(args, 3)?;
    ensure_non_negative_size(width, height)?;
    let offset_x = argument::expect_optional_i64_arg(args, 4)?.unwrap_or(0);
    let offset_y = argument::expect_optional_i64_arg(args, 5)?.unwrap_or(0);

    Ok(ResolveRectArgs {
        x_anchor,
        y_anchor,
        width,
        height,
        offset_x,
        offset_y,
    })
}

fn ensure_non_negative_width(width: i64) -> mlua::Result<()> {
    if width >= 0 {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "invalid width parameter: width must be non-negative, got {width}"
        )))
    }
}

fn ensure_non_negative_height(height: i64) -> mlua::Result<()> {
    if height >= 0 {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "invalid height parameter: height must be non-negative, got {height}"
        )))
    }
}

fn ensure_non_negative_size(width: i64, height: i64) -> mlua::Result<()> {
    if width >= 0 && height >= 0 {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "invalid width/height parameter: width and height must be non-negative, got width {width}, height {height}"
        )))
    }
}
