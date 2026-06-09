use crate::host_engine::services::CanvasService;

pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  pub fn draw_text(&mut self, canvas: &mut CanvasService, x: u16, y: u16, text: &str) {
    canvas.text(x, y, text);
  }

  pub fn draw_text_block(&mut self, canvas: &mut CanvasService, x: u16, y: u16, text: &str) {
    for (line_index, line) in text.lines().enumerate() {
      canvas.text(x, y.saturating_add(line_index as u16), line);
    }
  }
}
