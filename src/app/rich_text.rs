use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

use crate::app::i18n;

#[derive(Clone)]
struct StyledChar {
    ch: char,
    style: Style,
}

#[derive(Clone)]
struct StyleState {
    default_fg: Option<Color>,
    default_bg: Option<Color>,
    fg: Option<Color>,
    bg: Option<Color>,
    fg_count: Option<usize>,
    bg_count: Option<usize>,
    fg_need_clear: bool,
    bg_need_clear: bool,
}

/// 解析可选的 `f%` 富文本语法，并按指定宽度包装为 ratatui 可渲染的文本行。
///
/// 当前支持的指令：
/// - `{tc:<颜色>}` / `{tc:clear}` / `{tc:<颜色>><数量>}`：控制文字颜色。
/// - `{bg:<颜色>}` / `{bg:clear}` / `{bg:<颜色>><数量>}`：控制背景颜色。
pub fn parse_rich_text_wrapped(text: &str, width: usize, base: Style) -> Vec<Line<'static>> {
    let content = text.strip_prefix("f%").unwrap_or(text);

    let mut state = StyleState {
        default_fg: base.fg,
        default_bg: base.bg,
        fg: base.fg,
        bg: base.bg,
        fg_count: None,
        bg_count: None,
        fg_need_clear: false,
        bg_need_clear: false,
    };

    let mut out: Vec<StyledChar> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '\\' {
            if i + 1 < chars.len() {
                let next = chars[i + 1];
                if next == 'n' {
                    push_char(&mut out, '\n', &mut state, base);
                } else {
                    push_char(&mut out, next, &mut state, base);
                }
                i += 2;
            } else {
                push_char(&mut out, '\\', &mut state, base);
                i += 1;
            }
            continue;
        }

        if ch == '{' {
            if let Some((block, consumed)) = read_block(&chars[i..]) {
                if block.trim().is_empty() {
                    push_error(&mut out, &rt("rich_text.error.empty_command"), base);
                    reset_to_default(&mut state);
                    i += consumed;
                    continue;
                }

                let rest = &chars[i + consumed..];
                match apply_block(&block, &mut state, rest) {
                    Ok(()) => {}
                    Err(msg) => {
                        push_error(&mut out, &msg, base);
                        reset_to_default(&mut state);
                    }
                }

                i += consumed;
                continue;
            }

            push_error(&mut out, &rt("rich_text.error.unclosed_command"), base);
            reset_to_default(&mut state);
            i += 1;
            continue;
        }

        if ch == '}' {
            push_error(&mut out, &rt("rich_text.error.unclosed_command"), base);
            reset_to_default(&mut state);
            i += 1;
            continue;
        }

        push_char(&mut out, ch, &mut state, base);
        i += 1;
    }

    if state.fg_need_clear || state.bg_need_clear {
        push_error(&mut out, &rt("rich_text.error.unterminated_style"), base);
        reset_to_default(&mut state);
    }

    styled_chars_to_lines(&out, width.max(1), base)
}

fn reset_to_default(state: &mut StyleState) {
    state.fg = state.default_fg;
    state.bg = state.default_bg;
    state.fg_count = None;
    state.bg_count = None;
    state.fg_need_clear = false;
    state.bg_need_clear = false;
}

fn read_block(input: &[char]) -> Option<(String, usize)> {
    if input.first().copied() != Some('{') {
        return None;
    }

    let mut escape = false;
    let mut i = 1usize;
    while i < input.len() {
        let ch = input[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if ch == '\\' {
            escape = true;
            i += 1;
            continue;
        }
        if ch == '}' {
            let block: String = input[1..i].iter().collect();
            return Some((block, i + 1));
        }
        i += 1;
    }
    None
}

fn split_unescaped(input: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == sep {
            out.push(cur.trim().to_string());
            cur.clear();
            continue;
        }
        cur.push(ch);
    }

    if escape {
        cur.push('\\');
    }
    out.push(cur.trim().to_string());
    out
}

