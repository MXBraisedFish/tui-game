//! 文本绘图 API
//!
//! 提供三个 Rust 端绘图方法，封装普通文本与富文本的绘制逻辑。
//! 通过 RenderService 调用，画布的所有权和脏跟踪仍由 CanvasService 管理。

use crate::host_engine::services::{CanvasService, CanvasStyle, RichTextService};

use super::RenderService;

/// 富文本前缀标记
///
/// 以 "f%" 开头的文本字符串将被识别为富文本格式，
/// 否则视为普通文本，使用默认样式绘制。
const RICH_TEXT_PREFIX: &str = "f%";

impl RenderService {
  /// 根据前缀自动选择普通文本或富文本绘制
  ///
  /// - 以 `f%` 开头 → 调用 `draw_rich_text`
  /// - 否则        → 调用 `draw_normal_text`
  pub fn draw_text(
    &mut self,
    canvas: &mut CanvasService,
    rich_text: &RichTextService,
    x: u16,
    y: u16,
    text: &str,
  ) {
    if text.starts_with(RICH_TEXT_PREFIX) {
      self.draw_rich_text(canvas, rich_text, x, y, text);
    } else {
      self.draw_normal_text(canvas, x, y, text);
    }
  }

  /// 绘制普通文本（默认样式，无标记解析）
  ///
  /// 直接使用 `CanvasStyle::default()` 写入画布，
  /// 适用于不需要样式的静态或动态文本。
  pub fn draw_normal_text(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    text: &str,
  ) {
    canvas.write_text(x, y, text, CanvasStyle::default());
  }

  /// 绘制富文本
  ///
  /// 解析 "f%" 前缀的格式化字符串，逐段应用样式写入画布。
  /// 支持 `<bold>`、`<fg:red>`、`<bg:blue>` 等标签。
  pub fn draw_rich_text(
    &mut self,
    canvas: &mut CanvasService,
    rich_text: &RichTextService,
    x: u16,
    y: u16,
    text: &str,
  ) {
    let rich_text = rich_text.parse(text, None);
    canvas.write_rich_text(x, y, &rich_text);
  }
}
