//! 渲染服务
//!
//! 过渡性渲染服务，保持与 EngineServices 的兼容性。
//! 实际渲染状态已迁移到 CanvasService，此模块后续将承载高级绘图 API。

pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  // 尺寸变化回调
  //
  // 当前为空实现。CanvasService 拥有实际的呈现状态，
  // 终端大小变化由 canvas.resize() 处理。
  pub fn resize(&mut self, _width: u16, _height: u16) {}
}
