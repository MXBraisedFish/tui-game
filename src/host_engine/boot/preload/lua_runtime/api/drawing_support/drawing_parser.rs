//! 绘制参数解析

use mlua::{Table, Value, Variadic};

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
    pub styles: Vec<i64>,
    pub align: i64,
    pub wrap_width: Option<u16>,
}

/// 绘制富文本参数。
#[derive(Clone, Debug)]
pub struct DrawRichTextArgs {
    pub x: u16,
    pub y: u16,
    pub rich_text: String,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub align: i64,
    pub wrap_width: Option<u16>,
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
    argument::expect_arg_count_range(args, 3, 8)?;
    let x = parse_coordinate(args, 0)?;
    let y = parse_coordinate(args, 1)?;
    let text = argument::expect_string_arg(args, 2)?;
    let fg = argument::expect_optional_string_arg(args, 3)?;
    let bg = argument::expect_optional_string_arg(args, 4)?;
    let styles = parse_optional_styles(args, 5)?;
    let align = match args.get(6) {
        None => ALIGN_LEFT,
        Some(Value::Nil) => ALIGN_NO_WRAP,
        Some(_) => argument::expect_i64_arg(args, 6)?,
    };
    let wrap_width = parse_optional_wrap_width(args, 7)?;

    Ok(DrawTextArgs {
        x,
        y,
        text,
        fg,
        bg,
        styles,
        align,
        wrap_width,
    })
}

/// 解析 canvas_draw_rich_text 参数。
pub fn parse_draw_rich_text_args(args: &Variadic<Value>) -> mlua::Result<DrawRichTextArgs> {
    argument::expect_arg_count_range(args, 3, 7)?;
    let x = parse_coordinate(args, 0)?;
    let y = parse_coordinate(args, 1)?;
    let rich_text = argument::expect_string_arg(args, 2)?;
    let fg = argument::expect_optional_string_arg(args, 3)?;
    let bg = argument::expect_optional_string_arg(args, 4)?;
    let align = match args.get(5) {
        None => ALIGN_LEFT,
        Some(Value::Nil) => ALIGN_NO_WRAP,
        Some(_) => argument::expect_i64_arg(args, 5)?,
    };
    let wrap_width = parse_optional_wrap_width(args, 6)?;

    Ok(DrawRichTextArgs {
        x,
        y,
        rich_text,
        fg,
        bg,
        align,
        wrap_width,
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

pub fn parse_optional_wrap_width(
    args: &Variadic<Value>,
    index: usize,
) -> mlua::Result<Option<u16>> {
    let Some(value) = argument::expect_optional_i64_arg(args, index)? else {
        return Ok(None);
    };
    if value == 0 {
        return Ok(None);
    }
    if value < 0 {
        return Err(mlua::Error::external(
            "wrap_width must be nil, 0, or a positive integer",
        ));
    }
    Ok(Some(u16::try_from(value).map_err(mlua::Error::external)?))
}

fn parse_optional_styles(args: &Variadic<Value>, index: usize) -> mlua::Result<Vec<i64>> {
    match args.get(index) {
        Some(Value::Nil) | None => Ok(Vec::new()),
        Some(Value::Integer(style)) => parse_style_list([*style]),
        Some(Value::Number(style)) => parse_style_list([*style as i64]),
        Some(Value::Table(table)) => parse_style_table(table),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected integer or table, got {}",
            style_lua_type_name(value)
        ))),
    }
}

fn parse_style_table(table: &Table) -> mlua::Result<Vec<i64>> {
    let mut styles = Vec::new();
    for value in table.clone().sequence_values::<Value>() {
        let style = match value? {
            Value::Integer(style) => style,
            Value::Number(style) => style as i64,
            value => {
                return Err(mlua::Error::external(format!(
                    "invalid text style value type: {}",
                    style_lua_type_name(&value)
                )));
            }
        };
        push_unique_style(&mut styles, style)?;
    }
    Ok(styles)
}

fn parse_style_list<const LENGTH: usize>(styles: [i64; LENGTH]) -> mlua::Result<Vec<i64>> {
    let mut parsed_styles = Vec::new();
    for style in styles {
        push_unique_style(&mut parsed_styles, style)?;
    }
    Ok(parsed_styles)
}

fn push_unique_style(styles: &mut Vec<i64>, style: i64) -> mlua::Result<()> {
    if !(STYLE_BOLD..=STYLE_DIM).contains(&style) {
        return Err(mlua::Error::external(format!(
            "invalid text style value: {style}"
        )));
    }
    if !styles.contains(&style) {
        styles.push(style);
    }
    Ok(())
}

fn style_lua_type_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "light_userdata",
        Value::Integer(_) => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        Value::Error(_) => "error",
        Value::Other(_) => "other",
    }
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
