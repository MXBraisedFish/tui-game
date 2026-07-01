use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
  },
};
use once_cell::sync::Lazy;
use palette::{IntoColor, Lab, Srgb, color_difference::Ciede2000};

use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::{TerminalColor, TerminalService, TextColor, TextStyle};

/// 帧呈现器：将 ComposedFrame 转换为 crossterm 指令并输出到终端，支持增量刷新。
pub struct FramePresenter {
  previous: Option<ComposedFrame>,
  force_full_redraw: bool,

  truecolor: bool,
}

impl FramePresenter {
  pub fn new() -> Self {
    Self {
      previous: None,
      force_full_redraw: true,
      truecolor: false,
    }
  }

  /// 标记下次呈现时需要全量重绘。
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  /// 将帧输出到终端，仅重绘变化区域（除非要求全量重绘或尺寸变化）。
  pub fn present(
    &mut self,
    frame: &ComposedFrame,
    terminal: &mut TerminalService,
    text_force_redraw: bool,
    final_cursor: Option<(u16, u16)>,
  ) -> io::Result<()> {
    self.truecolor = terminal.capabilities().truecolor;

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
    let truecolor = self.truecolor;
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
                  queue_text_run(stdout, frame, y, start, x, style, truecolor)?;
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
              queue_text_run(stdout, frame, y, start, x, style, truecolor)?;
            }
            run = None;
          }
        }
      }

      if let Some((start, style)) = run {
        queue_text_run(stdout, frame, y, start, frame.width(), style, truecolor)?;
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
  truecolor: bool,
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
  queue_style(stdout, style, truecolor)?;
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

fn text_color_to_crossterm(color: &TextColor, truecolor: bool) -> Color {
  match color {
    TextColor::Terminal(color) => terminal_color_to_crossterm(color),
    TextColor::Rgb { r, g, b } => {
      if truecolor {
        Color::Rgb {
          r: *r,
          g: *g,
          b: *b,
        }
      } else {
        nearest_ansi256(*r, *g, *b)
      }
    }

    TextColor::Transparent => Color::Reset,
  }
}

type LabPalette = Vec<(u8, Lab)>;

static LAB_PALETTE: Lazy<LabPalette> = Lazy::new(|| {
  let mut entries = Vec::with_capacity(240);

  for r_idx in 0u8..6 {
    for g_idx in 0u8..6 {
      for b_idx in 0u8..6 {
        let code = 16 + 36 * r_idx + 6 * g_idx + b_idx;
        let rgb = cube_level_to_rgb(r_idx, g_idx, b_idx);
        let lab = rgb_to_lab(rgb);
        entries.push((code, lab));
      }
    }
  }

  for gray in 0u8..24 {
    let code = 232 + gray;
    let v = gray * 10 + 8;
    let rgb = (v, v, v);
    let lab = rgb_to_lab(rgb);
    entries.push((code, lab));
  }

  entries
});

fn cube_level_to_rgb(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
  fn level(l: u8) -> u8 {
    if l == 0 { 0 } else { l * 40 + 55 }
  }
  (level(r), level(g), level(b))
}

fn rgb_to_lab(rgb: (u8, u8, u8)) -> Lab {
  let linear = Srgb::new(
    rgb.0 as f32 / 255.0,
    rgb.1 as f32 / 255.0,
    rgb.2 as f32 / 255.0,
  );
  linear.into_color()
}

fn nearest_ansi256(r: u8, g: u8, b: u8) -> Color {
  let target = rgb_to_lab((r, g, b));

  let best = LAB_PALETTE
    .iter()
    .min_by(|(_, a), (_, b)| {
      let da = target.difference(*a);
      let db = target.difference(*b);
      da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
    .expect("LAB_PALETTE 非空");

  Color::AnsiValue(best.0)
}

fn queue_style(stdout: &mut impl Write, style: &TextStyle, truecolor: bool) -> io::Result<()> {
  stdout.queue(ResetColor)?;
  stdout.queue(SetAttribute(Attribute::Reset))?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm(
      foreground, truecolor,
    )))?;
  }

  if let Some(background) = &style.background {
    if !matches!(background, TextColor::Transparent) {
      stdout.queue(SetBackgroundColor(text_color_to_crossterm(
        background, truecolor,
      )))?;
    }
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

    queue_text_run(&mut output, &frame, 0, 0, 3, &TextStyle::default(), true).unwrap();

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
      true,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("\x1b[0m"));
    assert!(output.contains("\x1b[7m"));
  }

  #[test]
  fn nearest_ansi256_maps_primary_colors() {
    assert!(matches!(nearest_ansi256(0, 0, 0), Color::AnsiValue(_)));
    assert!(matches!(
      nearest_ansi256(255, 255, 255),
      Color::AnsiValue(_)
    ));
    assert!(matches!(nearest_ansi256(255, 0, 0), Color::AnsiValue(_)));
    assert!(matches!(nearest_ansi256(0, 0, 255), Color::AnsiValue(_)));
  }

  #[test]
  fn nearest_ansi256_gray_vs_cube_is_consistent() {
    let gray128 = nearest_ansi256(128, 128, 128);
    assert!(matches!(gray128, Color::AnsiValue(_)));
  }

  #[test]
  fn text_color_to_crossterm_falls_back_to_256_when_truecolor_disabled() {
    let rgb = TextColor::Rgb { r: 255, g: 0, b: 0 };

    assert_eq!(
      text_color_to_crossterm(&rgb, true),
      Color::Rgb { r: 255, g: 0, b: 0 }
    );

    assert!(matches!(
      text_color_to_crossterm(&rgb, false),
      Color::AnsiValue(_)
    ));
  }
}
