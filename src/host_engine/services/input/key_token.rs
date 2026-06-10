use super::service::Key;

pub fn parse_key_token(token: &str) -> Option<Key> {
  let token = token.trim().to_ascii_lowercase();

  match token.as_str() {
    "esc" => Some(Key::Esc),

    "enter" => Some(Key::Enter),
    "tab" => Some(Key::Tab),
    "backspace" => Some(Key::Backspace),
    "space" => Some(Key::Space),

    "up" => Some(Key::Up),
    "down" => Some(Key::Down),
    "left" => Some(Key::Left),
    "right" => Some(Key::Right),

    "home" => Some(Key::Home),
    "end" => Some(Key::End),
    "pageup" => Some(Key::PageUp),
    "pagedown" => Some(Key::PageDown),
    "ins" => Some(Key::Insert),
    "del" => Some(Key::Delete),

    "`" => Some(Key::BackQuote),
    "-" => Some(Key::Minus),
    "=" => Some(Key::Equal),
    "[" => Some(Key::LeftBracket),
    "]" => Some(Key::RightBracket),
    "\\" => Some(Key::BackSlash),
    ";" => Some(Key::Semicolon),
    "'" => Some(Key::Quote),
    "," => Some(Key::Comma),
    "." => Some(Key::Dot),
    "/" => Some(Key::Slash),

    "left_ctrl" | "ctrl" => Some(Key::LeftCtrl),
    "right_ctrl" => Some(Key::RightCtrl),
    "left_shift" | "shift" => Some(Key::LeftShift),
    "right_shift" => Some(Key::RightShift),
    "left_alt" | "alt" => Some(Key::LeftAlt),
    "right_alt" => Some(Key::RightAlt),
    "left_meta" | "meta" => Some(Key::LeftMeta),
    "right_meta" => Some(Key::RightMeta),

    "capslock" => Some(Key::CapsLock),
    "numlock" => Some(Key::NumLock),
    "scrolllock" => Some(Key::ScrollLock),

    "printscreen" => Some(Key::PrintScreen),
    "pause" => Some(Key::Pause),

    "k+" => Some(Key::NumpadAdd),
    "k-" => Some(Key::NumpadSubtract),
    "k*" => Some(Key::NumpadMultiply),
    "k/" => Some(Key::NumpadDivide),
    "kenter" => Some(Key::NumpadEnter),
    "kdel" => Some(Key::NumpadDelete),

    _ => parse_letter(&token)
      .or_else(|| parse_number(&token))
      .or_else(|| parse_function_key(&token))
      .or_else(|| parse_numpad_number(&token))
      .or_else(|| parse_unknown_key(&token)),
  }
}

fn parse_letter(token: &str) -> Option<Key> {
  match token {
    "a" => Some(Key::A),
    "b" => Some(Key::B),
    "c" => Some(Key::C),
    "d" => Some(Key::D),
    "e" => Some(Key::E),
    "f" => Some(Key::F),
    "g" => Some(Key::G),
    "h" => Some(Key::H),
    "i" => Some(Key::I),
    "j" => Some(Key::J),
    "k" => Some(Key::K),
    "l" => Some(Key::L),
    "m" => Some(Key::M),
    "n" => Some(Key::N),
    "o" => Some(Key::O),
    "p" => Some(Key::P),
    "q" => Some(Key::Q),
    "r" => Some(Key::R),
    "s" => Some(Key::S),
    "t" => Some(Key::T),
    "u" => Some(Key::U),
    "v" => Some(Key::V),
    "w" => Some(Key::W),
    "x" => Some(Key::X),
    "y" => Some(Key::Y),
    "z" => Some(Key::Z),
    _ => None,
  }
}

fn parse_number(token: &str) -> Option<Key> {
  let number = token.parse::<u8>().ok()?;
  if number <= 9 {
    Some(Key::Num(number))
  } else {
    None
  }
}

fn parse_function_key(token: &str) -> Option<Key> {
  let value = token.strip_prefix('f')?;
  let number = value.parse::<u8>().ok()?;
  if (1..=12).contains(&number) {
    Some(Key::Fn(number))
  } else {
    None
  }
}

fn parse_numpad_number(token: &str) -> Option<Key> {
  let value = token.strip_prefix('k')?;
  let number = value.parse::<u8>().ok()?;
  if number <= 9 {
    Some(Key::Numpad(number))
  } else {
    None
  }
}

