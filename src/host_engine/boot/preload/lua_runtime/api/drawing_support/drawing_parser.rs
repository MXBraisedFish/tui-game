//! 绘制参数解析

use mlua::{Value, Variadic};

use super::border_chars::BorderChars;
use crate::host_engine::boot::preload::lua_runtime::api::validation::argument;

pub const ALIGN_NO_WRAP: i64 = 0;
pub const ALIGN_LEFT: i64 = 1;
pub const ALIGN_CENTER: i64 = 2;
pub const ALIGN_RIGHT: i64 = 3;

pub const STYLE_BOLD: i64 = 0;
pub const STYLE_ITALIC: i64 = 1;
pub const STYLE_UNDERLINE: i64 = 2;
pub const STYLE_STRIKE: i64 = 3;
pub const STYLE_BLINK: i64 = 4;
pub const STYLE_REVERSE: i64 = 5;
pub const STYLE_HIDDEN: i64 = 6;
pub const STYLE_DIM: i64 = 7;

/// 绘制文本参数。
#[derive(Clone, Debug)]
pub struct DrawTextArgs {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub style: Option<i64>,
    pub align: i64,
}

/// 矩形填充参数。
#[derive(Clone, Debug)]
pub struct FillRectArgs {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub fill_char: char,
    pub fg: Option<String>,
    pub bg: Option<String>,
}

/// 矩形清除参数。
#[derive(Clone, Debug)]
pub struct EraserArgs {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// 边框矩形参数。
#[derive(Clone, Debug)]
pub struct BorderRectArgs {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_chars: BorderChars,
    pub fg: Option<String>,
    pub bg: Option<String>,
}

/// 解析 canvas_draw_text 参数。
pub fn parse_draw_text_args(args: &Variadic<Value>) -> mlua::Result<DrawTextArgs> {
    argument::expect_arg_count_range(args, 3, 7)?;
    let x = parse_coordinate(args, 0)?;
    let y = parse_coordinate(args, 1)?;
    let text = argument::expect_string_arg(args, 2)?;
    let fg = argument::expect_optional_string_arg(args, 3)?;
    let bg = argument::expect_optional_string_arg(args, 4)?;
    let style = argument::expect_optional_i64_arg(args, 5)?;
    let align = match args.get(6) {
        None => ALIGN_LEFT,
        Some(Value::Nil) => ALIGN_NO_WRAP,
        Some(_) => argument::expect_i64_arg(args, 6)?,
    };

    Ok(DrawTextArgs {
        x,
        y,
        text,
        fg,
        bg,
        style,
        align,
    })
}

/// 解析 canvas_fill_rect 参数。
pub fn parse_fill_rect_args(args: &Variadic<Value>) -> mlua::Result<FillRectArgs> {
    argument::expect_arg_count_range(args, 4, 7)?;
    Ok(FillRectArgs {
        x: parse_coordinate(args, 0)?,
        y: parse_coordinate(args, 1)?,
        width: parse_positive_size(args, 2)?,
        height: parse_positive_size(args, 3)?,
        fill_char: parse_optional_char(args.get(4), ' '),
        fg: argument::expect_optional_string_arg(args, 5)?,
        bg: argument::expect_optional_string_arg(args, 6)?,
    })
}

/// 解析 canvas_eraser 参数。
pub fn parse_eraser_args(args: &Variadic<Value>) -> mlua::Result<EraserArgs> {
    argument::expect_exact_arg_count(args, 4)?;
    Ok(EraserArgs {
        x: parse_coordinate(args, 0)?,
        y: parse_coordinate(args, 1)?,
        width: parse_positive_size(args, 2)?,
        height: parse_positive_size(args, 3)?,
    })
}

/// 解析 canvas_border_rect 参数。
pub fn parse_border_rect_args(args: &Variadic<Value>) -> mlua::Result<BorderRectArgs> {
    argument::expect_arg_count_range(args, 4, 7)?;
    let border_chars = match args.get(4) {
        Some(Value::Table(table)) => BorderChars::from_lua_table(table)?,
        _ => BorderChars::default(),
    };

    Ok(BorderRectArgs {
        x: parse_coordinate(args, 0)?,
        y: parse_coordinate(args, 1)?,
        width: parse_positive_size(args, 2)?,
        height: parse_positive_size(args, 3)?,
        border_chars,
        fg: argument::expect_optional_string_arg(args, 5)?,
        bg: argument::expect_optional_string_arg(args, 6)?,
    })
}

fn parse_coordinate(args: &Variadic<Value>, index: usize) -> mlua::Result<u16> {
    let value = argument::expect_i64_arg(args, index)?;
    if value < 0 {
        return Err(mlua::Error::external(
            "coordinate must be greater than or equal to 0",
        ));
    }
    u16::try_from(value).map_err(mlua::Error::external)
}

fn parse_positive_size(args: &Variadic<Value>, index: usize) -> mlua::Result<u16> {
    let value = argument::expect_i64_arg(args, index)?;
    if value <= 0 {
        return Err(mlua::Error::external(
            "width and height must be positive integers",
        ));
    }
    u16::try_from(value).map_err(mlua::Error::external)
}

fn parse_optional_char(value: Option<&Value>, default_char: char) -> char {
    match value {
        Some(Value::String(value)) => value
            .to_str()
            .ok()
            .and_then(|value| value.chars().next())
            .unwrap_or(default_char),
        _ => default_char,
    }
}
