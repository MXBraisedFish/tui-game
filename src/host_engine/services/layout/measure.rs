/// 测量文本宽度（按字符数，取最长行）
/// 当前用 chars().count()，未来可替换为 unicode-width 实现
pub fn measure_width(text: &str) -> u16 {
  text
    .lines()
    .map(|line| line.chars().count() as u16)
    .max()
    .unwrap_or(0)
}

/// 测量文本高度（按行数）
pub fn measure_height(text: &str) -> u16 {
  text.lines().count() as u16
}

/// 测量文本尺寸，返回（宽度, 高度）
pub fn measure_size(text: &str) -> (u16, u16) {
  (measure_width(text), measure_height(text))
}
