use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::{
  CanvasCell, TerminalColor, TerminalService, TextColor, TextStyle,
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

    self.present_cells(stdout, frame, full_redraw)?;

    stdout.queue(ResetColor)?;
    stdout.flush()?;

    self.previous = Some(frame.clone());
    self.force_full_redraw = false;

    Ok(())
  }

  fn present_cells(
    &self,
    stdout: &mut impl Write,
    frame: &ComposedFrame,
    full_redraw: bool,
  ) -> io::Result<()> {
    for y in 0..frame.height() {
      let mut run: Option<(u16, &TextStyle)> = None;

      for x in 0..frame.width() {
        let current = frame.get(x, y).unwrap_or(&ComposedCell::Empty);

        match current {
          ComposedCell::Text(current_cell) => {
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
          ComposedCell::Empty => {
            if let Some((start, style)) = run {
              queue_text_run(stdout, frame, y, start, x, style)?;
            }
            run = None;
          }
        }
      }

      if let Some((start, style)) = run {
        queue_text_run(stdout, frame, y, start, frame.width(), style)?;
      }
    }

    Ok(())
  }

  fn previous_cell(&self, x: u16, y: u16) -> &ComposedCell {
    self
      .previous
      .as_ref()
      .and_then(|previous| previous.get(x, y))
      .unwrap_or(&ComposedCell::Empty)
  }
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

  #[test]
  fn previous_cell_returns_empty_for_missing_previous_frame() {
    let presenter = FramePresenter::new();

    assert_eq!(presenter.previous_cell(0, 0), &ComposedCell::Empty);
  }
}
