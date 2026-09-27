use super::BorderStyle;
use tg_core_style::{TextColor, TextStyle};
use tg_core_unicode::char_width;
use tg_service_canvas::CanvasService;
use tg_service_canvas::{ScrollBoxId, SliceId, SurfaceId};
use tg_service_text_layout::DrawTextParams;

#[derive(Clone, Copy)]
enum Target {
  Base,
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
  Host,
  Top,
}

/// The render service, providing high-level drawing operations such as text, filled rectangles and
/// bordered rectangles.
pub struct RenderService;

impl Default for RenderService {
  fn default() -> Self {
    Self::new()
  }
}

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  /// Draws text on the base layer.
  pub fn draw_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    self.draw_text_target(canvas, Target::Base, params);
  }

  /// Draws text at signed coordinates on the base layer; the canvas clips whatever falls outside
  /// it.
  pub fn draw_text_at(
    &mut self,
    canvas: &mut CanvasService,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) {
    self.draw_text_target_at(canvas, Target::Base, x, y, params);
  }

  /// Draws text on the given slice. Returns whether it was drawn (`false` when the slice is not
  /// visible).
  pub fn draw_text_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    params: &DrawTextParams,
  ) -> bool {
    canvas.text_on(slice, params)
  }

  pub fn draw_text_at_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) -> bool {
    canvas.text_at_on(slice, x, y, params)
  }

  /// Draws text in the virtual content area of the given scroll box.
  pub fn draw_text_in_scroll_box(
    &mut self,
    canvas: &mut CanvasService,
    id: ScrollBoxId,
    params: &DrawTextParams,
  ) -> bool {
    canvas.text_in_scroll_box(id, params)
  }

  pub fn draw_text_at_in_scroll_box(
    &mut self,
    canvas: &mut CanvasService,
    id: ScrollBoxId,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) -> bool {
    canvas.text_at_in_scroll_box(id, x, y, params)
  }

  /// Draws text on the host layer (used for top-level UI elements).
  pub fn draw_host_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    let params = params.host_formatted();
    self.draw_text_target(canvas, Target::Host, params.as_ref());
  }

  /// Draws text on the host's top layer.
  pub fn draw_top_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    let params = params.host_formatted();
    self.draw_text_target(canvas, Target::Top, params.as_ref());
  }

  /// Draws a filled rectangle on the base layer.
  // reason: public API; grouping the parameters would change its signature.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_filled_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) {
    self.draw_filled_rect_target(
      canvas,
      Target::Base,
      x.into(),
      y.into(),
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
  }

  /// Draws a filled rectangle on the given slice. Returns whether it was drawn.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_filled_rect_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) -> bool {
    if canvas.prepared_slice_rect(slice).is_none() {
      return false;
    }
    self.draw_filled_rect_target(
      canvas,
      Target::Slice(slice),
      x.into(),
      y.into(),
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
    true
  }

  /// Draws a filled rectangle in the virtual content area of the given scroll box.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_filled_rect_in_scroll_box(
    &mut self,
    canvas: &mut CanvasService,
    id: ScrollBoxId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) -> bool {
    if canvas.prepared_scroll_box_rect(id).is_none() {
      return false;
    }
    self.draw_filled_rect_target(
      canvas,
      Target::ScrollBox(id),
      x.into(),
      y.into(),
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
    true
  }

  /// Draws a filled rectangle on the host layer.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_host_filled_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) {
    self.draw_filled_rect_target(
      canvas,
      Target::Host,
      x.into(),
      y.into(),
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_filled_rect_target(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) {
    let ch = match fill_char {
      Some(ref s) if !s.is_empty() => {
        let c = s.chars().next().unwrap_or(' ');
        if char_width(c) == 1 { c } else { ' ' }
      }
      _ => ' ',
    };

    let fill_str: String = std::iter::repeat_n(ch, width as usize).collect();
    for row in 0..height {
      self.draw_text_target_at(
        canvas,
        target,
        x,
        y.saturating_add(i32::from(row)),
        &DrawTextParams {
          text: fill_str.clone(),
          fg: fill_fg.clone(),
          bg: fill_bg.clone(),
          ..Default::default()
        },
      );
    }
  }

  /// Draws a styled bordered rectangle on the base layer.
  // reason: public API; grouping the parameters would change its signature.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_border_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) {
    self.draw_border_rect_target(
      canvas,
      Target::Base,
      x.into(),
      y.into(),
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
  }

  /// Draws a styled bordered rectangle on the given slice. Returns whether it was drawn.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_border_rect_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) -> bool {
    if canvas.prepared_slice_rect(slice).is_none() {
      return false;
    }
    self.draw_border_rect_target(
      canvas,
      Target::Slice(slice),
      x.into(),
      y.into(),
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
    true
  }

  /// Draws a bordered rectangle in the virtual content area of the given scroll box.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_border_rect_in_scroll_box(
    &mut self,
    canvas: &mut CanvasService,
    id: ScrollBoxId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) -> bool {
    if canvas.prepared_scroll_box_rect(id).is_none() {
      return false;
    }
    self.draw_border_rect_target(
      canvas,
      Target::ScrollBox(id),
      x.into(),
      y.into(),
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
    true
  }

  /// Draws a styled bordered rectangle on the host layer.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_host_border_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) {
    self.draw_border_rect_target(
      canvas,
      Target::Host,
      x.into(),
      y.into(),
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
  }

  /// Draws a styled bordered rectangle on the host's top layer.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_top_border_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) {
    self.draw_border_rect_target(
      canvas,
      Target::Top,
      x.into(),
      y.into(),
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_border_rect_target(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) {
    let custom = match border_style.to_custom() {
      Some(c) => c,
      None => return,
    };
    if width < 2 || height < 2 {
      return;
    }

    let mid_w = width.saturating_sub(2);
    let mid_h = height.saturating_sub(2);
    let fg_ref = border_fg.as_ref();
    let bg_ref = border_bg.as_ref();
    let attrs_ref = border_attrs.as_ref();

    let lt_s = custom.left_top.resolve(fg_ref, bg_ref, attrs_ref);
    let t_s = custom.top.resolve(fg_ref, bg_ref, attrs_ref);
    let rt_s = custom.right_top.resolve(fg_ref, bg_ref, attrs_ref);
    let r_s = custom.right.resolve(fg_ref, bg_ref, attrs_ref);
    let rb_s = custom.right_bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let b_s = custom.bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let lb_s = custom.left_bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let l_s = custom.left.resolve(fg_ref, bg_ref, attrs_ref);

    let lt_ch = custom.left_top.char.unwrap_or(' ');
    let t_ch = custom.top.char.unwrap_or(' ');
    let rt_ch = custom.right_top.char.unwrap_or(' ');
    let r_ch = custom.right.char.unwrap_or(' ');
    let rb_ch = custom.right_bottom.char.unwrap_or(' ');
    let b_ch = custom.bottom.char.unwrap_or(' ');
    let lb_ch = custom.left_bottom.char.unwrap_or(' ');
    let l_ch = custom.left.char.unwrap_or(' ');

    self.draw_border_cell(canvas, target, x, y, lt_ch, &lt_s);
    self.draw_border_span(
      canvas,
      target,
      x.saturating_add(1),
      y,
      std::iter::repeat_n(t_ch, mid_w as usize).collect(),
      &t_s,
    );
    self.draw_border_cell(
      canvas,
      target,
      x.saturating_add(i32::from(width - 1)),
      y,
      rt_ch,
      &rt_s,
    );

    let fill_text = fill_bg.as_ref().map(|_| " ".repeat(mid_w as usize));
    for row in 1..=mid_h {
      let cy = y.saturating_add(i32::from(row));
      self.draw_border_cell(canvas, target, x, cy, l_ch, &l_s);
      if let Some(fill_text) = &fill_text {
        self.draw_text_target_at(
          canvas,
          target,
          x.saturating_add(1),
          cy,
          &DrawTextParams {
            text: fill_text.clone(),
            bg: fill_bg.clone(),
            ..Default::default()
          },
        );
      }
      self.draw_border_cell(
        canvas,
        target,
        x.saturating_add(i32::from(width - 1)),
        cy,
        r_ch,
        &r_s,
      );
    }

    let bot_y = y.saturating_add(i32::from(height - 1));
    self.draw_border_cell(canvas, target, x, bot_y, lb_ch, &lb_s);
    self.draw_border_span(
      canvas,
      target,
      x.saturating_add(1),
      bot_y,
      std::iter::repeat_n(b_ch, mid_w as usize).collect(),
      &b_s,
    );
    self.draw_border_cell(
      canvas,
      target,
      x.saturating_add(i32::from(width - 1)),
      bot_y,
      rb_ch,
      &rb_s,
    );
  }

  fn draw_border_cell(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: i32,
    y: i32,
    ch: char,
    style: &TextStyle,
  ) {
    self.draw_text_target_at(
      canvas,
      target,
      x,
      y,
      &DrawTextParams {
        text: ch.to_string(),
        fg: style.foreground.clone(),
        bg: style.background.clone(),
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strike,
        blink: style.blink,
        reverse: style.reverse,
        hidden: style.hidden,
        dim: style.dim,
        ..Default::default()
      },
    );
  }

  fn draw_border_span(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: i32,
    y: i32,
    text: String,
    style: &TextStyle,
  ) {
    self.draw_text_target_at(
      canvas,
      target,
      x,
      y,
      &DrawTextParams {
        text,
        fg: style.foreground.clone(),
        bg: style.background.clone(),
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strike,
        blink: style.blink,
        reverse: style.reverse,
        hidden: style.hidden,
        dim: style.dim,
        ..Default::default()
      },
    );
  }

  // ─── Unified surface drawing API ──────────────────────────

  /// Draws text on the given surface.
  pub fn draw_text_on_surface(
    &mut self,
    canvas: &mut CanvasService,
    surface: SurfaceId,
    params: &DrawTextParams,
  ) -> bool {
    match surface {
      SurfaceId::Slice(id) => self.draw_text_on(canvas, id, params),
      SurfaceId::ScrollBox(id) => self.draw_text_in_scroll_box(canvas, id, params),
    }
  }

  /// Draws text at signed local coordinates of the given surface.
  pub fn draw_text_at_on_surface(
    &mut self,
    canvas: &mut CanvasService,
    surface: SurfaceId,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) -> bool {
    match surface {
      SurfaceId::Slice(id) => self.draw_text_at_on(canvas, id, x, y, params),
      SurfaceId::ScrollBox(id) => self.draw_text_at_in_scroll_box(canvas, id, x, y, params),
    }
  }

  /// Draws a filled rectangle on the given surface.
  // reason: public API; grouping the parameters would change its signature.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_filled_rect_on_surface(
    &mut self,
    canvas: &mut CanvasService,
    surface: SurfaceId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) -> bool {
    let (x, y) = (x.into(), y.into());
    match surface {
      SurfaceId::Slice(id) => {
        self.draw_filled_rect_on(canvas, id, x, y, width, height, fill_char, fill_fg, fill_bg)
      }
      SurfaceId::ScrollBox(id) => self.draw_filled_rect_in_scroll_box(
        canvas, id, x, y, width, height, fill_char, fill_fg, fill_bg,
      ),
    }
  }

  /// Draws a bordered rectangle on the given surface.
  // reason: public API; grouping the parameters would change its signature.
  #[allow(clippy::too_many_arguments)]
  pub fn draw_border_rect_on_surface(
    &mut self,
    canvas: &mut CanvasService,
    surface: SurfaceId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    width: u16,
    height: u16,
    border_style: &BorderStyle,
    border_fg: Option<TextColor>,
    border_bg: Option<TextColor>,
    fill_bg: Option<TextColor>,
    border_attrs: Option<TextStyle>,
  ) -> bool {
    let (x, y) = (x.into(), y.into());
    match surface {
      SurfaceId::Slice(id) => self.draw_border_rect_on(
        canvas,
        id,
        x,
        y,
        width,
        height,
        border_style,
        border_fg,
        border_bg,
        fill_bg,
        border_attrs,
      ),
      SurfaceId::ScrollBox(id) => self.draw_border_rect_in_scroll_box(
        canvas,
        id,
        x,
        y,
        width,
        height,
        border_style,
        border_fg,
        border_bg,
        fill_bg,
        border_attrs,
      ),
    }
  }

  fn draw_text_target(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    params: &DrawTextParams,
  ) {
    match target {
      Target::Base => canvas.text(params),
      Target::Slice(id) => {
        canvas.text_on(id, params);
      }
      Target::ScrollBox(id) => {
        canvas.text_in_scroll_box(id, params);
      }
      Target::Host => canvas.host_text(params),
      Target::Top => canvas.top_text(params),
    }
  }

  fn draw_text_target_at(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) {
    match target {
      Target::Base => canvas.text_at(x, y, params),
      Target::Slice(id) => {
        canvas.text_at_on(id, x, y, params);
      }
      Target::ScrollBox(id) => {
        canvas.text_at_in_scroll_box(id, x, y, params);
      }
      Target::Host => canvas.host_text_at(x, y, params),
      Target::Top => canvas.top_text_at(x, y, params),
    }
  }
}
