// 自定义富文本解析器，支持 f% 前缀的格式化文本语法。提供颜色、背景、文字样式的内联控制，支持按键占位符替换，最终将解析后的带样式字符按指定宽度自动换行，输出 Vec<Line> 供 ratatui 渲染

use ratatui::style::{Color, Modifier, Style}; // 颜色、修饰、样式类型
use ratatui::text::{Line, Span}; // 输出行和片段
use unicode_width::UnicodeWidthChar; // 字符宽度计算（用于换行）

use crate::app::i18n; // 错误消息的国际化

// 按键绑定查询模式
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyBindingMode {
    User,
    Original,
}

// 解析后的带样式字符
#[derive(Clone)]
struct StyledChar {
    ch: char,
    style: Style,
}

// 当前样式状态机
#[derive(Clone)]
struct StyleState {
    default_fg: Option<Color>,
    default_bg: Option<Color>,
    fg: Option<Color>,
    bg: Option<Color>,
    modifiers: Modifier,
    fg_count: Option<usize>,
    bg_count: Option<usize>,
    modifier_count: Option<usize>,
    fg_need_clear: bool,
    bg_need_clear: bool,
    modifier_need_clear: bool,
}

// 解析富文本，不处理按键替换，按指定宽度换行
pub fn parse_rich_text_wrapped(text: &str, width: usize, base: Style) -> Vec<Line<'static>> {
    parse_rich_text_wrapped_with_keys(text, width, base, |_, _| None)
}

