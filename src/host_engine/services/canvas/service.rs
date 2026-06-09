use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::Print,
};

use super::{buffer::CanvasBuffer, cell::CanvasCell};
use crate::host_engine::services::TerminalService;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasService {
  current: CanvasBuffer,
  previous: CanvasBuffer,
  force_full_redraw: bool,
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      current: CanvasBuffer::new(width, height),
      previous: CanvasBuffer::new(width, height),
      force_full_redraw: true,
    }
  }

  pub fn width(&self) -> u16 {
    self.current.width()
  }

  pub fn height(&self) -> u16 {
    self.current.height()
  }

  pub fn size(&self) -> (u16, u16) {
    (self.width(), self.height())
  }

  pub fn begin_frame(&mut self) {
    // 为运行生命周期保留。当前最小画布没有帧局部状态。
  }

  pub fn clear(&mut self) {
    self.current.clear();
  }

  pub fn text(&mut self, x: u16, y: u16, text: &str) {
    let mut cursor_x = x;
    for ch in text.chars() {
      if cursor_x >= self.current.width() || y >= self.current.height() {
        break;
      }
      self.current.set(cursor_x, y, CanvasCell::new(ch));
      cursor_x = cursor_x.saturating_add(1);
    }
  }

  pub fn resize(&mut self, width: u16, height: u16) {
    self.current.resize(width, height);
    self.previous.resize(width, height);
    self.force_full_redraw = true;
  }

  pub fn present(&mut self, terminal: &mut TerminalService) -> io::Result<()> {
    let Some(stdout) = terminal.writer_mut() else {
      return Ok(());
    };

    for y in 0..self.current.height() {
      let mut run_start: Option<u16> = None;

      for x in 0..self.current.width() {
        let current_ch = self
          .current
          .get(x, y)
          .map(|cell| cell.ch)
          .unwrap_or(' ');

        let previous_ch = self
          .previous
          .get(x, y)
          .map(|cell| cell.ch)
          .unwrap_or(' ');

        let cell_changed =
          self.force_full_redraw || current_ch != previous_ch;

        match (run_start, cell_changed) {
          // 进入连续变化区段
          (None, true) => {
            run_start = Some(x);
          }

          // 离开连续变化区段，输出累积的字符串
          (Some(start), false) => {
            let run_text: String = (start..x)
              .map(|rx| {
                self
                  .current
                  .get(rx, y)
                  .map(|cell| cell.ch)
                  .unwrap_or(' ')
              })
              .collect();

            stdout.queue(MoveTo(start, y))?;
            stdout.queue(Print(run_text))?;
            run_start = None;
          }

          _ => {}
        }
      }

      // 行尾：输出最后一个连续区段
      if let Some(start) = run_start {
        let run_text: String = (start..self.current.width())
          .map(|rx| {
            self
              .current
              .get(rx, y)
              .map(|cell| cell.ch)
              .unwrap_or(' ')
          })
          .collect();

        stdout.queue(MoveTo(start, y))?;
        stdout.queue(Print(run_text))?;
      }
    }

    stdout.flush()?;

    self.previous = self.current.clone();
    self.force_full_redraw = false;

    Ok(())
  }
}
