//! 富文本尺寸计算

use unicode_width::UnicodeWidthChar;

use crate::host_engine::boot::preload::lua_runtime::LuaRuntimeContext;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::{
    WrapLimit, WrapOptions,
};
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::rich_text_parser::{
    StyledCharacter, parse_rich_text,
};

use super::text_measurement::TextSize;

/// 计算富文本解析后的终端字符宽高。
pub fn measure_rich_text(
    rich_text: &str,
    wrap_width: Option<u16>,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<TextSize> {
    measure_rich_text_with_options(
        rich_text,
        &WrapOptions {
            wrap_width: wrap_width.map_or(WrapLimit::Disabled, WrapLimit::Fixed),
            ..WrapOptions::default()
        },
        runtime_context,
    )
}

/// 按换行配置计算富文本解析后的终端字符宽高。
pub fn measure_rich_text_with_options(
    rich_text: &str,
    wrap_options: &WrapOptions,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<TextSize> {
    let characters = parse_rich_text(rich_text, runtime_context)?;
    if characters.is_empty() {
        return Ok(TextSize {
            width: 0,
            height: 0,
        });
    }

    let lines = rich_text_lines(&characters, wrap_options, runtime_context)?;
    let width = lines
        .iter()
        .map(|line| rich_line_width(line))
        .max()
        .unwrap_or_default();
    let height = lines.len();

    Ok(TextSize {
        width: width.min(usize::from(u16::MAX)) as u16,
        height: height.min(usize::from(u16::MAX)) as u16,
    })
}

fn rich_text_lines(
    characters: &[StyledCharacter],
    wrap_options: &WrapOptions,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<Vec<Vec<StyledCharacter>>> {
    let mut lines = wrap_rich_text_lines(characters, wrap_options.wrap_width);
    apply_rich_line_limit(&mut lines, wrap_options, runtime_context)?;
    Ok(lines)
}

fn wrap_rich_text_lines(
    characters: &[StyledCharacter],
    wrap_width: WrapLimit,
) -> Vec<Vec<StyledCharacter>> {
    let WrapLimit::Fixed(wrap_width) = wrap_width else {
        return no_wrap_rich_lines(characters);
    };

    let wrap_width = usize::from(wrap_width);
    let mut lines = vec![Vec::new()];
    let mut current_width = 0usize;

    for character in characters {
        if character.character == '\n' {
            lines.push(Vec::new());
            current_width = 0;
            continue;
        }

        let character_width = character.character.width().unwrap_or(1).max(1);
        if current_width > 0 && current_width + character_width > wrap_width {
            lines.push(Vec::new());
            current_width = 0;
        }

        if let Some(line) = lines.last_mut() {
            line.push(character.clone());
        }
        current_width += character_width;
    }

    lines
}

fn apply_rich_line_limit(
    lines: &mut Vec<Vec<StyledCharacter>>,
    wrap_options: &WrapOptions,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<()> {
    let WrapLimit::Fixed(wrap_height) = wrap_options.wrap_height else {
        return Ok(());
    };
    let wrap_height = usize::from(wrap_height).max(1);
    if lines.len() <= wrap_height {
        return Ok(());
    }

    lines.truncate(wrap_height);
    let Some(text_overflow) = wrap_options.text_overflow.as_deref() else {
        return Ok(());
    };
    if text_overflow.is_empty() {
        return Ok(());
    }
    if let Some(last_line) = lines.last_mut() {
        let overflow_characters = parse_rich_text(text_overflow, runtime_context)?;
        let overflow_count = overflow_characters.len();
        if overflow_count == 0 {
            return Ok(());
        }
        let keep_count = last_line.len().saturating_sub(overflow_count);
        last_line.truncate(keep_count);
        last_line.extend(overflow_characters);
    }
    Ok(())
}

fn no_wrap_rich_lines(characters: &[StyledCharacter]) -> Vec<Vec<StyledCharacter>> {
    let mut lines = vec![Vec::new()];
    for character in characters {
        if character.character == '\n' {
            let mut character = character.clone();
            character.character = ' ';
            if let Some(line) = lines.last_mut() {
                line.push(character);
            }
        } else if let Some(line) = lines.last_mut() {
            line.push(character.clone());
        }
    }
    lines
}

fn rich_line_width(line: &[StyledCharacter]) -> usize {
    line.iter()
        .map(|character| character.character.width().unwrap_or(1).max(1))
        .sum()
}
