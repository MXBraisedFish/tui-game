use std::ops::Range;

const MAX_PATTERN_STEPS: usize = 1_000_000;

#[derive(Clone, Debug)]
pub enum LuaCapture {
  Text(Range<usize>),
  Position(usize),
}

#[derive(Clone, Debug)]
pub struct LuaCaptures {
  pub full: Range<usize>,
  pub captures: Vec<Option<LuaCapture>>,
}

impl LuaCaptures {
  pub fn len(&self) -> usize {
    self.captures.len() + 1
  }

  pub fn value(&self, index: usize) -> Option<LuaCapture> {
    if index == 0 {
      Some(LuaCapture::Text(self.full.clone()))
    } else {
      self.captures.get(index - 1).cloned().flatten()
    }
  }
}

#[derive(Clone, Debug)]
pub struct LuaPattern {
  ops: Vec<Op>,
  capture_count: usize,
  anchored: bool,
}

#[derive(Clone, Debug)]
enum Op {
  Atom(Atom, Quantifier),
  CaptureStart(usize),
  CaptureEnd(usize),
  CapturePosition(usize),
  Frontier(CharSet),
  End,
}

#[derive(Clone, Debug)]
enum Atom {
  Any,
  Literal(char),
  Class(CharClass),
  Set(CharSet),
  Balanced(char, char),
  BackReference(usize),
}

#[derive(Clone, Copy, Debug)]
enum Quantifier {
  One,
  Optional,
  ZeroOrMore,
  OneOrMore,
  ZeroOrMoreMinimal,
}

#[derive(Clone, Debug)]
struct CharSet {
  negated: bool,
  entries: Vec<SetEntry>,
}

#[derive(Clone, Debug)]
enum SetEntry {
  Literal(char),
  Range(char, char),
  Class(CharClass),
}

#[derive(Clone, Copy, Debug)]
enum CharClass {
  Alpha,
  Control,
  Digit,
  Graph,
  Lower,
  Punctuation,
  Space,
  Upper,
  Word,
  Hex,
  Zero,
  NotAlpha,
  NotControl,
  NotDigit,
  NotGraph,
  NotLower,
  NotPunctuation,
  NotSpace,
  NotUpper,
  NotWord,
  NotHex,
}

#[derive(Clone)]
struct MatchState {
  captures: Vec<Option<LuaCapture>>,
  starts: Vec<Option<usize>>,
}

struct Input<'a> {
  text: &'a str,
  chars: Vec<char>,
  bytes: Vec<usize>,
}

impl<'a> Input<'a> {
  fn new(text: &'a str) -> Self {
    let mut bytes = text
      .char_indices()
      .map(|(index, _)| index)
      .collect::<Vec<_>>();
    bytes.push(text.len());
    Self {
      text,
      chars: text.chars().collect(),
      bytes,
    }
  }

  fn byte_range(&self, range: Range<usize>) -> Range<usize> {
    self.bytes[range.start]..self.bytes[range.end]
  }

  fn capture_text(&self, capture: &LuaCapture) -> Option<&'a str> {
    let LuaCapture::Text(range) = capture else {
      return None;
    };
    Some(&self.text[self.byte_range(range.clone())])
  }
}

impl LuaPattern {
  pub fn compile(pattern: &str) -> Result<Self, String> {
    let chars = pattern.chars().collect::<Vec<_>>();
    let mut parser = Parser {
      chars: &chars,
      index: 0,
      capture_count: 0,
    };
    let anchored = parser.take('^');
    let ops = parser.sequence(false)?;
    if parser.index != chars.len() {
      return Err("unexpected ')' in pattern".to_string());
    }
    Ok(Self {
      ops,
      capture_count: parser.capture_count,
      anchored,
    })
  }

  pub fn captures(&self, text: &str, start: usize) -> Result<Option<LuaCaptures>, String> {
    let mut steps = 0;
    self.captures_with_steps(text, start, &mut steps)
  }

  pub fn captures_incremental(
    &self,
    text: &str,
    start: usize,
    steps: &mut usize,
  ) -> Result<Option<LuaCaptures>, String> {
    self.captures_with_steps(text, start, steps)
  }

