//! 绘制操作

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::canvas_state::{CanvasCell, CanvasState};
use super::drawing_parser::{
    ALIGN_CENTER, ALIGN_LEFT, ALIGN_RIGHT, BorderRectArgs, DrawRichTextArgs, DrawTextArgs,
    EraserArgs, FillRectArgs, WrapLimit, WrapOptions,
};
use super::rich_text_parser::{StyledCharacter, parse_rich_text};
use crate::host_engine::boot::preload::lua_runtime::LuaRuntimeContext;
use crate::host_engine::boot::preload::lua_runtime::api::text_support::text_wrapping;

/// 执行文本绘制。
pub fn draw_text(canvas_state: &mut CanvasState, args: DrawTextArgs) {
    let wrap_options = drawing_wrap_options(canvas_state, args.x, args.y, &args.wrap_options);
    let lines = text_lines(args.text.as_str(), args.align, &wrap_options);
    let first_line_width = lines
        .first()
        .map(|line| UnicodeWidthStr::width(line.as_str()) as i64)
        .unwrap_or_default();

    for (line_index, line) in lines.iter().enumerate() {
        let line_width = UnicodeWidthStr::width(line.as_str()) as i64;
        let x = match args.align {
            ALIGN_CENTER => i64::from(args.x) + ((first_line_width - line_width) / 2),
            ALIGN_RIGHT => i64::from(args.x) + first_line_width - line_width,
            ALIGN_LEFT => i64::from(args.x),
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
    let wrap_options = drawing_wrap_options(canvas_state, args.x, args.y, &args.wrap_options);
    let lines = rich_text_lines(&characters, args.align, &wrap_options, runtime_context)?;
    let first_line_width = lines
        .first()
        .map(|line| rich_line_width(line))
        .unwrap_or_default();

    for (line_index, line) in lines.iter().enumerate() {
        let line_width = rich_line_width(line);
        let x = match args.align {
            ALIGN_CENTER => i64::from(args.x) + ((first_line_width - line_width) / 2),
            ALIGN_RIGHT => i64::from(args.x) + first_line_width - line_width,
            ALIGN_LEFT => i64::from(args.x),
            _ => i64::from(args.x),
        };
        let y = i64::from(args.y) + line_index as i64;
        draw_styled_line(
            canvas_state,
            x,
            y,
            line,
            args.fg.clone(),
            args.bg.clone(),
            args.styles.clone(),
        );
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

fn text_lines(text: &str, _align: i64, wrap_options: &WrapOptions) -> Vec<String> {
    if wrap_options.wrap_width == WrapLimit::Disabled {
        text_wrapping::no_wrap_line(text)
    } else {
        text_wrapping::wrap_text_lines_with_options(text, wrap_options)
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
    _align: i64,
    wrap_options: &WrapOptions,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<Vec<Vec<StyledCharacter>>> {
    let mut lines = if wrap_options.wrap_width == WrapLimit::Disabled {
        no_wrap_rich_lines(characters)
    } else {
        wrap_rich_lines(characters, wrap_options.wrap_width)
    };
    apply_rich_line_limit(&mut lines, wrap_options, runtime_context)?;
    Ok(lines)
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

fn wrap_rich_lines(
    characters: &[StyledCharacter],
    wrap_width: WrapLimit,
) -> Vec<Vec<StyledCharacter>> {
    let WrapLimit::Fixed(wrap_width) = wrap_width else {
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
    default_styles: Vec<i64>,
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
                styles: if character.style_explicit {
                    character.styles.clone()
                } else {
                    default_styles.clone()
                },
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
                        styles: if character.style_explicit {
                            character.styles.clone()
                        } else {
                            default_styles.clone()
                        },
                        is_continuation: true,
                    },
                );
            }
        }

        x += character_width as i64;
    }
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
        replace_rich_tail_by_char_count(last_line, overflow_characters);
    }
    Ok(())
}

fn drawing_wrap_options(
    canvas_state: &CanvasState,
    x: u16,
    y: u16,
    wrap_options: &WrapOptions,
) -> WrapOptions {
    let window_width = canvas_state.width().saturating_sub(x);
    let window_height = canvas_state.height().saturating_sub(y);
    wrap_options.resolved(window_width, window_height)
}

fn replace_rich_tail_by_char_count(
    line: &mut Vec<StyledCharacter>,
    overflow_characters: Vec<StyledCharacter>,
) {
    let overflow_count = overflow_characters.len();
    if overflow_count == 0 {
        return;
    }
    let keep_count = line.len().saturating_sub(overflow_count);
    let style_template = line
        .get(keep_count.saturating_sub(1))
        .or_else(|| line.first())
        .cloned();
    line.truncate(keep_count);
    line.extend(
        overflow_characters
            .into_iter()
            .map(|character| inherit_overflow_style(character, style_template.as_ref())),
    );
}

fn inherit_overflow_style(
    mut character: StyledCharacter,
    style_template: Option<&StyledCharacter>,
) -> StyledCharacter {
    let Some(style_template) = style_template else {
        return character;
    };
    if character.fg.is_none() {
        character.fg = style_template.fg.clone();
    }
    if character.bg.is_none() {
        character.bg = style_template.bg.clone();
    }
    if !character.style_explicit {
        character.styles = style_template.styles.clone();
        character.style_explicit = style_template.style_explicit;
    }
    character
}