fn apply_block(block: &str, state: &mut StyleState, rest: &[char]) -> Result<(), String> {
    let commands = split_unescaped(block, '|');
    if commands.is_empty() {
        return Err(rt("rich_text.error.empty_command"));
    }

    for command in commands {
        if command.trim().is_empty() {
            return Err(rt("rich_text.error.empty_command"));
        }
        let pair = split_unescaped(&command, ':');
        if pair.len() != 2 {
            return Err(rt("rich_text.error.invalid_param"));
        }

        let cmd = pair[0].trim().to_ascii_lowercase();
        let params = split_unescaped(&pair[1], '>');

        match cmd.as_str() {
            "tc" => apply_color_command(params, true, state, rest)?,
            "bg" => apply_color_command(params, false, state, rest)?,
            _ => return Err(rt("rich_text.error.invalid_command")),
        }
    }

    Ok(())
}

fn apply_color_command(
    params: Vec<String>,
    is_fg: bool,
    state: &mut StyleState,
    rest: &[char],
) -> Result<(), String> {
    if params.is_empty() || params[0].is_empty() {
        return Err(rt("rich_text.error.invalid_param"));
    }

    let cmd_name = if is_fg { "tc" } else { "bg" };

    if params[0].eq_ignore_ascii_case("clear") {
        if params.len() != 1 {
            return Err(rt("rich_text.error.invalid_param"));
        }
        if is_fg {
            state.fg = state.default_fg;
            state.fg_count = None;
            state.fg_need_clear = false;
        } else {
            state.bg = state.default_bg;
            state.bg_count = None;
            state.bg_need_clear = false;
        }
        return Ok(());
    }

    let Some(color) = parse_color(&params[0]) else {
        return Err(rt("rich_text.error.invalid_param"));
    };

    let count = if params.len() >= 2 && !params[1].trim().is_empty() {
        match params[1].trim().parse::<usize>() {
            Ok(v) if v > 0 => Some(v),
            _ => return Err(rt("rich_text.error.invalid_param")),
        }
    } else {
        None
    };

    if params.len() > 2 {
        return Err(rt("rich_text.error.invalid_param"));
    }

    if count.is_none() && !has_future_clear(rest, cmd_name) {
        return Err(rt("rich_text.error.unterminated_style"));
    }

    if is_fg {
        state.fg = Some(color);
        state.fg_count = count;
        state.fg_need_clear = count.is_none();
    } else {
        state.bg = Some(color);
        state.bg_count = count;
        state.bg_need_clear = count.is_none();
    }

    Ok(())
}

fn has_future_clear(rest: &[char], cmd: &str) -> bool {
    let mut i = 0usize;
    while i < rest.len() {
        if rest[i] == '\\' {
            i += 2;
            continue;
        }
        if rest[i] == '{' {
            if let Some((block, consumed)) = read_block(&rest[i..]) {
                for command in split_unescaped(&block, '|') {
                    let pair = split_unescaped(&command, ':');
                    if pair.len() != 2 {
                        continue;
                    }
                    if pair[0].trim().eq_ignore_ascii_case(cmd) {
                        let params = split_unescaped(&pair[1], '>');
                        if params.len() == 1 && params[0].eq_ignore_ascii_case("clear") {
                            return true;
                        }
                    }
                }
                i += consumed;
                continue;
            }
        }
        i += 1;
    }
    false
}

fn push_char(out: &mut Vec<StyledChar>, ch: char, state: &mut StyleState, base: Style) {
    let mut style = base;
    style.fg = state.fg;
    style.bg = state.bg;
    out.push(StyledChar { ch, style });

    if let Some(rem) = state.fg_count {
        if rem <= 1 {
            state.fg_count = None;
            state.fg = state.default_fg;
        } else {
            state.fg_count = Some(rem - 1);
        }
    }

    if let Some(rem) = state.bg_count {
        if rem <= 1 {
            state.bg_count = None;
            state.bg = state.default_bg;
        } else {
            state.bg_count = Some(rem - 1);
        }
    }
}

fn rt(key: &str) -> String {
    i18n::t(key).to_string()
}

fn push_error(out: &mut Vec<StyledChar>, msg: &str, base: Style) {
    let mut style = base;
    style.fg = Some(Color::Red);
    style.bg = base.bg;
    for ch in format!("{{{msg}}}").chars() {
        out.push(StyledChar { ch, style });
    }
}