  fn captures_with_steps(
    &self,
    text: &str,
    start: usize,
    steps: &mut usize,
  ) -> Result<Option<LuaCaptures>, String> {
    let input = Input::new(text);
    let first = start.min(input.chars.len());
    let positions: Box<dyn Iterator<Item = usize>> = if self.anchored {
      if first == 0 {
        Box::new(std::iter::once(0))
      } else {
        Box::new(std::iter::empty())
      }
    } else {
      Box::new(first..=input.chars.len())
    };
    for position in positions {
      let state = MatchState {
        captures: vec![None; self.capture_count],
        starts: vec![None; self.capture_count],
      };
      if let Some((end, state)) = self.match_ops(&input, 0, position, state, steps)? {
        let full = input.byte_range(position..end);
        let captures = state
          .captures
          .into_iter()
          .map(|capture| {
            capture.map(|capture| match capture {
              LuaCapture::Text(range) => LuaCapture::Text(input.byte_range(range)),
              LuaCapture::Position(position) => LuaCapture::Position(position + 1),
            })
          })
          .collect();
        return Ok(Some(LuaCaptures { full, captures }));
      }
    }
    Ok(None)
  }

  pub fn captures_iter(&self, text: &str) -> Result<Vec<LuaCaptures>, String> {
    let output = self.captures_iter_limited(text, 10_001)?;
    if output.len() > 10_000 {
      Err("result exceeds 10000 items".to_string())
    } else {
      Ok(output)
    }
  }

  pub fn captures_iter_limited(
    &self,
    text: &str,
    limit: usize,
  ) -> Result<Vec<LuaCaptures>, String> {
    let char_bytes = text
      .char_indices()
      .map(|(index, _)| index)
      .collect::<Vec<_>>();
    let mut byte_start = 0;
    let mut output = Vec::new();
    let mut steps = 0;
    while byte_start <= text.len() && output.len() < limit {
      let char_start = char_bytes.partition_point(|index| *index < byte_start);
      let Some(captures) = self.captures_with_steps(text, char_start, &mut steps)? else {
        break;
      };
      let next = if captures.full.end > captures.full.start {
        captures.full.end
      } else if captures.full.end < text.len() {
        text[captures.full.end..]
          .chars()
          .next()
          .map_or(text.len() + 1, |value| captures.full.end + value.len_utf8())
      } else {
        text.len() + 1
      };
      output.push(captures);
      byte_start = next;
    }
    Ok(output)
  }

  fn match_ops(
    &self,
    input: &Input<'_>,
    op_index: usize,
    position: usize,
    mut state: MatchState,
    steps: &mut usize,
  ) -> Result<Option<(usize, MatchState)>, String> {
    *steps += 1;
    if *steps > MAX_PATTERN_STEPS {
      return Err("pattern exceeded 1000000 matching steps".to_string());
    }
    let Some(op) = self.ops.get(op_index) else {
      return Ok(Some((position, state)));
    };
    match op {
      Op::CaptureStart(index) => {
        state.starts[*index] = Some(position);
        self.match_ops(input, op_index + 1, position, state, steps)
      }
      Op::CaptureEnd(index) => {
        let Some(start) = state.starts[*index] else {
          return Ok(None);
        };
        state.captures[*index] = Some(LuaCapture::Text(start..position));
        self.match_ops(input, op_index + 1, position, state, steps)
      }
      Op::CapturePosition(index) => {
        state.captures[*index] = Some(LuaCapture::Position(position));
        self.match_ops(input, op_index + 1, position, state, steps)
      }
      Op::Frontier(set) => {
        let previous_matches = position
          .checked_sub(1)
          .and_then(|index| input.chars.get(index))
          .is_some_and(|value| set.matches(*value));
        let current_matches = input
          .chars
          .get(position)
          .is_some_and(|value| set.matches(*value));
        if !previous_matches && current_matches {
          self.match_ops(input, op_index + 1, position, state, steps)
        } else {
          Ok(None)
        }
      }
      Op::End => {
        if position == input.chars.len() {
          self.match_ops(input, op_index + 1, position, state, steps)
        } else {
          Ok(None)
        }
      }
      Op::Atom(atom, quantifier) => {
        let mut positions = vec![position];
        let mut current = position;
        while let Some(next) = atom.matches(input, current, &state, steps)? {
          if next == current {
            break;
          }
          positions.push(next);
          current = next;
          if positions.len() > input.chars.len() + 1 {
            break;
          }
        }
        let candidates: Vec<usize> = match quantifier {
          Quantifier::One => positions.get(1).copied().into_iter().collect(),
          Quantifier::Optional => positions.iter().take(2).rev().copied().collect(),
          Quantifier::ZeroOrMore => positions.iter().rev().copied().collect(),
          Quantifier::OneOrMore => positions.iter().skip(1).rev().copied().collect(),
          Quantifier::ZeroOrMoreMinimal => positions,
        };
        for candidate in candidates {
          if let Some(result) =
            self.match_ops(input, op_index + 1, candidate, state.clone(), steps)?
          {
            return Ok(Some(result));
          }
        }
        Ok(None)
      }
    }
  }
}