// 解析富文本，支持 {key:...} 按键占位符替换，按宽度换行
pub fn parse_rich_text_wrapped_with_keys<F>(
    text: &str,
    width: usize,
    base: Style,
    key_resolver: F,
) -> Vec<Line<'static>>
where
    F: Fn(&str, KeyBindingMode) -> Option<Vec<String>>,
{
    let content = text.strip_prefix("f%").unwrap_or(text);
    let content = replace_key_commands(content, &key_resolver);

    let mut state = StyleState {
        default_fg: base.fg,
        default_bg: base.bg,
        fg: base.fg,
        bg: base.bg,
        modifiers: Modifier::empty(),
        fg_count: None,
        bg_count: None,
        modifier_count: None,
        fg_need_clear: false,
        bg_need_clear: false,
        modifier_need_clear: false,
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

    if state.fg_need_clear || state.bg_need_clear || state.modifier_need_clear {
        push_error(&mut out, &rt("rich_text.error.unterminated_style"), base);
        reset_to_default(&mut state);
    }

    styled_chars_to_lines(&out, width.max(1), base)
}

// 在全文中查找 {key:...} 块并替换为实际键名
fn replace_key_commands<F>(content: &str, key_resolver: &F) -> String
where
    F: Fn(&str, KeyBindingMode) -> Option<Vec<String>>,
{
    let chars: Vec<char> = content.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] == '\\' {
            if i + 1 < chars.len() {
                out.push(chars[i]);
                out.push(chars[i + 1]);
                i += 2;
            } else {
                out.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if chars[i] == '{'
            && let Some((block, consumed)) = read_block(&chars[i..])
            && let Some(replacement) = replace_key_commands_in_block(&block, key_resolver)
        {
            out.push_str(&replacement);
            i += consumed;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

// 在花括号块中分离样式指令和按键指令，按键指令替换为文本
fn replace_key_commands_in_block<F>(block: &str, key_resolver: &F) -> Option<String>
where
    F: Fn(&str, KeyBindingMode) -> Option<Vec<String>>,
{
    let mut style_commands = Vec::new();
    let mut replacements = Vec::new();

    for command in split_unescaped(block, '|') {
        if command.trim().is_empty() {
            style_commands.push(command);
            continue;
        }

        if let Some(replacement) = resolve_key_command(&command, key_resolver) {
            replacements.push(replacement);
        } else {
            style_commands.push(command);
        }
    }

    if replacements.is_empty() {
        return None;
    }

    let mut out = String::new();
    if !style_commands.is_empty() {
        out.push('{');
        out.push_str(&style_commands.join("|"));
        out.push('}');
    }
    out.push_str(&replacements.join(""));
    Some(out)
}

// 解析单个 key:语义键名>模式 命令
fn resolve_key_command<F>(block: &str, key_resolver: &F) -> Option<String>
where
    F: Fn(&str, KeyBindingMode) -> Option<Vec<String>>,
{
    let pair = split_unescaped(block, ':');
    if pair.len() != 2 || !pair[0].trim().eq_ignore_ascii_case("key") {
        return None;
    }

    let params = split_unescaped(&pair[1], '>');
    if params.is_empty() || params[0].trim().is_empty() || params.len() > 2 {
        return None;
    }

    let mode = match params.get(1).map(|value| value.trim().to_ascii_lowercase()) {
        None => KeyBindingMode::User,
        Some(value) if value.is_empty() || value == "user" => KeyBindingMode::User,
        Some(value) if value == "original" => KeyBindingMode::Original,
        Some(_) => return None,
    };

    let keys = key_resolver(params[0].trim(), mode)?;
    Some(format_key_list(&keys))
}

// 将键名列表格式化为 [key1]/[key2]
fn format_key_list(keys: &[String]) -> String {
    keys.iter()
        .filter(|key| !key.trim().is_empty())
        .map(|key| format!("[{}]", escape_rich_replacement(key.trim())))
        .collect::<Vec<_>>()
        .join("/")
}

// 转义富文本中的特殊字符（\、{、}）
fn escape_rich_replacement(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if matches!(ch, '\\' | '{' | '}') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// 将样式状态重置为默认值
fn reset_to_default(state: &mut StyleState) {
    state.fg = state.default_fg;
    state.bg = state.default_bg;
    state.modifiers = Modifier::empty();
    state.fg_count = None;
    state.bg_count = None;
    state.modifier_count = None;
    state.fg_need_clear = false;
    state.bg_need_clear = false;
    state.modifier_need_clear = false;
}

// 从字符数组中读取 {...} 块，处理转义，返回块内容和消费长度
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

// 按分隔符切分字符串，忽略转义的分隔符
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

// 将花括号块中的指令应用到样式状态机
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
            "ts" => apply_text_style_command(params, state, rest)?,
            _ => return Err(rt("rich_text.error.invalid_command")),
        }
    }

    Ok(())
}

// 处理 tc/bg 颜色指令：支持 clear、命名颜色、#RGB、rgb() 和数量限定
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

// 处理 ts 文字样式指令：支持 clear、bold+italic 组合、数量限定
fn apply_text_style_command(
    params: Vec<String>,
    state: &mut StyleState,
    rest: &[char],
) -> Result<(), String> {
    if params.is_empty() || params[0].is_empty() {
        return Err(rt("rich_text.error.invalid_param"));
    }

    if params[0].eq_ignore_ascii_case("clear") {
        if params.len() != 1 {
            return Err(rt("rich_text.error.invalid_param"));
        }
        state.modifiers = Modifier::empty();
        state.modifier_count = None;
        state.modifier_need_clear = false;
        return Ok(());
    }

    let Some(modifiers) = parse_text_styles(&params[0]) else {
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

    if count.is_none() && !has_future_clear(rest, "ts") {
        return Err(rt("rich_text.error.unterminated_style"));
    }

    state.modifiers = modifiers;
    state.modifier_count = count;
    state.modifier_need_clear = count.is_none();
    Ok(())
}

// 前瞻搜索后续文本中是否存在对应命令的 clear 指令（防止无限样式蔓延）
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

// 将字符以当前样式状态压入输出，并更新数量计数
fn push_char(out: &mut Vec<StyledChar>, ch: char, state: &mut StyleState, base: Style) {
    let mut style = base;
    style.fg = state.fg;
    style.bg = state.bg;
    if !state.modifiers.is_empty() {
        style = style.add_modifier(state.modifiers);
    }
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

    if let Some(rem) = state.modifier_count {
        if rem <= 1 {
            state.modifier_count = None;
            state.modifiers = Modifier::empty();
        } else {
            state.modifier_count = Some(rem - 1);
        }
    }
}

// 国际化错误消息的便捷函数
fn rt(key: &str) -> String {
    i18n::t(key).to_string()
}

// 以红色样式输出错误标记 {错误消息}
fn push_error(out: &mut Vec<StyledChar>, msg: &str, base: Style) {
    let mut style = base;
    style.fg = Some(Color::Red);
    style.bg = base.bg;
    for ch in format!("{{{msg}}}").chars() {
        out.push(StyledChar { ch, style });
    }
}

// 将 StyledChar 列表转换为 Vec<Line>：遇到 \n 换行，其余按宽度自动换行
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

// 将一段连续的 StyledChar 按宽度切分，优先在空格处换行（词边界）
fn wrap_segment_wordwise(
    segment: &[StyledChar],
    width: usize,
    base: Style,
    out: &mut Vec<Line<'static>>,
) {
    let mut remaining: Vec<StyledChar> = segment.to_vec();
    let width = width.max(1);

    while !remaining.is_empty() {
        let total_w = remaining
            .iter()
            .map(|c| UnicodeWidthChar::width(c.ch).unwrap_or(0))
            .sum::<usize>();
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

// 将 StyledChar 列表构建为单个 Line，合并相同样式的连续字符为 Span
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

// 解析颜色字符串：支持 16 种命名颜色、#RGB/#RRGGBB 十六进制、rgb(r,g,b)
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

// 解析文字样式组合：bold、italic、underline、strike、blink、reverse、hidden、dim，可用 + 连接
fn parse_text_styles(raw: &str) -> Option<Modifier> {
    let mut modifiers = Modifier::empty();
    let mut saw_any = false;

    for token in raw.split('+').map(str::trim).filter(|token| !token.is_empty()) {
        let modifier = match token.to_ascii_lowercase().as_str() {
            "bold" => Modifier::BOLD,
            "italic" => Modifier::ITALIC,
            "underline" => Modifier::UNDERLINED,
            "strike" => Modifier::CROSSED_OUT,
            "blink" => Modifier::SLOW_BLINK,
            "reverse" => Modifier::REVERSED,
            "hidden" => Modifier::HIDDEN,
            "dim" => Modifier::DIM,
            _ => return None,
        };
        modifiers |= modifier;
        saw_any = true;
    }

    if saw_any {
        Some(modifiers)
    } else {
        None
    }
}

// 解析 #RGB（3位）和 #RRGGBB（6位）十六进制颜色
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

// 解析 rgb(r,g,b) 函数格式颜色
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
