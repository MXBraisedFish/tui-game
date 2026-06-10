use crate::host_engine::services::TextStyle;

/// 标记被左侧宽字符"占用"的单元格。
/// 终端中宽字符（如 CJK、emoji）占 2 列，
/// 它右侧的那一格不写入独立字符，仅作为视觉延续。
pub const WIDE_CONTINUATION: char = '\0';

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
    pub ch: char,
    pub style: TextStyle,
}

impl CanvasCell {
    pub fn blank() -> Self {
        Self {
            ch: ' ',
            style: TextStyle::default(),
        }
    }

    pub fn new(ch: char) -> Self {
        Self {
            ch,
            style: TextStyle::default(),
        }
    }

    pub fn styled(ch: char, style: TextStyle) -> Self {
        Self { ch, style }
    }

    /// 构造一个"宽字符延续"占位格。
    pub fn continuation() -> Self {
        Self {
            ch: WIDE_CONTINUATION,
            style: TextStyle::default(),
        }
    }

    /// 是否为"宽字符延续"占位格。
    pub fn is_continuation(&self) -> bool {
        self.ch == WIDE_CONTINUATION
    }
}
