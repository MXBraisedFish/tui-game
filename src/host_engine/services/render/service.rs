use crate::host_engine::services::CanvasService;

pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  pub fn draw_text(&mut self, canvas: &mut CanvasService, x: u16, y: u16, text: &str) {
    canvas.text(x, y, text);
  }
}
