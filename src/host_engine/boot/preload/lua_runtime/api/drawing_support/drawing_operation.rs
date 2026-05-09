//! 绘制操作

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::canvas_state::{CanvasCell, CanvasState};
use super::drawing_parser::{
    ALIGN_CENTER, ALIGN_LEFT, ALIGN_NO_WRAP, ALIGN_RIGHT, BorderRectArgs, DrawRichTextArgs,
    DrawTextArgs, EraserArgs, FillRectArgs,
};
use super::rich_text_parser::{StyledCharacter, parse_rich_text};
use crate::host_engine::boot::preload::lua_runtime::LuaRuntimeContext;
use crate::host_engine::boot::preload::lua_runtime::api::text_support::text_wrapping;

/// 执行文本绘制。
pub fn draw_text(canvas_state: &mut CanvasState, args: DrawTextArgs) {
    let lines = text_lines(args.text.as_str(), args.align, args.wrap_width);
    let first_line_width = lines
        .first()
        .map(|line| UnicodeWidthStr::width(line.as_str()) as i64)
        .unwrap_or_default();

    for (line_index, line) in lines.iter().enumerate() {
        let line_width = UnicodeWidthStr::width(line.as_str()) as i64;
        let x = match args.align {
            ALIGN_CENTER => i64::from(args.x) + ((first_line_width - line_width) / 2),
            ALIGN_RIGHT => i64::from(args.x) + first_line_width - line_width,
            ALIGN_LEFT | ALIGN_NO_WRAP => i64::from(args.x),
            _ => i64::from(args.x),
        };
        let y = i64::from(args.y) + line_index as i64;
        draw_line(
            canvas_state,
            x,
            y,
            line,
            args.fg.clone(),
            args.bg.clone(),
            args.styles.clone(),
        );
    }
}