fn styled_chars_to_lines(chars: &[StyledChar], width: usize, base: Style) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut segment: Vec<StyledChar> = Vec::new();

    for item in chars {
        if item.ch == '\n' {
            if segment.is_empty() {
                lines.push(Line::default());
            } else {
                wrap_segment_wordwise(&segment, width, base, &mut lines);
                segment.clear();
            }
            continue;
        }
        segment.push(item.clone());
    }

    if !segment.is_empty() {
        wrap_segment_wordwise(&segment, width, base, &mut lines);
    }

    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines
}

fn wrap_segment_wordwise(segment: &[StyledChar], width: usize, base: Style, out: &mut Vec<Line<'static>>) {
    let mut remaining: Vec<StyledChar> = segment.to_vec();
    let width = width.max(1);

    while !remaining.is_empty() {
        let total_w = remaining.iter().map(|c| UnicodeWidthChar::width(c.ch).unwrap_or(0)).sum::<usize>();
        if total_w <= width {
            out.push(build_line(&remaining, base));
            break;
        }

        let mut cur_w = 0usize;
        let mut limit = 0usize;
        for (idx, ch) in remaining.iter().enumerate() {
            let w = UnicodeWidthChar::width(ch.ch).unwrap_or(0);
            if cur_w + w > width {
                break;
            }
            cur_w += w;
            limit = idx + 1;
        }

        if limit == 0 {
            limit = 1;
        }

        let mut break_at = None;
        for i in (0..limit).rev() {
            if remaining[i].ch.is_whitespace() {
                break_at = Some(i);
                break;
            }
        }

        let cut = match break_at {
            Some(i) if i > 0 => i,
            _ => limit,
        };

        let mut head = remaining[..cut].to_vec();
        while head.last().map(|v| v.ch.is_whitespace()).unwrap_or(false) {
            head.pop();
        }
        out.push(build_line(&head, base));

        let mut next = if break_at.is_some() {
            remaining[cut + 1..].to_vec()
        } else {
            remaining[cut..].to_vec()
        };
        while next.first().map(|v| v.ch.is_whitespace()).unwrap_or(false) {
            next.remove(0);
        }
        remaining = next;
    }
}

fn build_line(chars: &[StyledChar], base: Style) -> Line<'static> {
    if chars.is_empty() {
        return Line::default();
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cur = String::new();
    let mut style = base;

    for item in chars {
        if cur.is_empty() {
            style = item.style;
        } else if style != item.style {
            spans.push(Span::styled(std::mem::take(&mut cur), style));
            style = item.style;
        }
        cur.push(item.ch);
    }

    if !cur.is_empty() {
        spans.push(Span::styled(cur, style));
    }

    Line::from(spans)
}



fn parse_color(raw: &str) -> Option<Color> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }

    if let Some(c) = parse_hex_color(text) {
        return Some(c);
    }
    if let Some(c) = parse_rgb_color(text) {
        return Some(c);
    }

    match text.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "dark_gray" | "dark_grey" => Some(Color::DarkGray),
        "light_red" => Some(Color::LightRed),
        "light_green" => Some(Color::LightGreen),
        "light_yellow" => Some(Color::LightYellow),
        "light_blue" => Some(Color::LightBlue),
        "light_magenta" => Some(Color::LightMagenta),
        "light_cyan" => Some(Color::LightCyan),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<Color> {
    if !raw.starts_with('#') {
        return None;
    }

    if raw.len() == 4 {
        let r = u8::from_str_radix(&raw[1..2], 16).ok()?;
        let g = u8::from_str_radix(&raw[2..3], 16).ok()?;
        let b = u8::from_str_radix(&raw[3..4], 16).ok()?;
        return Some(Color::Rgb(r * 17, g * 17, b * 17));
    }

    if raw.len() == 7 {
        let r = u8::from_str_radix(&raw[1..3], 16).ok()?;
        let g = u8::from_str_radix(&raw[3..5], 16).ok()?;
        let b = u8::from_str_radix(&raw[5..7], 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }

    None
}

fn parse_rgb_color(raw: &str) -> Option<Color> {
    let lower = raw.to_ascii_lowercase();
    if !lower.starts_with("rgb(") || !lower.ends_with(')') {
        return None;
    }

    let inner = &lower[4..lower.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|v| v.trim()).collect();
    if parts.len() != 3 {
        return None;
    }

    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    Some(Color::Rgb(r, g, b))
}
