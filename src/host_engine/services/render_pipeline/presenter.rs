use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
  },
};

use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::{TerminalColor, TerminalService, TextColor, TextStyle};

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
    final_cursor: Option<(u16, u16)>,
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
    if let Some((x, y)) = final_cursor {
      stdout.queue(MoveTo(x, y))?;
    }
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
  let mut run_text = String::new();
  for x in start..end {
    if let Some(ComposedCell::Text(cell)) = frame.get(x, y) {
      if !cell.is_continuation() {
        run_text.push_str(&cell.text);
      }
    }
  }

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
  stdout.queue(SetAttribute(Attribute::Reset))?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm(foreground)))?;
  }

  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm(background)))?;
  }

  if style.reverse {
    stdout.queue(SetAttribute(Attribute::Reverse))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::CanvasCell;

  #[test]
  fn previous_cell_returns_empty_for_missing_previous_frame() {
    let presenter = FramePresenter::new();

    assert_eq!(presenter.previous_cell(0, 0), &ComposedCell::Empty);
  }

  #[test]
  fn queue_text_run_writes_complete_graphemes() {
    let mut frame = ComposedFrame::new(3, 1);
    frame.set(
      0,
      0,
      ComposedCell::Text(CanvasCell::styled("e\u{301}", TextStyle::default())),
    );
    frame.set(
      1,
      0,
      ComposedCell::Text(CanvasCell::styled("👨‍👩", TextStyle::default())),
    );
    frame.set(2, 0, ComposedCell::Text(CanvasCell::continuation()));
    let mut output = Vec::new();

    queue_text_run(&mut output, &frame, 0, 0, 3, &TextStyle::default()).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.ends_with("e\u{301}👨‍👩"));
  }

  #[test]
  fn queue_style_emits_and_resets_reverse_attribute() {
    let mut output = Vec::new();
    queue_style(
      &mut output,
      &TextStyle {
        reverse: true,
        ..Default::default()
      },
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("\x1b[0m"));
    assert!(output.contains("\x1b[7m"));
  }
}
