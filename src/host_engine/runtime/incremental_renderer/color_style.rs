//! 终端颜色和样式转换

use crossterm::style::{Attribute, Color};

/// 解析颜色字符串。
pub fn parse_color(color: &str) -> Option<Color> {
    if let Some(rgb_color) = parse_hex_color(color) {
        return Some(rgb_color);
    }

    match color.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "red" | "light_red" => Some(Color::Red),
        "dark_red" => Some(Color::DarkRed),
        "yellow" | "light_yellow" => Some(Color::Yellow),
        "dark_yellow" => Some(Color::DarkYellow),
        "orange" => Some(Color::Rgb { r: 255, g: 165, b: 0 }),
        "green" | "light_green" => Some(Color::Green),
        "blue" | "light_blue" => Some(Color::Blue),
        "cyan" | "light_cyan" => Some(Color::Cyan),
        "magenta" | "light_magenta" => Some(Color::Magenta),
        "grey" | "gray" => Some(Color::Grey),
        "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        _ => None,
    }
}

/// 解析样式常量。
pub fn parse_style(style: i64) -> Option<Attribute> {
    match style {
        0 => Some(Attribute::Bold),
        1 => Some(Attribute::Italic),
        2 => Some(Attribute::Underlined),
        3 => Some(Attribute::CrossedOut),
        4 => Some(Attribute::SlowBlink),
        5 => Some(Attribute::Reverse),
        6 => Some(Attribute::Hidden),
        7 => Some(Attribute::Dim),
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
    Some(Color::Rgb { r: red, g: green, b: blue })
}
