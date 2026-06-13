use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use super::{ComposedCell, ComposedFrame, ComposedImage};
use crate::host_engine::services::{
  CanvasCell, ImageCellRect, TerminalColor, TerminalService, TextColor, TextStyle,
};

pub struct FramePresenter {
  previous: Option<ComposedFrame>,
  force_full_redraw: bool,
}

impl FramePresenter {
  pub fn new() -> Self {
    Self {
      previous: None,
      force_full_redraw: true,
    }
  }

  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  pub fn present(
    &mut self,
    frame: &ComposedFrame,
    terminal: &mut TerminalService,
    text_force_redraw: bool,
  ) -> io::Result<()> {
    let Some(stdout) = terminal.writer_mut() else {
      return Ok(());
    };

    let full_redraw = self.force_full_redraw
      || text_force_redraw
      || self.previous.as_ref().is_none_or(|previous| {
        previous.width() != frame.width() || previous.height() != frame.height()
      });

    for rect in regions_to_clear(self.previous.as_ref(), frame, full_redraw) {
      clear_rect(stdout, rect)?;
    }

    self.present_text(stdout, frame, full_redraw)?;
    self.present_images(stdout, frame, full_redraw)?;

    stdout.queue(ResetColor)?;
    stdout.flush()?;

    self.previous = Some(frame.clone());
    self.force_full_redraw = false;

    Ok(())
  }

  fn present_text(
    &self,
    stdout: &mut impl Write,
    frame: &ComposedFrame,
    full_redraw: bool,
  ) -> io::Result<()> {
    for y in 0..frame.height() {
      let mut run: Option<(u16, &TextStyle)> = None;

      for x in 0..frame.width() {
        let current = frame.get(x, y).unwrap_or(&ComposedCell::Empty);

        let ComposedCell::Text(current_cell) = current else {
          if let Some((start, style)) = run {
            queue_text_run(stdout, frame, y, start, x, style)?;
          }
          run = None;
          continue;
        };

        if current_cell.is_continuation() {
          continue;
        }

        let changed = full_redraw || self.previous_cell(x, y) != current;

        match &run {
          None if changed => run = Some((x, &current_cell.style)),
          Some((_, style)) if changed && *style == &current_cell.style => {}
          _ => {
            if let Some((start, style)) = run {
              queue_text_run(stdout, frame, y, start, x, style)?;
            }

            if changed {
              run = Some((x, &current_cell.style));
            } else {
              run = None;
            }
          }
        }
      }

      if let Some((start, style)) = run {
        queue_text_run(stdout, frame, y, start, frame.width(), style)?;
      }
    }

    Ok(())
  }

  fn present_images(
    &self,
    stdout: &mut impl Write,
    frame: &ComposedFrame,
    full_redraw: bool,
  ) -> io::Result<()> {
    for image in frame.images() {
      if !self.should_present_image(frame, image, full_redraw) {
        continue;
      }

      stdout.queue(MoveTo(image.rect.x, image.rect.y))?;
      write!(stdout, "{}", image.sequence)?;
      stdout.queue(MoveTo(0, 0))?;
    }

    Ok(())
  }

  fn should_present_image(
    &self,
    frame: &ComposedFrame,
    image: &ComposedImage,
    full_redraw: bool,
  ) -> bool {
    if full_redraw || frame.image_dirty() {
      return true;
    }

    self
      .previous
      .as_ref()
      .and_then(|previous| previous.image_at(image.id))
      .is_none_or(|previous| previous.signature != image.signature)
  }

  fn previous_cell(&self, x: u16, y: u16) -> &ComposedCell {
    self
      .previous
      .as_ref()
      .and_then(|previous| previous.get(x, y))
      .unwrap_or(&ComposedCell::Empty)
  }
}

fn regions_to_clear(
  previous: Option<&ComposedFrame>,
  frame: &ComposedFrame,
  full_redraw: bool,
) -> Vec<ImageCellRect> {
  let mut regions = frame.removed_regions().to_vec();

  if let Some(previous) = previous {
    for image in previous.images() {
      let current = frame.image_at(image.id);
      if current.is_none_or(|current| current.signature != image.signature) {
        regions.push(image.rect);
      }
    }
  }

  if full_redraw {
    for image in frame.images() {
      regions.push(image.rect);
    }
  }

  regions
}

