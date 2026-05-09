//! 富文本尺寸计算

use unicode_width::UnicodeWidthChar;

use crate::host_engine::boot::preload::lua_runtime::LuaRuntimeContext;
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
    let characters = parse_rich_text(rich_text, runtime_context)?;
    if characters.is_empty() {
        return Ok(TextSize {
            width: 0,
            height: 0,
        });
    }

    let lines = rich_text_lines(&characters, wrap_width);
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
    wrap_width: Option<u16>,
) -> Vec<Vec<StyledCharacter>> {
    let Some(wrap_width) = wrap_width.filter(|wrap_width| *wrap_width > 0) else {
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

fn no_wrap_rich_lines(characters: &[StyledCharacter]) -> Vec<Vec<StyledCharacter>> {
    let mut lines = vec![Vec::new()];
    for character in characters {
        if character.character == '\n' {
            lines.push(Vec::new());
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
