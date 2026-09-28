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
use tg_core_style::{TerminalColor, TextColor, TextStyle};
use tg_service_terminal::TerminalService;

/// The frame presenter, turning a [`ComposedFrame`] into crossterm commands written to the
/// terminal, with incremental redraws.
pub struct FramePresenter {
  previous: Option<ComposedFrame>,
  force_full_redraw: bool,
  output_buffer: Vec<u8>,
}

impl Default for FramePresenter {
  fn default() -> Self {
    Self::new()
  }
}

impl FramePresenter {
  pub fn new() -> Self {
    Self {
      previous: None,
      force_full_redraw: true,
      output_buffer: Vec::new(),
    }
  }

  /// Requests a full redraw on the next present.
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  /// Writes the frame to the terminal, redrawing only the changed cells unless a full redraw was
  /// requested or the frame size changed.
  ///
  /// # Errors
  ///
  /// Returns an error when writing to or flushing the terminal fails.
  pub fn present(
    &mut self,
    frame: &ComposedFrame,
    terminal: &mut TerminalService,
    text_force_redraw: bool,
    final_cursor: Option<(u16, u16)>,
  ) -> io::Result<()> {
    let truecolor = terminal.capabilities().truecolor;

    let Some(writer) = terminal.writer_mut() else {
      // TODO: log warn when terminal writer is missing (headless mode)
      return Ok(());
    };

    self.present_to_writer(frame, writer, text_force_redraw, final_cursor, truecolor)
  }

  fn present_to_writer(
    &mut self,
    frame: &ComposedFrame,
    writer: &mut impl Write,
    text_force_redraw: bool,
    final_cursor: Option<(u16, u16)>,
    truecolor: bool,
  ) -> io::Result<()> {
    let full_redraw = self.force_full_redraw
      || text_force_redraw
      || self.previous.as_ref().is_none_or(|previous| {
        previous.width() != frame.width() || previous.height() != frame.height()
      });

    let mut output = std::mem::take(&mut self.output_buffer);
    output.clear();
    let encode_result = (|| {
      self.present_cells(&mut output, frame, full_redraw, truecolor)?;
      output.queue(ResetColor)?;
      if let Some((x, y)) = final_cursor {
        output.queue(MoveTo(x, y))?;
      }
      Ok::<(), io::Error>(())
    })();
    self.output_buffer = output;
    encode_result?;

    if let Err(error) = writer.write_all(&self.output_buffer) {
      self.force_full_redraw = true;
      return Err(error);
    }
    if let Err(error) = writer.flush() {
      self.force_full_redraw = true;
      return Err(error);
    }

    self.previous = Some(frame.clone());
    self.force_full_redraw = false;

    Ok(())
  }

  fn present_cells(
    &self,
    stdout: &mut impl Write,
    frame: &ComposedFrame,
    full_redraw: bool,
    truecolor: bool,
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
    if let Some(ComposedCell::Text(cell)) = frame.get(x, y)
      && !cell.is_continuation()
    {
      run_text.push_str(&cell.text);
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
    TextColor::ForceRgb { r, g, b } => Color::Rgb {
      r: *r,
      g: *g,
      b: *b,
    },

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

  if let Some(background) = &style.background
    && !matches!(background, TextColor::Transparent)
  {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm(
      background, truecolor,
    )))?;
  }

