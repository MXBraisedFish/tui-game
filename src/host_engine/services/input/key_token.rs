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

    "left_ctrl" => Some(Key::LeftCtrl),
    "right_ctrl" => Some(Key::RightCtrl),
    "left_shift" => Some(Key::LeftShift),
    "right_shift" => Some(Key::RightShift),
    "left_alt" => Some(Key::LeftAlt),
    "right_alt" => Some(Key::RightAlt),
    "left_meta" => Some(Key::LeftMeta),
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

    _ => {
      parse_letter(&token)
        .or_else(|| parse_number(&token))
        .or_else(|| parse_function_key(&token))
        .or_else(|| parse_numpad_number(&token))
        .or_else(|| parse_unknown_key(&token))
    }
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
    Key::A => "a".to_string(),
    Key::B => "b".to_string(),
    Key::C => "c".to_string(),
    Key::D => "d".to_string(),
    Key::E => "e".to_string(),
    Key::F => "f".to_string(),
    Key::G => "g".to_string(),
    Key::H => "h".to_string(),
    Key::I => "i".to_string(),
    Key::J => "j".to_string(),
    Key::K => "k".to_string(),
    Key::L => "l".to_string(),
    Key::M => "m".to_string(),
    Key::N => "n".to_string(),
    Key::O => "o".to_string(),
    Key::P => "p".to_string(),
    Key::Q => "q".to_string(),
    Key::R => "r".to_string(),
    Key::S => "s".to_string(),
    Key::T => "t".to_string(),
    Key::U => "u".to_string(),
    Key::V => "v".to_string(),
    Key::W => "w".to_string(),
    Key::X => "x".to_string(),
    Key::Y => "y".to_string(),
    Key::Z => "z".to_string(),
    Key::LeftCtrl => "LCtrl".to_string(),
    Key::RightCtrl => "RCtrl".to_string(),
    Key::LeftShift => "LShift".to_string(),
    Key::RightShift => "RShift".to_string(),
    Key::LeftAlt => "LAlt".to_string(),
    Key::RightAlt => "RAlt".to_string(),
    Key::LeftMeta => "LMeta".to_string(),
    Key::RightMeta => "RMeta".to_string(),
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