/// 执行富文本绘制。
pub fn draw_rich_text(
    canvas_state: &mut CanvasState,
    args: DrawRichTextArgs,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<()> {
    let characters = parse_rich_text(args.rich_text.as_str(), runtime_context)?;
    let lines = rich_text_lines(&characters, args.align, args.wrap_width);
    let first_line_width = lines
        .first()
        .map(|line| rich_line_width(line))
        .unwrap_or_default();

    for (line_index, line) in lines.iter().enumerate() {
        let line_width = rich_line_width(line);
        let x = match args.align {
            ALIGN_CENTER => i64::from(args.x) + ((first_line_width - line_width) / 2),
            ALIGN_RIGHT => i64::from(args.x) + first_line_width - line_width,
            ALIGN_LEFT | ALIGN_NO_WRAP => i64::from(args.x),
            _ => i64::from(args.x),
        };
        let y = i64::from(args.y) + line_index as i64;
        draw_styled_line(canvas_state, x, y, line, args.fg.clone(), args.bg.clone());
    }

    Ok(())
}

/// 执行矩形填充。
pub fn fill_rect(canvas_state: &mut CanvasState, args: FillRectArgs) {
    for y in args.y..args.y.saturating_add(args.height) {
        for x in args.x..args.x.saturating_add(args.width) {
            canvas_state.set_cell(
                x,
                y,
                CanvasCell {
                    text: args.fill_char.to_string(),
                    fg: args.fg.clone(),
                    bg: args.bg.clone(),
                    styles: Vec::new(),
                    is_continuation: false,
                },
            );
        }
    }
}

/// 执行区域清除。
pub fn erase_rect(canvas_state: &mut CanvasState, args: EraserArgs) {
    for y in args.y..args.y.saturating_add(args.height) {
        for x in args.x..args.x.saturating_add(args.width) {
            canvas_state.erase_cell(x, y);
        }
    }
}

/// 执行边框绘制。
pub fn border_rect(canvas_state: &mut CanvasState, args: BorderRectArgs) {
    if args.width == 0 || args.height == 0 {
        return;
    }

    let right_x = args.x.saturating_add(args.width.saturating_sub(1));
    let bottom_y = args.y.saturating_add(args.height.saturating_sub(1));

    draw_horizontal_border(
        canvas_state,
        args.x,
        right_x,
        args.y,
        args.border_chars.top,
        &args,
    );
    draw_horizontal_border(
        canvas_state,
        args.x,
        right_x,
        bottom_y,
        args.border_chars.bottom,
        &args,
    );
    draw_vertical_border(
        canvas_state,
        args.y,
        bottom_y,
        args.x,
        args.border_chars.left,
        &args,
    );
    draw_vertical_border(
        canvas_state,
        args.y,
        bottom_y,
        right_x,
        args.border_chars.right,
        &args,
    );

    set_border_cell(
        canvas_state,
        args.x,
        args.y,
        args.border_chars.top_left,
        &args,
    );
    set_border_cell(
        canvas_state,
        right_x,
        args.y,
        args.border_chars.top_right,
        &args,
    );
    set_border_cell(
        canvas_state,
        right_x,
        bottom_y,
        args.border_chars.bottom_right,
        &args,
    );
    set_border_cell(
        canvas_state,
        args.x,
        bottom_y,
        args.border_chars.bottom_left,
        &args,
    );
}

fn text_lines(text: &str, align: i64, wrap_width: Option<u16>) -> Vec<String> {
    if wrap_width.is_none() && align == ALIGN_NO_WRAP {
        text_wrapping::no_wrap_line(text)
    } else {
        text_wrapping::wrap_text_lines(text, wrap_width)
    }
}

fn draw_line(
    canvas_state: &mut CanvasState,
    start_x: i64,
    y: i64,
    line: &str,
    fg: Option<String>,
    bg: Option<String>,
    styles: Vec<i64>,
) {
    if y < 0 {
        return;
    }

    let mut x = start_x;
    for character in line.chars() {
        if x < 0 {
            x += character.width().unwrap_or(1) as i64;
            continue;
        }

        let character_width = character.width().unwrap_or(1).max(1);
        let Ok(cell_x) = u16::try_from(x) else {
            break;
        };
        let Ok(cell_y) = u16::try_from(y) else {
            break;
        };

        canvas_state.set_cell(
            cell_x,
            cell_y,
            CanvasCell {
                text: character.to_string(),
                fg: fg.clone(),
                bg: bg.clone(),
                styles: styles.clone(),
                is_continuation: false,
            },
        );

        if character_width > 1 {
            for offset in 1..character_width {
                let continuation_x = cell_x.saturating_add(offset as u16);
                canvas_state.set_cell(
                    continuation_x,
                    cell_y,
                    CanvasCell {
                        text: String::new(),
                        fg: fg.clone(),
                        bg: bg.clone(),
                        styles: styles.clone(),
                        is_continuation: true,
                    },
                );
            }
        }

        x += character_width as i64;
    }
}

fn draw_horizontal_border(
    canvas_state: &mut CanvasState,
    start_x: u16,
    end_x: u16,
    y: u16,
    character: Option<char>,
    args: &BorderRectArgs,
) {
    let Some(character) = character else {
        return;
    };
    for x in start_x..=end_x {
        set_border_cell(canvas_state, x, y, Some(character), args);
    }
}

fn draw_vertical_border(
    canvas_state: &mut CanvasState,
    start_y: u16,
    end_y: u16,
    x: u16,
    character: Option<char>,
    args: &BorderRectArgs,
) {
    let Some(character) = character else {
        return;
    };
    for y in start_y..=end_y {
        set_border_cell(canvas_state, x, y, Some(character), args);
    }
}

fn set_border_cell(
    canvas_state: &mut CanvasState,
    x: u16,
    y: u16,
    character: Option<char>,
    args: &BorderRectArgs,
) {
    let Some(character) = character else {
        return;
    };
    canvas_state.set_cell(
        x,
        y,
        CanvasCell {
            text: character.to_string(),
            fg: args.fg.clone(),
            bg: args.bg.clone(),
            styles: Vec::new(),
            is_continuation: false,
        },
    );
}

fn rich_text_lines(
    characters: &[StyledCharacter],
    align: i64,
    wrap_width: Option<u16>,
) -> Vec<Vec<StyledCharacter>> {
    if wrap_width.is_none() && align == ALIGN_NO_WRAP {
        return no_wrap_rich_lines(characters);
    }
    wrap_rich_lines(characters, wrap_width)
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

fn wrap_rich_lines(
    characters: &[StyledCharacter],
    wrap_width: Option<u16>,
) -> Vec<Vec<StyledCharacter>> {
    let Some(wrap_width) = wrap_width.filter(|wrap_width| *wrap_width > 0) else {
        return no_wrap_rich_lines(characters);
    };

    let wrap_width = i64::from(wrap_width);
    let mut lines = vec![Vec::new()];
    let mut current_width = 0_i64;

    for character in characters {
        if character.character == '\n' {
            lines.push(Vec::new());
            current_width = 0;
            continue;
        }

        let character_width = character.character.width().unwrap_or(1).max(1) as i64;
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

fn rich_line_width(line: &[StyledCharacter]) -> i64 {
    line.iter()
        .map(|character| character.character.width().unwrap_or(1).max(1) as i64)
        .sum()
}

fn draw_styled_line(
    canvas_state: &mut CanvasState,
    start_x: i64,
    y: i64,
    line: &[StyledCharacter],
    default_fg: Option<String>,
    default_bg: Option<String>,
) {
    if y < 0 {
        return;
    }

    let mut x = start_x;
    for character in line {
        let character_width = character.character.width().unwrap_or(1).max(1);
        if x < 0 {
            x += character_width as i64;
            continue;
        }

        let Ok(cell_x) = u16::try_from(x) else {
            break;
        };
        let Ok(cell_y) = u16::try_from(y) else {
            break;
        };

        canvas_state.set_cell(
            cell_x,
            cell_y,
            CanvasCell {
                text: character.character.to_string(),
                fg: character.fg.clone().or_else(|| default_fg.clone()),
                bg: character.bg.clone().or_else(|| default_bg.clone()),
                styles: character.styles.clone(),
                is_continuation: false,
            },
        );

        if character_width > 1 {
            for offset in 1..character_width {
                canvas_state.set_cell(
                    cell_x.saturating_add(offset as u16),
                    cell_y,
                    CanvasCell {
                        text: String::new(),
                        fg: character.fg.clone().or_else(|| default_fg.clone()),
                        bg: character.bg.clone().or_else(|| default_bg.clone()),
                        styles: character.styles.clone(),
                        is_continuation: true,
                    },
                );
            }
        }

        x += character_width as i64;
    }
}