fn parse_unknown_key(token: &str) -> Option<Key> {
  let value = token.strip_prefix("key(")?.strip_suffix(')')?;
  let code = value.parse::<u32>().ok()?;
  Some(Key::Unknown(code))
}

/// 将原始按键配置格式化为人类可读的显示字符串。
///
/// 每个 pattern 内部 key 按固定优先级排序（保证 `[Shift + D]` 而非 `[D + Shift]`），
/// 再调用 `display_key_token` 显示，格式化后包裹在 `[...]` 中，
/// 多个 pattern 之间用 `/` 分隔。
///
/// 示例：
/// - `[["shift"]]` → `"[Shift]"`
/// - `[["d"], ["left", "shift"]]` → `"[D]/[← + Shift]"`
/// - `[["d", "shift"]]` → `"[Shift + D]"`（Shift 优先级高于字母）
pub fn format_key_display(patterns: &[Vec<String>]) -> String {
  patterns
    .iter()
    .map(|pattern| {
      let mut keys: Vec<Key> = pattern
        .iter()
        .filter_map(|token| parse_key_token(token))
        .collect();
      // 按显示优先级排序
      keys.sort_by(|a, b| key_display_order(a).cmp(&key_display_order(b)));
      let display: Vec<String> = keys.iter().map(|k| display_key_token(*k)).collect();
      if display.is_empty() {
        // 全部 token 解析失败，回退到原始文本
        pattern.join(" + ")
      } else {
        display.join(" + ")
      }
    })
    .map(|s| format!("[{}]", s))
    .collect::<Vec<_>>()
    .join("/")
}

/// 按键显示优先级。值越小越靠前。
fn key_display_order(key: &Key) -> u8 {
  match key {
    // 1. 功能键
    Key::LeftCtrl | Key::RightCtrl => 0,
    Key::LeftShift | Key::RightShift => 1,
    Key::LeftAlt | Key::RightAlt => 2,
    Key::LeftMeta | Key::RightMeta => 3,
    // 2. 字母
    Key::A | Key::B | Key::C | Key::D | Key::E | Key::F | Key::G | Key::H | Key::I
    | Key::J | Key::K | Key::L | Key::M | Key::N | Key::O | Key::P | Key::Q | Key::R
    | Key::S | Key::T | Key::U | Key::V | Key::W | Key::X | Key::Y | Key::Z => 10,
    // 3. 数字
    Key::Num(_) => 20,
    // 4. 小键盘数字
    Key::Numpad(_) => 30,
    // 5. 其它符号
    Key::BackQuote
    | Key::Minus
    | Key::Equal
    | Key::LeftBracket
    | Key::RightBracket
    | Key::BackSlash
    | Key::Semicolon
    | Key::Quote
    | Key::Comma
    | Key::Dot
    | Key::Slash => 40,
    // 6. 小键盘其它符号
    Key::NumpadAdd
    | Key::NumpadSubtract
    | Key::NumpadMultiply
    | Key::NumpadDivide
    | Key::NumpadEnter
    | Key::NumpadDelete => 50,
    // 7. 未知键 & 其它
    _ => 60,
  }
}

