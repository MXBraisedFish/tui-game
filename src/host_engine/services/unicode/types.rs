/// 单个 grapheme cluster 的完整信息。
///
/// 一个用户感知的"字符"可能由多个 Rust `char` 组成：
/// - 带组合标记的字母：`e\u{0301}` → é（2 个 char，1 个 grapheme）
/// - ZWJ 拼接 emoji：👨 + ZWJ + 👩 → 👨‍👩（3+ 个 char，1 个 grapheme）
/// - 表情肤色修饰：👍 + 🏽 → 👍🏽（2 个 char，1 个 grapheme）
///
/// 画布写入时必须以 grapheme 为单位整体处理，不能逐 char 推进光标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphemeInfo {
    /// grapheme 原始文本（可能含多个 char）
    pub text: String,
    /// 在终端中占用的列数：0（零宽/组合标记）、1（普通）、2（宽字符）、>2（极罕见）
    pub display_width: usize,
}

/// 文字方向
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    LTR,
    RTL,
    Neutral,
}

/// 一个方向连续的文字段。
///
/// 当需要手动 Bidi 重排时，文本先被切分成方向连续的 run，
/// 然后 RTL run 在写入 canvas 时反转视觉位置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BidiRun {
    pub text: String,
    pub graphemes: Vec<GraphemeInfo>,
    pub direction: TextDirection,
}