fn clear_rect(stdout: &mut impl Write, rect: ImageCellRect) -> io::Result<()> {
  let blank = " ".repeat(rect.width as usize);
  for y in rect.y..rect.y.saturating_add(rect.height) {
    stdout.queue(MoveTo(rect.x, y))?;
    stdout.queue(Print(&blank))?;
  }
  Ok(())
}

fn queue_text_run(
  stdout: &mut impl Write,
  frame: &ComposedFrame,
  y: u16,
  start: u16,
  end: u16,
  style: &TextStyle,
) -> io::Result<()> {
  let run_text: String = (start..end)
    .filter_map(|x| match frame.get(x, y) {
      Some(ComposedCell::Text(CanvasCell { ch, .. })) if *ch != '\0' => Some(*ch),
      _ => None,
    })
    .collect();

  if run_text.is_empty() {
    return Ok(());
  }

  stdout.queue(MoveTo(start, y))?;
  queue_style(stdout, style)?;
  stdout.queue(Print(run_text))?;

  Ok(())
}

fn terminal_color_to_crossterm(color: &TerminalColor) -> Color {
  match color {
    TerminalColor::Black => Color::Black,
    TerminalColor::Red => Color::DarkRed,
    TerminalColor::Green => Color::DarkGreen,
    TerminalColor::Yellow => Color::DarkYellow,
    TerminalColor::Blue => Color::DarkBlue,
    TerminalColor::Magenta => Color::DarkMagenta,
    TerminalColor::Cyan => Color::DarkCyan,
    TerminalColor::White => Color::White,
    TerminalColor::BrightBlack => Color::Grey,
    TerminalColor::BrightRed => Color::Red,
    TerminalColor::BrightGreen => Color::Green,
    TerminalColor::BrightYellow => Color::Yellow,
    TerminalColor::BrightBlue => Color::Blue,
    TerminalColor::BrightMagenta => Color::Magenta,
    TerminalColor::BrightCyan => Color::Cyan,
    TerminalColor::BrightWhite => Color::White,
  }
}

fn text_color_to_crossterm(color: &TextColor) -> Color {
  match color {
    TextColor::Terminal(color) => terminal_color_to_crossterm(color),
    TextColor::Rgb { r, g, b } => Color::Rgb {
      r: *r,
      g: *g,
      b: *b,
    },
  }
}

fn queue_style(stdout: &mut impl Write, style: &TextStyle) -> io::Result<()> {
  stdout.queue(ResetColor)?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm(foreground)))?;
  }

  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm(background)))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{CellPixelSize, ImageProtocol, ImageSignature};
  use std::path::PathBuf;

  fn image_signature(rect: ImageCellRect) -> ImageSignature {
    ImageSignature {
      protocol: ImageProtocol::Kitty,
      path: PathBuf::from("image.png"),
      rect,
      cell: CellPixelSize {
        width: 8,
        height: 16,
      },
      preserve_aspect_ratio: false,
    }
  }

  #[test]
  fn regions_to_clear_includes_removed_regions() {
    let rect = ImageCellRect {
      x: 1,
      y: 2,
      width: 3,
      height: 4,
    };
    let mut frame = ComposedFrame::new(10, 10);
    frame.set_removed_regions(vec![rect]);

    assert_eq!(regions_to_clear(None, &frame, false), vec![rect]);
  }

  #[test]
  fn image_anchor_present_only_changes_when_signature_changes() {
    let rect = ImageCellRect {
      x: 0,
      y: 0,
      width: 2,
      height: 2,
    };
    let mut previous = ComposedFrame::new(4, 4);
    previous.add_image(crate::host_engine::services::LayerImage {
      id: 1,
      protocol: ImageProtocol::Kitty,
      rect,
      signature: image_signature(rect),
      sequence: "old".to_string(),
    });

    let presenter = FramePresenter {
      previous: Some(previous),
      force_full_redraw: false,
    };
    let mut current = ComposedFrame::new(4, 4);
    current.add_image(crate::host_engine::services::LayerImage {
      id: 1,
      protocol: ImageProtocol::Kitty,
      rect,
      signature: image_signature(rect),
      sequence: "new".to_string(),
    });
    let image = current.images().first().expect("image");

    assert!(!presenter.should_present_image(&current, image, false));
    assert!(presenter.should_present_image(&current, image, true));
  }
}
