use super::key::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Other target platform variants are constructed by platform-injected tests.
enum KeyDisplayPlatform {
  Windows,
  MacOs,
  Other,
}

fn current_key_display_platform() -> KeyDisplayPlatform {
  #[cfg(target_os = "windows")]
  {
    KeyDisplayPlatform::Windows
  }
  #[cfg(target_os = "macos")]
  {
    KeyDisplayPlatform::MacOs
  }
  #[cfg(not(any(target_os = "windows", target_os = "macos")))]
  {
    KeyDisplayPlatform::Other
  }
}

/// Parses a key token (such as "shift", "a" or "f1") into a [`Key`].
///
/// The token is trimmed and matched case-insensitively; unknown tokens yield `None`.
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

/// Converts any recognized key token into the stable token the input system persists.
pub fn canonical_key_token(token: &str) -> Option<String> {
  parse_key_token(token).map(key_token)
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

/// Formats key patterns as user-readable display text (such as "[LShift + D]/[LCtrl + C]").
pub fn format_key_display(patterns: &[Vec<String>]) -> String {
  format_key_display_for_platform(patterns, current_key_display_platform())
}

fn format_key_display_for_platform(
  patterns: &[Vec<String>],
  platform: KeyDisplayPlatform,
) -> String {
  patterns
    .iter()
    .map(|pattern| {
      let mut keys: Vec<Key> = pattern
        .iter()
        .filter_map(|token| parse_key_token(token))
        .collect();

      keys.sort_by_key(key_display_order);
      let display: Vec<String> = keys
        .iter()
        .map(|key| display_key_token_for_platform(*key, platform))
        .collect();
      if display.is_empty() {
        pattern.join(" + ")
      } else {
        display.join(" + ")
      }
    })
    .map(|s| format!("[{}]", s))
    .collect::<Vec<_>>()
    .join("/")
}

/// Returns the display sort weight of a key: modifiers < letters < digits < numpad digits <
/// symbols < numpad operators < other keys.
fn key_display_order(key: &Key) -> u8 {
  match key {
    Key::LeftCtrl | Key::RightCtrl => 0,
    Key::LeftShift | Key::RightShift => 1,
    Key::LeftAlt | Key::RightAlt => 2,
    Key::LeftMeta | Key::RightMeta => 3,

    Key::A
    | Key::B
    | Key::C
    | Key::D
    | Key::E
    | Key::F
    | Key::G
    | Key::H
    | Key::I
    | Key::J
    | Key::K
    | Key::L
    | Key::M
    | Key::N
    | Key::O
    | Key::P
    | Key::Q
    | Key::R
    | Key::S
    | Key::T
    | Key::U
    | Key::V
    | Key::W
    | Key::X
    | Key::Y
    | Key::Z => 10,

    Key::Num(_) => 20,

    Key::Numpad(_) => 30,

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

    Key::NumpadAdd
    | Key::NumpadSubtract
    | Key::NumpadMultiply
    | Key::NumpadDivide
    | Key::NumpadEnter
    | Key::NumpadDelete => 50,

    _ => 60,
  }
}

/// Converts a [`Key`] into its human-readable display string.
pub fn display_key_token(key: Key) -> String {
  display_key_token_for_platform(key, current_key_display_platform())
}

fn display_key_token_for_platform(key: Key, platform: KeyDisplayPlatform) -> String {
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
    Key::LeftCtrl => "LCtrl".to_string(),
    Key::RightCtrl => "RCtrl".to_string(),
    Key::LeftShift => "LShift".to_string(),
    Key::RightShift => "RShift".to_string(),
    Key::LeftAlt => "LAlt".to_string(),
    Key::RightAlt => "RAlt".to_string(),
    Key::LeftMeta => match platform {
      KeyDisplayPlatform::Windows => "LWin",
      KeyDisplayPlatform::MacOs => "LCmd",
      KeyDisplayPlatform::Other => "LMeta",
    }
    .to_string(),
    Key::RightMeta => match platform {
      KeyDisplayPlatform::Windows => "RWin",
      KeyDisplayPlatform::MacOs => "RCmd",
      KeyDisplayPlatform::Other => "RMeta",
    }
    .to_string(),
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

/// Converts a key into the canonical token that can be persisted and parsed again.
pub fn key_token(key: Key) -> String {
  match key {
    Key::Esc => "esc".into(),
    Key::Enter => "enter".into(),
    Key::Tab => "tab".into(),
    Key::Backspace => "backspace".into(),
    Key::Space => "space".into(),
    Key::Up => "up".into(),
    Key::Down => "down".into(),
    Key::Left => "left".into(),
    Key::Right => "right".into(),
    Key::Home => "home".into(),
    Key::End => "end".into(),
    Key::PageUp => "pageup".into(),
    Key::PageDown => "pagedown".into(),
    Key::Insert => "ins".into(),
    Key::Delete => "del".into(),
    Key::Fn(number) => format!("f{number}"),
    Key::Num(number) => number.to_string(),
    Key::Numpad(number) => format!("k{number}"),
    Key::A => "a".into(),
    Key::B => "b".into(),
    Key::C => "c".into(),
    Key::D => "d".into(),
    Key::E => "e".into(),
    Key::F => "f".into(),
    Key::G => "g".into(),
    Key::H => "h".into(),
    Key::I => "i".into(),
    Key::J => "j".into(),
    Key::K => "k".into(),
    Key::L => "l".into(),
    Key::M => "m".into(),
    Key::N => "n".into(),
    Key::O => "o".into(),
    Key::P => "p".into(),
    Key::Q => "q".into(),
    Key::R => "r".into(),
    Key::S => "s".into(),
    Key::T => "t".into(),
    Key::U => "u".into(),
    Key::V => "v".into(),
    Key::W => "w".into(),
    Key::X => "x".into(),
    Key::Y => "y".into(),
    Key::Z => "z".into(),
    Key::LeftCtrl => "left_ctrl".into(),
    Key::RightCtrl => "right_ctrl".into(),
    Key::LeftShift => "left_shift".into(),
    Key::RightShift => "right_shift".into(),
    Key::LeftAlt => "left_alt".into(),
    Key::RightAlt => "right_alt".into(),
    Key::LeftMeta => "left_meta".into(),
    Key::RightMeta => "right_meta".into(),
    Key::CapsLock => "capslock".into(),
    Key::NumLock => "numlock".into(),
    Key::ScrollLock => "scrolllock".into(),
    Key::PrintScreen => "printscreen".into(),
    Key::Pause => "pause".into(),
    Key::BackQuote => "`".into(),
    Key::Minus => "-".into(),
    Key::Equal => "=".into(),
    Key::LeftBracket => "[".into(),
    Key::RightBracket => "]".into(),
    Key::BackSlash => "\\".into(),
    Key::Semicolon => ";".into(),
    Key::Quote => "'".into(),
    Key::Comma => ",".into(),
    Key::Dot => ".".into(),
    Key::Slash => "/".into(),
    Key::NumpadAdd => "k+".into(),
    Key::NumpadSubtract => "k-".into(),
    Key::NumpadMultiply => "k*".into(),
    Key::NumpadDivide => "k/".into(),
    Key::NumpadEnter => "kenter".into(),
    Key::NumpadDelete => "kdel".into(),
    Key::Unknown(code) => format!("key({code})"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn modifier_displays_distinguish_sides_and_follow_platform_names() {
    let cases = [
      (Key::LeftCtrl, "LCtrl", "LCtrl", "LCtrl"),
      (Key::RightCtrl, "RCtrl", "RCtrl", "RCtrl"),
      (Key::LeftShift, "LShift", "LShift", "LShift"),
      (Key::RightShift, "RShift", "RShift", "RShift"),
      (Key::LeftAlt, "LAlt", "LAlt", "LAlt"),
      (Key::RightAlt, "RAlt", "RAlt", "RAlt"),
      (Key::LeftMeta, "LWin", "LCmd", "LMeta"),
      (Key::RightMeta, "RWin", "RCmd", "RMeta"),
    ];

    for (key, windows, macos, other) in cases {
      assert_eq!(
        display_key_token_for_platform(key, KeyDisplayPlatform::Windows),
        windows
      );
      assert_eq!(
        display_key_token_for_platform(key, KeyDisplayPlatform::MacOs),
        macos
      );
      assert_eq!(
        display_key_token_for_platform(key, KeyDisplayPlatform::Other),
        other
      );
    }
  }

  #[test]
  fn display_formatting_preserves_canonical_left_and_right_tokens() {
    let keys = [
      Key::LeftCtrl,
      Key::RightCtrl,
      Key::LeftShift,
      Key::RightShift,
      Key::LeftAlt,
      Key::RightAlt,
      Key::LeftMeta,
      Key::RightMeta,
    ];
    for key in keys {
      let token = key_token(key);
      assert_eq!(parse_key_token(&token), Some(key));
      assert_eq!(canonical_key_token(&token).as_deref(), Some(token.as_str()));
    }
    assert_eq!(canonical_key_token("ctrl").as_deref(), Some("left_ctrl"));
    assert_eq!(canonical_key_token("meta").as_deref(), Some("left_meta"));

    let patterns = vec![vec![
      "right_meta".to_string(),
      "left_shift".to_string(),
      "left_ctrl".to_string(),
      "x".to_string(),
    ]];
    assert_eq!(
      format_key_display_for_platform(&patterns, KeyDisplayPlatform::Windows),
      "[LCtrl + LShift + RWin + X]"
    );
    assert_eq!(
      format_key_display_for_platform(&patterns, KeyDisplayPlatform::MacOs),
      "[LCtrl + LShift + RCmd + X]"
    );
    assert_eq!(
      format_key_display_for_platform(&patterns, KeyDisplayPlatform::Other),
      "[LCtrl + LShift + RMeta + X]"
    );
  }

  #[test]
  fn single_key() {
    assert_eq!(format_key_display(&[vec!["shift".into()]]), "[LShift]");
    assert_eq!(format_key_display(&[vec!["d".into()]]), "[D]");
    assert_eq!(format_key_display(&[vec!["enter".into()]]), "[Enter]");
  }

  #[test]
  fn combo_sorted_modifier_first() {
    assert_eq!(
      format_key_display(&[vec!["d".into(), "shift".into()]]),
      "[LShift + D]"
    );
    assert_eq!(
      format_key_display(&[vec!["shift".into(), "d".into()]]),
      "[LShift + D]"
    );
  }

  #[test]
  fn multi_pattern() {
    assert_eq!(
      format_key_display(&[vec!["d".into()], vec!["left".into(), "shift".into()]]),
      "[D]/[LShift + ←]"
    );
  }

  #[test]
  fn empty_patterns() {
    assert_eq!(format_key_display(&[]), "");
  }

  #[test]
  fn unknown_token_fallback() {
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