impl Atom {
  fn matches(
    &self,
    input: &Input<'_>,
    position: usize,
    state: &MatchState,
    steps: &mut usize,
  ) -> Result<Option<usize>, String> {
    *steps += 1;
    if *steps > MAX_PATTERN_STEPS {
      return Err("pattern exceeded 1000000 matching steps".to_string());
    }
    let value = input.chars.get(position).copied();
    Ok(match self {
      Self::Any => value.map(|_| position + 1),
      Self::Literal(expected) => (value == Some(*expected)).then_some(position + 1),
      Self::Class(class) => value
        .is_some_and(|value| class.matches(value))
        .then_some(position + 1),
      Self::Set(set) => value
        .is_some_and(|value| set.matches(value))
        .then_some(position + 1),
      Self::Balanced(open, close) => {
        if value != Some(*open) {
          None
        } else if open == close {
          input.chars[position + 1..]
            .iter()
            .position(|value| value == close)
            .map(|offset| position + offset + 2)
        } else {
          let mut depth = 1_usize;
          let mut end = position + 1;
          while let Some(value) = input.chars.get(end) {
            *steps += 1;
            if *steps > MAX_PATTERN_STEPS {
              return Err("pattern exceeded 1000000 matching steps".to_string());
            }
            if value == open {
              depth += 1;
            } else if value == close {
              depth -= 1;
              if depth == 0 {
                break;
              }
            }
            end += 1;
          }
          (depth == 0).then_some(end + 1)
        }
      }
      Self::BackReference(index) => {
        let capture = state.captures.get(*index).and_then(Option::as_ref);
        let Some(capture) = capture.and_then(|capture| input.capture_text(capture)) else {
          return Ok(None);
        };
        let count = capture.chars().count();
        let end = position.saturating_add(count);
        (end <= input.chars.len()
          && input.chars[position..end]
            .iter()
            .copied()
            .eq(capture.chars()))
        .then_some(end)
      }
    })
  }
}

impl CharSet {
  fn matches(&self, value: char) -> bool {
    let matched = self.entries.iter().any(|entry| match entry {
      SetEntry::Literal(expected) => value == *expected,
      SetEntry::Range(start, end) => *start <= value && value <= *end,
      SetEntry::Class(class) => class.matches(value),
    });
    matched != self.negated
  }
}

impl CharClass {
  fn matches(self, value: char) -> bool {
    match self {
      Self::Alpha => value.is_alphabetic(),
      Self::Control => value.is_control(),
      Self::Digit => value.is_ascii_digit(),
      Self::Graph => !value.is_whitespace() && !value.is_control(),
      Self::Lower => value.is_lowercase(),
      Self::Punctuation => value.is_ascii_punctuation(),
      Self::Space => value.is_whitespace(),
      Self::Upper => value.is_uppercase(),
      Self::Word => value.is_alphanumeric(),
      Self::Hex => value.is_ascii_hexdigit(),
      Self::Zero => value == '\0',
      Self::NotAlpha => !Self::Alpha.matches(value),
      Self::NotControl => !Self::Control.matches(value),
      Self::NotDigit => !Self::Digit.matches(value),
      Self::NotGraph => !Self::Graph.matches(value),
      Self::NotLower => !Self::Lower.matches(value),
      Self::NotPunctuation => !Self::Punctuation.matches(value),
      Self::NotSpace => !Self::Space.matches(value),
      Self::NotUpper => !Self::Upper.matches(value),
      Self::NotWord => !Self::Word.matches(value),
      Self::NotHex => !Self::Hex.matches(value),
    }
  }

