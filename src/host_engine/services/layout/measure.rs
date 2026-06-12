use super::types::Size;
use crate::host_engine::services::text_layout;
use crate::host_engine::services::DrawTextParams;
use crate::host_engine::services::RichTextParams;

/// 获取文本渲染后的尺寸。
///
/// 自动处理富文本标签和 `{param}` 模板替换。
/// 宽度使用 Unicode 显示宽度（`unicode_width`），正确处理
/// CJK（宽 2）、emoji（宽 2）、零宽字符（宽 0）等。
pub fn get_text_size(text: &str, params: Option<&RichTextParams>) -> Size {
  let mut draw_params = DrawTextParams::new(0, 0, text.to_string());
  draw_params.params = params.cloned();
  get_draw_text_size(&draw_params)
}

/// 获取文本渲染后的宽度。
pub fn get_text_width(text: &str, params: Option<&RichTextParams>) -> u16 {
  get_text_size(text, params).width
}

/// 获取文本渲染后的高度。
pub fn get_text_height(text: &str, params: Option<&RichTextParams>) -> u16 {
  get_text_size(text, params).height
}

/// 按 `draw_text` 的完整参数测量最终排版尺寸。
///
/// 与 canvas 绘制共用同一套 grapheme 排版逻辑，因此会尊重
/// `wrap_mode`、`max_width`、`max_height` 和 `overflow_marker`。
pub fn get_draw_text_size(params: &DrawTextParams) -> Size {
  let (width, height) = text_layout::measure_draw_text(params);
  Size { width, height }
}

pub fn get_draw_text_width(params: &DrawTextParams) -> u16 {
  get_draw_text_size(params).width
}

pub fn get_draw_text_height(params: &DrawTextParams) -> u16 {
  get_draw_text_size(params).height
}

/// 获取终端画布尺寸。
pub fn get_terminal_size() -> Size {
  let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
  Size { width, height }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::text_layout::TextWrapMode;

  #[test]
  fn draw_text_measure_respects_auto_wrap() {
    let mut params = DrawTextParams::new(10, 5, "abcd");
    params.wrap_mode = TextWrapMode::Auto;
    params.max_width = Some(2);

    assert_eq!(
      get_draw_text_size(&params),
      Size {
        width: 2,
        height: 2
      }
    );
  }

  #[test]
  fn legacy_text_measure_uses_draw_text_layout() {
    assert_eq!(
      get_text_size("ab\ncd", None),
      Size {
        width: 2,
        height: 2
      }
    );
  }
}
