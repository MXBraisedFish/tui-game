use super::types::Size;
use crate::host_engine::services::DrawTextParams;
use crate::host_engine::services::RichTextParams;
use crate::host_engine::services::text_layout;

/// 计算文本的渲染尺寸
pub fn get_text_size(text: &str, params: Option<&RichTextParams>) -> Size {
  let mut draw_params = DrawTextParams::new(0, 0, text.to_string());
  draw_params.params = params.cloned();
  get_draw_text_size(&draw_params)
}

/// 计算文本的渲染宽度
pub fn get_text_width(text: &str, params: Option<&RichTextParams>) -> u16 {
  get_text_size(text, params).width
}

/// 计算文本的渲染高度
pub fn get_text_height(text: &str, params: Option<&RichTextParams>) -> u16 {
  get_text_size(text, params).height
}

/// 计算带排版参数的文本渲染尺寸
pub fn get_draw_text_size(params: &DrawTextParams) -> Size {
  let (width, height) = text_layout::measure_draw_text(params);
  Size { width, height }
}

/// 计算带排版参数的文本渲染宽度
pub fn get_draw_text_width(params: &DrawTextParams) -> u16 {
  get_draw_text_size(params).width
}

/// 计算带排版参数的文本渲染高度
pub fn get_draw_text_height(params: &DrawTextParams) -> u16 {
  get_draw_text_size(params).height
}

/// 获取当前终端尺寸
pub fn get_terminal_size() -> Size {
  let (width, height) = crossterm::terminal::size().unwrap_or((95, 24));
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
