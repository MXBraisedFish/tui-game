use super::types::Size;
use crate::host_engine::services::RichTextParams;
use crate::host_engine::services::RichTextService;
use crate::host_engine::services::display_width;

/// 获取文本渲染后的尺寸。
///
/// 自动处理富文本标签和 `{param}` 模板替换。
/// 宽度使用 Unicode 显示宽度（`unicode_width`），正确处理
/// CJK（宽 2）、emoji（宽 2）、零宽字符（宽 0）等。
pub fn get_text_size(
  text: &str,
  params: Option<&RichTextParams>,
) -> Size {
  let visible = visible_content(text, params);
  let width = visible
    .lines()
    .map(|line| display_width(line) as u16)
    .max()
    .unwrap_or(0);
  let height = visible.lines().count() as u16;
  Size { width, height }
}

/// 获取文本渲染后的宽度。
pub fn get_text_width(
  text: &str,
  params: Option<&RichTextParams>,
) -> u16 {
  get_text_size(text, params).width
}

/// 获取文本渲染后的高度。
pub fn get_text_height(
  text: &str,
  params: Option<&RichTextParams>,
) -> u16 {
  get_text_size(text, params).height
}

/// 获取终端画布尺寸。
pub fn get_terminal_size() -> Size {
  let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
  Size { width, height }
}

/// 去除 f% 前缀、富文本标签、并进行模板替换，返回纯可见文本。
fn visible_content(
  text: &str,
  params: Option<&RichTextParams>,
) -> String {
  if !text.starts_with("f%") {
    return text.to_string();
  }
  RichTextService::new().visible_text(text, params)
}