  fn parse(value: char) -> Option<Self> {
    Some(match value {
      'a' => Self::Alpha,
      'c' => Self::Control,
      'd' => Self::Digit,
      'g' => Self::Graph,
      'l' => Self::Lower,
      'p' => Self::Punctuation,
      's' => Self::Space,
      'u' => Self::Upper,
      'w' => Self::Word,
      'x' => Self::Hex,
      'z' => Self::Zero,
      'A' => Self::NotAlpha,
      'C' => Self::NotControl,
      'D' => Self::NotDigit,
      'G' => Self::NotGraph,
      'L' => Self::NotLower,
      'P' => Self::NotPunctuation,
      'S' => Self::NotSpace,
      'U' => Self::NotUpper,
      'W' => Self::NotWord,
      'X' => Self::NotHex,
      _ => return None,
    })
  }
}

struct Parser<'a> {
  chars: &'a [char],
  index: usize,
  capture_count: usize,
}

impl Parser<'_> {
  fn sequence(&mut self, nested: bool) -> Result<Vec<Op>, String> {
    let mut output = Vec::new();
    while let Some(value) = self.peek() {
      if value == ')' {
        if nested {
          break;
        }
        return Err("unexpected ')' in pattern".to_string());
      }
      if value == '$' && self.index + 1 == self.chars.len() {
        self.index += 1;
        output.push(Op::End);
        continue;
      }
      if value == '(' {
        self.index += 1;
        let capture = self.new_capture()?;
        if self.take(')') {
          output.push(Op::CapturePosition(capture));
          continue;
        }
        output.push(Op::CaptureStart(capture));
        output.extend(self.sequence(true)?);
        if !self.take(')') {
          return Err("unfinished capture in pattern".to_string());
        }
        output.push(Op::CaptureEnd(capture));
        continue;
      }
      if value == '%' && self.peek_n(1) == Some('f') {
        self.index += 2;
        if !self.take('[') {
          return Err("%f must be followed by a character set".to_string());
        }
        output.push(Op::Frontier(self.set_body()?));
        continue;
      }
      let atom = self.atom()?;
      let quantifier = match self.peek() {
        Some('?') => Quantifier::Optional,
        Some('*') => Quantifier::ZeroOrMore,
        Some('+') => Quantifier::OneOrMore,
        Some('-') => Quantifier::ZeroOrMoreMinimal,
        _ => Quantifier::One,
      };
      if !matches!(quantifier, Quantifier::One) {
        self.index += 1;
      }
      output.push(Op::Atom(atom, quantifier));
    }
    Ok(output)
  }

  fn atom(&mut self) -> Result<Atom, String> {
    let value = self
      .next()
      .ok_or_else(|| "missing pattern atom".to_string())?;
    Ok(match value {
      '.' => Atom::Any,
      '[' => Atom::Set(self.set_body()?),
      '%' => {
        let escaped = self
          .next()
          .ok_or_else(|| "dangling '%' in pattern".to_string())?;
        if escaped == 'b' {
          let open = self
            .next()
            .ok_or_else(|| "%b requires two delimiter characters".to_string())?;
          let close = self
            .next()
            .ok_or_else(|| "%b requires two delimiter characters".to_string())?;
          Atom::Balanced(open, close)
        } else if let Some(index) = escaped.to_digit(10).filter(|value| *value > 0) {
          let index = index as usize - 1;
          if index >= self.capture_count {
            return Err("invalid capture reference".to_string());
          }
          Atom::BackReference(index)
        } else if let Some(class) = CharClass::parse(escaped) {
          Atom::Class(class)
        } else {
          Atom::Literal(escaped)
        }
      }
      value => Atom::Literal(value),
    })
  }

  fn set_body(&mut self) -> Result<CharSet, String> {
    let negated = self.take('^');
    let mut entries = Vec::new();
    let mut first = true;
    loop {
      let Some(value) = self.next() else {
        return Err("unfinished character set".to_string());
      };
      if value == ']' && !first {
        break;
      }
      first = false;
      let entry = if value == '%' {
        let escaped = self
          .next()
          .ok_or_else(|| "dangling '%' in character set".to_string())?;
        CharClass::parse(escaped).map_or(SetEntry::Literal(escaped), SetEntry::Class)
      } else if self.peek() == Some('-') && self.peek_n(1).is_some_and(|end| end != ']') {
        self.index += 1;
        let end = self.next().unwrap();
        if value > end {
          return Err("invalid descending range in character set".to_string());
        }
        SetEntry::Range(value, end)
      } else {
        SetEntry::Literal(value)
      };
      entries.push(entry);
    }
    if entries.is_empty() {
      return Err("empty character set".to_string());
    }
    Ok(CharSet { negated, entries })
  }

  fn new_capture(&mut self) -> Result<usize, String> {
    if self.capture_count >= 32 {
      return Err("pattern exceeds 32 captures".to_string());
    }
    let value = self.capture_count;
    self.capture_count += 1;
    Ok(value)
  }

  fn peek(&self) -> Option<char> {
    self.chars.get(self.index).copied()
  }

  fn peek_n(&self, offset: usize) -> Option<char> {
    self.chars.get(self.index + offset).copied()
  }

  fn next(&mut self) -> Option<char> {
    let value = self.peek()?;
    self.index += 1;
    Some(value)
  }

  fn take(&mut self, expected: char) -> bool {
    if self.peek() == Some(expected) {
      self.index += 1;
      true
    } else {
      false
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn balanced_and_frontier_patterns_match() {
    let balanced = LuaPattern::compile("%b()").unwrap();
    let capture = balanced.captures("x(a(b)c)y", 0).unwrap().unwrap();
    assert_eq!(&"x(a(b)c)y"[capture.full], "(a(b)c)");

    let frontier = LuaPattern::compile("%f[%a]word%f[%A]").unwrap();
    let capture = frontier.captures("a word!", 0).unwrap().unwrap();
    assert_eq!(&"a word!"[capture.full], "word");
  }

  #[test]
  fn captures_backreferences_and_minimal_repeats_work() {
    let pattern = LuaPattern::compile("(a+).- %1").unwrap();
    let capture = pattern.captures("aaa x aaa", 0).unwrap().unwrap();
    assert_eq!(&"aaa x aaa"[capture.full], "aaa x aaa");
  }

  #[test]
  fn classes_sets_anchors_and_position_captures_follow_lua_patterns() {
    let word = LuaPattern::compile("^%w+$").unwrap();
    assert!(word.captures("abc123", 0).unwrap().is_some());
    assert!(word.captures("abc_123", 0).unwrap().is_none());

    let set = LuaPattern::compile("^[^%s%d]+$").unwrap();
    assert!(set.captures("alpha", 0).unwrap().is_some());
    assert!(set.captures("alpha2", 0).unwrap().is_none());

    let position = LuaPattern::compile("()b").unwrap();
    let captures = position.captures("abc", 0).unwrap().unwrap();
    assert!(matches!(captures.value(1), Some(LuaCapture::Position(2))));
  }

  #[test]
  fn empty_matches_advance_by_unicode_character() {
    let pattern = LuaPattern::compile("").unwrap();
    let captures = pattern.captures_iter("你a").unwrap();
    assert_eq!(captures.len(), 3);
    assert_eq!(captures[0].full, 0..0);
    assert_eq!(captures[1].full, 3..3);
    assert_eq!(captures[2].full, 4..4);
  }

  #[test]
  fn invalid_patterns_are_rejected() {
    for pattern in ["%", "[", "(", "%b(", "%f%a", "[z-a]", "%1"] {
      assert!(LuaPattern::compile(pattern).is_err(), "{pattern}");
    }
  }
}
