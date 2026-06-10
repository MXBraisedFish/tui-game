use crate::host_engine::services::{CanvasService, DrawTextParams};

/// 渲染服务 —— 当前为薄壳，直接委托给 CanvasService。
///
/// 未来可在此层加入视口裁剪、坐标变换等宿主侧渲染逻辑。
pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  /// 唯一的绘制入口。
  /// 委托给 `canvas.text()`，由其内部完成 f% 路由和样式解析。
  pub fn draw_text(
    &mut self,
    canvas: &mut CanvasService,
    params: &DrawTextParams,
  ) {
    canvas.text(params);
  }
}