pub fn display_key_token(key: Key) -> String {
  match key {
    Key::Esc => "Esc".to_string(),
    Key::Enter => "Enter".to_string(),
    Key::Tab => "Tab".to_string(),
    Key::Backspace => "Bksp".to_string(),
    Key::Space => "Space".to_string(),
    Key::Up => "\u{2191}".to_string(),
    Key::Down => "\u{2193}".to_string(),
    Key::Left => "\u{2190}".to_string(),
    Key::Right => "\u{2192}".to_string(),
    Key::Home => "Home".to_string(),
    Key::End => "End".to_string(),
    Key::PageUp => "PgUp".to_string(),
    Key::PageDown => "PgDn".to_string(),
    Key::Insert => "Ins".to_string(),
    Key::Delete => "Del".to_string(),
    Key::Fn(number) => format!("F{}", number),
    Key::Num(number) => number.to_string(),
    Key::Numpad(number) => format!("K{}", number),
    Key::A => "a".to_uppercase().to_string(),
    Key::B => "b".to_uppercase().to_string(),
    Key::C => "c".to_uppercase().to_string(),
    Key::D => "d".to_uppercase().to_string(),
    Key::E => "e".to_uppercase().to_string(),
    Key::F => "f".to_uppercase().to_string(),
    Key::G => "g".to_uppercase().to_string(),
    Key::H => "h".to_uppercase().to_string(),
    Key::I => "i".to_uppercase().to_string(),
    Key::J => "j".to_uppercase().to_string(),
    Key::K => "k".to_uppercase().to_string(),
    Key::L => "l".to_uppercase().to_string(),
    Key::M => "m".to_uppercase().to_string(),
    Key::N => "n".to_uppercase().to_string(),
    Key::O => "o".to_uppercase().to_string(),
    Key::P => "p".to_uppercase().to_string(),
    Key::Q => "q".to_uppercase().to_string(),
    Key::R => "r".to_uppercase().to_string(),
    Key::S => "s".to_uppercase().to_string(),
    Key::T => "t".to_uppercase().to_string(),
    Key::U => "u".to_uppercase().to_string(),
    Key::V => "v".to_uppercase().to_string(),
    Key::W => "w".to_uppercase().to_string(),
    Key::X => "x".to_uppercase().to_string(),
    Key::Y => "y".to_uppercase().to_string(),
    Key::Z => "z".to_uppercase().to_string(),
    Key::LeftCtrl | Key::RightCtrl => "Ctrl".to_string(),
    Key::LeftShift | Key::RightShift => "Shift".to_string(),
    Key::LeftAlt | Key::RightAlt => "Alt".to_string(),
    Key::LeftMeta | Key::RightMeta => "Meta".to_string(),
    Key::CapsLock => "Caps".to_string(),
    Key::NumLock => "Num".to_string(),
    Key::ScrollLock => "Scrl".to_string(),
    Key::PrintScreen => "Prtsc".to_string(),
    Key::Pause => "Pause".to_string(),
    Key::BackQuote => "`".to_string(),
    Key::Minus => "-".to_string(),
    Key::Equal => "=".to_string(),
    Key::LeftBracket => "[".to_string(),
    Key::RightBracket => "]".to_string(),
    Key::BackSlash => "\\".to_string(),
    Key::Semicolon => ";".to_string(),
    Key::Quote => "'".to_string(),
    Key::Comma => ",".to_string(),
    Key::Dot => ".".to_string(),
    Key::Slash => "/".to_string(),
    Key::NumpadAdd => "K+".to_string(),
    Key::NumpadSubtract => "K-".to_string(),
    Key::NumpadMultiply => "K*".to_string(),
    Key::NumpadDivide => "K/".to_string(),
    Key::NumpadEnter => "KEnter".to_string(),
    Key::NumpadDelete => "KDel".to_string(),
    Key::Unknown(code) => format!("key({})", code),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn single_key() {
    assert_eq!(format_key_display(&[vec!["shift".into()]]), "[Shift]");
    assert_eq!(format_key_display(&[vec!["d".into()]]), "[D]");
    assert_eq!(format_key_display(&[vec!["enter".into()]]), "[Enter]");
  }

  #[test]
  fn combo_sorted_modifier_first() {
    // d + shift → Shift 优先级 > 字母，所以 [Shift + D]
    assert_eq!(
      format_key_display(&[vec!["d".into(), "shift".into()]]),
      "[Shift + D]"
    );
    // shift + d（参数顺序无所谓，内部排序）
    assert_eq!(
      format_key_display(&[vec!["shift".into(), "d".into()]]),
      "[Shift + D]"
    );
  }

  #[test]
  fn multi_pattern() {
    assert_eq!(
      format_key_display(&[vec!["d".into()], vec!["left".into(), "shift".into()]]),
      "[D]/[Shift + ←]"
    );
  }

  #[test]
  fn empty_patterns() {
    assert_eq!(format_key_display(&[]), "");
  }

  #[test]
  fn unknown_token_fallback() {
    // 未知 token 回退到原始文本
    assert_eq!(
      format_key_display(&[vec!["not_a_real_key".into()]]),
      "[not_a_real_key]"
    );
  }

  #[test]
  fn arrow_keys() {
    assert_eq!(format_key_display(&[vec!["up".into()]]), "[↑]");
    assert_eq!(format_key_display(&[vec!["left".into()]]), "[←]");
  }
}
