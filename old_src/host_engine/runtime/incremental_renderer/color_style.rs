//! 终端颜色和样式转换

use crossterm::style::{Attribute, Color};

/// 解析颜色字符串。
pub fn parse_color(color: &str) -> Option<Color> {
    if let Some(rgb_color) = parse_hex_color(color) {
        return Some(rgb_color);
    }
    if let Some(rgb_color) = parse_rgb_color(color) {
        return Some(rgb_color);
    }

    if let Ok(color_id) = color.trim().parse::<i64>() {
        return parse_color_id(color_id);
    }

    match color.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "grey" | "gray" => Some(Color::Grey),
        "dark_red" => Some(Color::DarkRed),
        "dark_green" => Some(Color::DarkGreen),
        "dark_yellow" => Some(Color::DarkYellow),
        "dark_blue" => Some(Color::DarkBlue),
        "dark_magenta" => Some(Color::DarkMagenta),
        "dark_cyan" => Some(Color::DarkCyan),
        "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        _ => None,
    }
}

fn parse_color_id(color_id: i64) -> Option<Color> {
    match color_id {
        0 => Some(Color::Black),
        1 => Some(Color::Red),
        2 => Some(Color::Green),
        3 => Some(Color::Yellow),
        4 => Some(Color::Blue),
        5 => Some(Color::Magenta),
        6 => Some(Color::Cyan),
        7 => Some(Color::White),
        8 => Some(Color::Grey),
        9 => Some(Color::DarkRed),
        10 => Some(Color::DarkGreen),
        11 => Some(Color::DarkYellow),
        12 => Some(Color::DarkBlue),
        13 => Some(Color::DarkMagenta),
        14 => Some(Color::DarkCyan),
        15 => Some(Color::DarkGrey),
        _ => None,
    }
}

/// 解析样式常量。
pub fn parse_style(style: i64) -> Option<Attribute> {
    match style {
        0 => None,
        1 => Some(Attribute::Bold),
        2 => Some(Attribute::Italic),
        3 => Some(Attribute::Underlined),
        4 => Some(Attribute::CrossedOut),
        5 => Some(Attribute::SlowBlink),
        6 => Some(Attribute::Reverse),
        7 => Some(Attribute::Hidden),
        8 => Some(Attribute::Dim),
        _ => None,
    }
}

fn parse_hex_color(color: &str) -> Option<Color> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb {
        r: red,
        g: green,
        b: blue,
    })
}

fn parse_rgb_color(color: &str) -> Option<Color> {
    let body = color.strip_prefix("rgb(")?.strip_suffix(')')?;
    let values = body
        .split(',')
        .map(|value| value.trim().parse::<u8>().ok())
        .collect::<Option<Vec<_>>>()?;
    if values.len() != 3 {
        return None;
    }

    Some(Color::Rgb {
        r: values[0],
        g: values[1],
        b: values[2],
    })
}
