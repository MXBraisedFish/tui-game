use super::types::Size;
use crate::host_engine::services::DrawTextParams;
use crate::host_engine::services::RichTextParams;
use crate::host_engine::services::text_layout;
use crate::host_engine::services::rich_text::TextMode;

/// 计算文本的渲染尺寸
pub fn get_text_size(text: &str, params: Option<&RichTextParams>) -> Size {
  let mut draw_params = DrawTextParams::new(
    0,
    0,
    if params.is_some() {
      text.strip_prefix("f%").unwrap_or(text)
    } else {
      text
    }
    .to_string(),
  );
  draw_params.params = params.cloned();
  if params.is_some() {
    draw_params.text_mode = TextMode::Rich;
  }
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
  let params = params.host_formatted();
  let (width, height) = text_layout::measure_draw_text(params.as_ref());
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
  let (width, height) = crossterm::terminal::size().unwrap_or_else(|_e| {
    // TODO: log warn when terminal size query fails — fallback to (95, 24)
    (95, 24)
  });
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
  fn plain_text_measure_uses_draw_text_layout() {
    assert_eq!(
      get_text_size("ab\ncd", None),
      Size {
        width: 2,
        height: 2
      }
    );
  }

  #[test]
  fn host_parameterized_text_is_measured_as_formatted_text() {
    let mut params = RichTextParams::default();
    params.values.insert("action".into(), "[Enter]".into());

    assert_eq!(get_text_width("{value:action}", Some(&params)), 7);
    assert_eq!(get_text_width("f%{value:action}", Some(&params)), 7);
  }
}