  if style.reverse {
    stdout.queue(SetAttribute(Attribute::Reverse))?;
  }
  if style.bold {
    stdout.queue(SetAttribute(Attribute::Bold))?;
  }
  if style.italic {
    stdout.queue(SetAttribute(Attribute::Italic))?;
  }
  if style.underline {
    stdout.queue(SetAttribute(Attribute::Underlined))?;
  }
  if style.strike {
    stdout.queue(SetAttribute(Attribute::CrossedOut))?;
  }
  if style.blink {
    stdout.queue(SetAttribute(Attribute::SlowBlink))?;
  }
  if style.hidden {
    stdout.queue(SetAttribute(Attribute::Hidden))?;
  }
  if style.dim {
    stdout.queue(SetAttribute(Attribute::Dim))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_core_style::CanvasCell;
  use tg_service_canvas::CanvasService;

  #[derive(Default)]
  struct TrackingWriter {
    bytes: Vec<u8>,
    writes: usize,
    flushes: usize,
    fail_on_write: Option<usize>,
    fail_flush: bool,
  }

  impl Write for TrackingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.writes += 1;
      if self.fail_on_write == Some(self.writes) {
        return Err(io::Error::other("configured write failure"));
      }
      self.bytes.extend_from_slice(bytes);
      Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
      self.flushes += 1;
      if self.fail_flush {
        return Err(io::Error::other("configured flush failure"));
      }
      Ok(())
    }
  }

  #[derive(Default)]
  struct ShortWriter {
    bytes: Vec<u8>,
    write_limit: usize,
    writes: usize,
    flushes: usize,
    fail_on_write: Option<usize>,
  }

  impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.writes += 1;
      if self.fail_on_write == Some(self.writes) {
        return Err(io::Error::other("configured short-write failure"));
      }
      let written = bytes.len().min(self.write_limit);
      self.bytes.extend_from_slice(&bytes[..written]);
      Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
      self.flushes += 1;
      Ok(())
    }
  }

  fn text_frame(text: &str) -> ComposedFrame {
    let mut frame = ComposedFrame::new(text.chars().count() as u16, 1);
    for (x, character) in text.chars().enumerate() {
      let grapheme = character.to_string();
      frame.set(
        x as u16,
        0,
        ComposedCell::Text(CanvasCell::styled(grapheme, TextStyle::default())),
      );
    }
    frame
  }

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
  fn present_to_writer_emits_expected_bytes_and_flushes_once() {
    let mut presenter = FramePresenter::new();
    let frame = text_frame("AB");
    let mut writer = TrackingWriter::default();

    presenter
      .present_to_writer(&frame, &mut writer, false, None, true)
      .unwrap();

    assert_eq!(writer.bytes, b"\x1b[1;1H\x1b[0m\x1b[0mAB\x1b[0m");
    assert_eq!(writer.writes, 1);
    assert_eq!(writer.flushes, 1);
    assert_eq!(presenter.previous.as_ref(), Some(&frame));
  }

  #[test]
  fn final_cursor_is_counted_and_written_after_the_frame_reset() {
    let mut presenter = FramePresenter::new();
    let frame = text_frame("AB");
    let mut writer = TrackingWriter::default();

    presenter
      .present_to_writer(&frame, &mut writer, false, Some((1, 0)), true)
      .unwrap();

    assert_eq!(writer.bytes, b"\x1b[1;1H\x1b[0m\x1b[0mAB\x1b[0m\x1b[1;2H");
    assert_eq!(writer.bytes.len(), 26);
    assert_eq!(writer.writes, 1);
    assert_eq!(writer.flushes, 1);
  }

  #[test]
  fn merged_frame_write_handles_short_writes() {
    let mut presenter = FramePresenter::new();
    let frame = text_frame("AB");
    let mut writer = ShortWriter {
      write_limit: 3,
      ..Default::default()
    };

    presenter
      .present_to_writer(&frame, &mut writer, false, Some((1, 0)), true)
      .unwrap();

    assert_eq!(writer.bytes, b"\x1b[1;1H\x1b[0m\x1b[0mAB\x1b[0m\x1b[1;2H");
    assert_eq!(writer.writes, writer.bytes.len().div_ceil(3));
    assert_eq!(writer.flushes, 1);
    assert_eq!(presenter.previous.as_ref(), Some(&frame));
  }

  #[test]
  fn partial_merged_write_failure_after_a_frame_forces_full_redraw() {
    let mut presenter = FramePresenter::new();
    let previous = text_frame("AB");
    let frame = text_frame("AC");
    presenter
      .present_to_writer(&previous, &mut TrackingWriter::default(), false, None, true)
      .unwrap();
    let mut failing_writer = ShortWriter {
      write_limit: 3,
      fail_on_write: Some(3),
      ..Default::default()
    };

    assert!(
      presenter
        .present_to_writer(&frame, &mut failing_writer, false, Some((1, 0)), true)
        .is_err()
    );
    assert_eq!(failing_writer.bytes.len(), 6);
    assert_eq!(presenter.previous.as_ref(), Some(&previous));
    assert!(presenter.force_full_redraw);

    let mut retry_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&frame, &mut retry_writer, false, Some((1, 0)), true)
      .unwrap();

    assert!(
      String::from_utf8(retry_writer.bytes)
        .unwrap()
        .contains("AC")
    );
    assert_eq!(presenter.previous.as_ref(), Some(&frame));
  }

  #[test]
  fn cursor_output_counts_cover_static_local_and_full_frames() {
    let first = text_frame("AB");
    let changed = text_frame("AC");
    let mut presenter = FramePresenter::new();
    presenter
      .present_to_writer(&first, &mut TrackingWriter::default(), false, None, true)
      .unwrap();

    let mut static_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&first, &mut static_writer, false, Some((1, 0)), true)
      .unwrap();

    let mut local_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&changed, &mut local_writer, false, Some((1, 0)), true)
      .unwrap();

    let mut full_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&changed, &mut full_writer, true, Some((1, 0)), true)
      .unwrap();

    let measurements = [
      (
        static_writer.bytes.len(),
        static_writer.writes,
        static_writer.flushes,
      ),
      (
        local_writer.bytes.len(),
        local_writer.writes,
        local_writer.flushes,
      ),
      (
        full_writer.bytes.len(),
        full_writer.writes,
        full_writer.flushes,
      ),
    ];
    assert!(
      [&static_writer, &local_writer, &full_writer]
        .into_iter()
        .all(|writer| writer.bytes.ends_with(b"\x1b[0m\x1b[1;2H"))
    );
    assert_eq!(measurements, [(10, 1, 1), (25, 1, 1), (26, 1, 1)]);
  }

  #[test]
  fn write_failure_does_not_mark_the_frame_as_presented() {
    let mut presenter = FramePresenter::new();
    let frame = text_frame("A");
    let mut failing_writer = TrackingWriter {
      fail_on_write: Some(1),
      ..TrackingWriter::default()
    };

    assert!(
      presenter
        .present_to_writer(&frame, &mut failing_writer, false, None, true)
        .is_err()
    );
    assert!(presenter.previous.is_none());
    assert!(presenter.force_full_redraw);

    let mut retry_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&frame, &mut retry_writer, false, None, true)
      .unwrap();
    assert_eq!(presenter.previous.as_ref(), Some(&frame));
    assert!(
      String::from_utf8(retry_writer.bytes)
        .unwrap()
        .contains("\x1b[1;1H")
    );
  }

  #[test]
  fn flush_failure_keeps_previous_frame_and_forces_full_redraw() {
    let mut presenter = FramePresenter::new();
    let first = text_frame("A");
    let second = text_frame("B");
    presenter
      .present_to_writer(&first, &mut TrackingWriter::default(), false, None, true)
      .unwrap();

    let mut failing_writer = TrackingWriter {
      fail_flush: true,
      ..TrackingWriter::default()
    };
    assert!(
      presenter
        .present_to_writer(&second, &mut failing_writer, false, None, true)
        .is_err()
    );
    assert_eq!(presenter.previous.as_ref(), Some(&first));
    assert!(presenter.force_full_redraw);

    let mut retry_writer = TrackingWriter::default();
    presenter
      .present_to_writer(&second, &mut retry_writer, false, None, true)
      .unwrap();
    assert_eq!(presenter.previous.as_ref(), Some(&second));
    assert!(String::from_utf8(retry_writer.bytes).unwrap().contains("B"));
    assert_eq!(retry_writer.writes, 1);
  }

  #[test]
  fn compositor_clear_changes_text_to_a_printed_blank_cell() {
    let mut canvas = CanvasService::new();
    canvas.resize(1, 1);
    canvas.styled_text(0, 0, "A", TextStyle::default());
    let compositor = crate::FrameCompositor::new();
    let first = compositor.compose(&canvas);
    assert!(matches!(
      first.get(0, 0),
      Some(ComposedCell::Text(cell)) if cell.text == "A"
    ));

    let mut presenter = FramePresenter::new();
    presenter
      .present_to_writer(&first, &mut TrackingWriter::default(), false, None, true)
      .unwrap();

    canvas.clear();
    let cleared = compositor.compose(&canvas);
    assert!(matches!(
      cleared.get(0, 0),
      Some(ComposedCell::Text(cell)) if cell.text == " "
    ));
    let mut writer = TrackingWriter::default();
    presenter
      .present_to_writer(&cleared, &mut writer, false, None, true)
      .unwrap();

    assert_eq!(writer.bytes, b"\x1b[1;1H\x1b[0m\x1b[0m \x1b[0m");
  }

  #[test]
  fn narrowing_a_wide_cell_overwrites_its_previous_continuation() {
    let mut wide = ComposedFrame::new(2, 1);
    wide.set(
      0,
      0,
      ComposedCell::Text(CanvasCell::styled("界", TextStyle::default())),
    );
    wide.set(1, 0, ComposedCell::Text(CanvasCell::continuation()));
    let narrow = text_frame("A ");
    let mut presenter = FramePresenter::new();
    presenter
      .present_to_writer(&wide, &mut TrackingWriter::default(), false, None, true)
      .unwrap();
    let mut writer = TrackingWriter::default();

    presenter
      .present_to_writer(&narrow, &mut writer, false, None, true)
      .unwrap();

    assert!(
      String::from_utf8(writer.bytes)
        .unwrap()
        .ends_with("A \x1b[0m")
    );
  }

  #[test]
  fn changing_styles_resets_attributes_before_the_next_run() {
    let mut frame = ComposedFrame::new(2, 1);
    frame.set(
      0,
      0,
      ComposedCell::Text(CanvasCell::styled(
        "A",
        TextStyle {
          bold: true,
          ..TextStyle::default()
        },
      )),
    );
    frame.set(
      1,
      0,
      ComposedCell::Text(CanvasCell::styled("B", TextStyle::default())),
    );
    let mut presenter = FramePresenter::new();
    let mut writer = TrackingWriter::default();

    presenter
      .present_to_writer(&frame, &mut writer, false, None, true)
      .unwrap();

    let output = String::from_utf8(writer.bytes).unwrap();
    assert!(output.contains("\x1b[1m"), "{output:?}");
    assert!(output.contains("\x1b[0m\x1b[0mB"), "{output:?}");
  }

  #[test]
  fn resize_forces_equal_cells_to_be_redrawn_and_cursor_is_last() {
    let first = text_frame("B");
    let resized = text_frame("BC");
    let mut presenter = FramePresenter::new();
    presenter
      .present_to_writer(&first, &mut TrackingWriter::default(), false, None, true)
      .unwrap();
    let mut writer = TrackingWriter::default();

    presenter
      .present_to_writer(&resized, &mut writer, false, Some((1, 0)), true)
      .unwrap();

    let output = String::from_utf8(writer.bytes).unwrap();
    assert!(output.contains("BC"));
    assert!(output.ends_with("\x1b[0m\x1b[1;2H"));
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

  #[test]
  fn forced_rgb_ignores_truecolor_capability_flag() {
    let rgb = TextColor::ForceRgb { r: 1, g: 2, b: 3 };

    assert_eq!(
      text_color_to_crossterm(&rgb, false),
      Color::Rgb { r: 1, g: 2, b: 3 }
    );
  }

  struct MeasuredWriter<'a, W> {
    inner: &'a mut W,
    bytes: u64,
    writes: u64,
    flushes: u64,
  }

  impl<'a, W> MeasuredWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
      Self {
        inner,
        bytes: 0,
        writes: 0,
        flushes: 0,
      }
    }

    fn reset_counts(&mut self) {
      self.bytes = 0;
      self.writes = 0;
      self.flushes = 0;
    }
  }

  impl<W: Write> Write for MeasuredWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.writes += 1;
      let written = self.inner.write(bytes)?;
      self.bytes = self.bytes.saturating_add(written as u64);
      Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
      self.flushes += 1;
      self.inner.flush()
    }
  }

  #[derive(Clone, Copy)]
  enum BenchmarkMode {
    Static,
    Local,
    Full,
  }

  impl BenchmarkMode {
    fn label(self) -> &'static str {
      match self {
        Self::Static => "static",
        Self::Local => "local",
        Self::Full => "full",
      }
    }
  }

  fn benchmark_frame(width: u16, height: u16) -> ComposedFrame {
    const SAMPLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut frame = ComposedFrame::new(width, height);
    for y in 0..height {
      for x in 0..width {
        if y % 3 == 1 {
          if x.is_multiple_of(2) {
            frame.set(
              x,
              y,
              ComposedCell::Text(CanvasCell::styled("界", TextStyle::default())),
            );
            frame.set(x + 1, y, ComposedCell::Text(CanvasCell::continuation()));
          }
          continue;
        }
        let character = SAMPLE[(usize::from(x) + usize::from(y) * 7) % SAMPLE.len()] as char;
        let style = if y % 3 == 2 {
          TextStyle {
            bold: x.is_multiple_of(2),
            underline: x.is_multiple_of(3),
            reverse: x.is_multiple_of(5),
            ..TextStyle::default()
          }
        } else {
          TextStyle::default()
        };
        frame.set(
          x,
          y,
          ComposedCell::Text(CanvasCell::styled(character.to_string(), style)),
        );
      }
    }
    frame
  }

  fn percentile_microseconds(samples: &[u64], percentile: f64) -> f64 {
    if samples.is_empty() {
      return 0.0;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ((ordered.len() - 1) as f64 * percentile).ceil() as usize;
    ordered[index.min(ordered.len() - 1)] as f64 / 1_000.0
  }

  #[cfg(windows)]
  fn process_cpu_time() -> Option<std::time::Duration> {
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
      low: u32,
      high: u32,
    }

    impl FileTime {
      fn ticks(&self) -> u64 {
        (u64::from(self.high) << 32) | u64::from(self.low)
      }
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
      fn GetCurrentProcess() -> *mut std::ffi::c_void;
      fn GetProcessTimes(
        process: *mut std::ffi::c_void,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
      ) -> i32;
    }

    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    let process = unsafe { GetCurrentProcess() };
    let succeeded =
      unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) };
    if succeeded == 0 {
      return None;
    }
    Some(std::time::Duration::from_nanos(
      kernel
        .ticks()
        .saturating_add(user.ticks())
        .saturating_mul(100),
    ))
  }

  #[cfg(not(windows))]
  fn process_cpu_time() -> Option<std::time::Duration> {
    None
  }

  #[test]
  #[ignore = "requires an interactive terminal; runs the 30-second-per-mode B8.3 baseline"]
  fn terminal_presenter_performance_baseline() {
    use std::{fs, thread, time::Instant};

    let seconds_per_target = std::env::var("TG_B8_SECONDS_PER_TARGET")
      .ok()
      .and_then(|value| value.parse::<u64>().ok())
      .filter(|seconds| *seconds > 0)
      .unwrap_or(10);
    let report_path = std::env::var_os("TG_B8_REPORT")
      .map(std::path::PathBuf::from)
      .unwrap_or_else(|| std::env::temp_dir().join("tg_b8_presenter_baseline.csv"));
    if let Some(parent) = report_path
      .parent()
      .filter(|parent| !parent.as_os_str().is_empty())
    {
      fs::create_dir_all(parent).expect("benchmark report directory should be writable");
    }
    let (terminal_width, terminal_height) = crossterm::terminal::size().unwrap_or((0, 0));
    let mut terminal = TerminalService::new();
    terminal
      .enter()
      .expect("B8.3 requires a usable terminal attached to this process");
    let truecolor = terminal.capabilities().truecolor;
    let mut report = "terminal_width,terminal_height,frame_width,frame_height,mode,target_fps,frames,elapsed_ms,actual_fps,p50_us,p95_us,max_us,cpu_one_core_percent,bytes,writes,flushes\n".to_string();
    fs::write(&report_path, &report).expect("benchmark report should be writable");
    {
      let mut measured = MeasuredWriter::new(
        terminal
          .writer_mut()
          .expect("terminal writer is available after entering the terminal"),
      );

      for (width, height) in [(80u16, 24u16), (160, 50), (240, 80)] {
        for mode in [
          BenchmarkMode::Static,
          BenchmarkMode::Local,
          BenchmarkMode::Full,
        ] {
          let mut frame = benchmark_frame(width, height);
          let mut presenter = FramePresenter::new();
          presenter
            .present_to_writer(&frame, &mut measured, true, None, truecolor)
            .expect("initial benchmark frame should render");

          for target_fps in [30u64, 60, 120] {
            measured.reset_counts();
            let target_interval = std::time::Duration::from_nanos(1_000_000_000 / target_fps);
            let segment_duration = std::time::Duration::from_secs(seconds_per_target);
            let segment_start = Instant::now();
            let cpu_start = process_cpu_time();
            let mut frame_times = Vec::new();
            let mut frames = 0u64;

            while segment_start.elapsed() < segment_duration {
              if !matches!(mode, BenchmarkMode::Static) {
                let next = if frames.is_multiple_of(2) { "X" } else { "Y" };
                frame.set(
                  0,
                  0,
                  ComposedCell::Text(CanvasCell::styled(next, TextStyle::default())),
                );
              }
              let draw_start = Instant::now();
              presenter
                .present_to_writer(
                  &frame,
                  &mut measured,
                  matches!(mode, BenchmarkMode::Full),
                  None,
                  truecolor,
                )
                .expect("benchmark frame should render");
              let draw_time = draw_start.elapsed();
              frame_times.push(draw_time.as_nanos().min(u64::MAX as u128) as u64);
              frames += 1;
              thread::sleep(target_interval.saturating_sub(draw_time));
            }

            let elapsed = segment_start.elapsed();
            let cpu_time = cpu_start
              .zip(process_cpu_time())
              .map(|(start, end)| end.saturating_sub(start));
            let mut ordered = frame_times;
            ordered.sort_unstable();
            let maximum_us = ordered.last().copied().unwrap_or_default() as f64 / 1_000.0;
            let p50_us = percentile_microseconds(&ordered, 0.50);
            let p95_us = percentile_microseconds(&ordered, 0.95);
            let actual_fps = frames as f64 / elapsed.as_secs_f64();
            let cpu_percent = cpu_time
              .map(|cpu| 100.0 * cpu.as_secs_f64() / elapsed.as_secs_f64())
              .map(|percent| format!("{percent:.2}"))
              .unwrap_or_default();
            report.push_str(&format!(
              "{terminal_width},{terminal_height},{width},{height},{},{target_fps},{frames},{},{actual_fps:.2},{p50_us:.2},{p95_us:.2},{maximum_us:.2},{cpu_percent},{},{},{}\n",
              mode.label(),
              elapsed.as_millis(),
              measured.bytes,
              measured.writes,
              measured.flushes,
            ));
            fs::write(&report_path, &report).expect("benchmark report should be writable");
          }
        }
      }
    }

    terminal.clear_all_and_home().unwrap();
    terminal.exit();
    fs::write(&report_path, &report).expect("benchmark report should be writable");
    println!(
      "B8.3 terminal baseline written to {}",
      report_path.display()
    );
    println!("{report}");
  }

  #[test]
  #[ignore = "requires an interactive terminal; measures static, local, and full frames with a final cursor move"]
  fn terminal_presenter_cursor_output_baseline() {
    use std::{fs, thread, time::Instant};

    let seconds_per_target = std::env::var("TG_B9_SECONDS_PER_TARGET")
      .ok()
      .and_then(|value| value.parse::<u64>().ok())
      .filter(|seconds| *seconds > 0)
      .unwrap_or(10);
    let report_path = std::env::var_os("TG_B9_REPORT")
      .map(std::path::PathBuf::from)
      .unwrap_or_else(|| std::env::temp_dir().join("tg_b9_cursor_baseline.csv"));
    if let Some(parent) = report_path
      .parent()
      .filter(|parent| !parent.as_os_str().is_empty())
    {
      fs::create_dir_all(parent).expect("benchmark report directory should be writable");
    }

    let (terminal_width, terminal_height) = crossterm::terminal::size().unwrap_or((0, 0));
    let mut terminal = TerminalService::new();
    terminal
      .enter()
      .expect("B9.2 requires a usable terminal attached to this process");
    let truecolor = terminal.capabilities().truecolor;
    let mut report = "terminal_width,terminal_height,frame_width,frame_height,mode,target_fps,frames,elapsed_ms,actual_fps,p50_us,p95_us,max_us,cpu_one_core_percent,bytes,writes,flushes\n".to_string();
    fs::write(&report_path, &report).expect("benchmark report should be writable");
    {
      let mut measured = MeasuredWriter::new(
        terminal
          .writer_mut()
          .expect("terminal writer is available after entering the terminal"),
      );
      let mut frame = benchmark_frame(80, 24);
      let mut presenter = FramePresenter::new();
      presenter
        .present_to_writer(&frame, &mut measured, true, Some((40, 12)), truecolor)
        .expect("initial cursor benchmark frame should render");

      for mode in [
        BenchmarkMode::Static,
        BenchmarkMode::Local,
        BenchmarkMode::Full,
      ] {
        for target_fps in [30u64, 60, 120] {
          measured.reset_counts();
          let target_interval = std::time::Duration::from_nanos(1_000_000_000 / target_fps);
          let segment_duration = std::time::Duration::from_secs(seconds_per_target);
          let segment_start = Instant::now();
          let cpu_start = process_cpu_time();
          let mut frame_times = Vec::new();
          let mut frames = 0u64;

          while segment_start.elapsed() < segment_duration {
            if !matches!(mode, BenchmarkMode::Static) {
              let next = if frames.is_multiple_of(2) { "X" } else { "Y" };
              frame.set(
                0,
                0,
                ComposedCell::Text(CanvasCell::styled(next, TextStyle::default())),
              );
            }
            let draw_start = Instant::now();
            presenter
              .present_to_writer(
                &frame,
                &mut measured,
                matches!(mode, BenchmarkMode::Full),
                Some((40, 12)),
                truecolor,
              )
              .expect("cursor benchmark frame should render");
            let draw_time = draw_start.elapsed();
            frame_times.push(draw_time.as_nanos().min(u64::MAX as u128) as u64);
            frames += 1;
            thread::sleep(target_interval.saturating_sub(draw_time));
          }

          let elapsed = segment_start.elapsed();
          let cpu_time = cpu_start
            .zip(process_cpu_time())
            .map(|(start, end)| end.saturating_sub(start));
          let mut ordered = frame_times;
          ordered.sort_unstable();
          let maximum_us = ordered.last().copied().unwrap_or_default() as f64 / 1_000.0;
          let p50_us = percentile_microseconds(&ordered, 0.50);
          let p95_us = percentile_microseconds(&ordered, 0.95);
          let actual_fps = frames as f64 / elapsed.as_secs_f64();
          let cpu_percent = cpu_time
            .map(|cpu| 100.0 * cpu.as_secs_f64() / elapsed.as_secs_f64())
            .map(|percent| format!("{percent:.2}"))
            .unwrap_or_default();
          report.push_str(&format!(
            "{terminal_width},{terminal_height},80,24,{},{target_fps},{frames},{},{actual_fps:.2},{p50_us:.2},{p95_us:.2},{maximum_us:.2},{cpu_percent},{},{},{}\n",
            mode.label(),
            elapsed.as_millis(),
            measured.bytes,
            measured.writes,
            measured.flushes,
          ));
          fs::write(&report_path, &report).expect("benchmark report should be writable");
        }
      }
    }

    terminal.clear_all_and_home().unwrap();
    terminal.exit();
    fs::write(&report_path, &report).expect("benchmark report should be writable");
    println!(
      "B9.2 terminal cursor baseline written to {}",
      report_path.display()
    );
    println!("{report}");
  }
}
