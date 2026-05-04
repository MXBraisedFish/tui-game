//! 绘制操作

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::canvas_state::{CanvasCell, CanvasState};
use super::drawing_parser::{
    ALIGN_CENTER, ALIGN_LEFT, ALIGN_NO_WRAP, ALIGN_RIGHT, BorderRectArgs, DrawTextArgs, EraserArgs,
    FillRectArgs,
};

/// 执行文本绘制。
pub fn draw_text(canvas_state: &mut CanvasState, args: DrawTextArgs) {
    let lines = text_lines(args.text.as_str(), args.align);
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
            args.style,
        );
    }
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
                    style: None,
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

fn text_lines(text: &str, align: i64) -> Vec<String> {
    if align == ALIGN_NO_WRAP {
        vec![text.replace('\n', "\\n")]
    } else {
        text.split('\n').map(ToString::to_string).collect()
    }
}

fn draw_line(
    canvas_state: &mut CanvasState,
    start_x: i64,
    y: i64,
    line: &str,
    fg: Option<String>,
    bg: Option<String>,
    style: Option<i64>,
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
                style,
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
                        style,
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
            style: None,
            is_continuation: false,
        },
    );
}
