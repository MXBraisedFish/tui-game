use super::BorderStyle;
use crate::host_engine::services::unicode::char_width;
use crate::host_engine::services::{CanvasService, DrawTextParams, SliceId, TextColor, TextStyle};

#[derive(Clone, Copy)]
enum Target {
  Base,
  Slice(SliceId),
  Host,
}

/// 渲染服务：提供文本、填充矩形和边框矩形等高层绘制操作。
pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  /// 在基础层上绘制文本。
  pub fn draw_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    self.draw_text_target(canvas, Target::Base, params);
  }

  /// 在指定切片上绘制文本，返回是否绘制成功（切片不可见时返回 false）。
  pub fn draw_text_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    params: &DrawTextParams,
  ) -> bool {
    canvas.text_on(slice, params)
  }

  /// 在宿主层上绘制文本（用于顶层 UI 元素）。
  pub(crate) fn draw_host_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    self.draw_text_target(canvas, Target::Host, params);
  }

  /// 在基础层上绘制填充矩形。
  pub fn draw_filled_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) {
    self.draw_filled_rect_target(
      canvas,
      Target::Base,
      x,
      y,
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
  }

  /// 在指定切片上绘制填充矩形，返回是否绘制成功。
  #[allow(clippy::too_many_arguments)]
  pub fn draw_filled_rect_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    x: u16,
    y: u16,
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
      x,
      y,
      width,
      height,
      fill_char,
      fill_fg,
      fill_bg,
    );
    true
  }

  /// 在宿主层上绘制填充矩形。
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn draw_host_filled_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fill_fg: Option<TextColor>,
    fill_bg: Option<TextColor>,
  ) {
    self.draw_filled_rect_target(
      canvas,
      Target::Host,
      x,
      y,
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
    x: u16,
    y: u16,
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

    let fill_str: String = std::iter::repeat(ch).take(width as usize).collect();
    for row in 0..height {
      self.draw_text_target(
        canvas,
        target,
        &DrawTextParams {
          x,
          y: y.saturating_add(row),
          text: fill_str.clone(),
          fg: fill_fg.clone(),
          bg: fill_bg.clone(),
          ..Default::default()
        },
      );
    }
  }

  /// 在基础层上绘制带样式的边框矩形。
  pub fn draw_border_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
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
      x,
      y,
      width,
      height,
      border_style,
      border_fg,
      border_bg,
      fill_bg,
      border_attrs,
    );
  }

  /// 在指定切片上绘制带样式的边框矩形，返回是否绘制成功。
  #[allow(clippy::too_many_arguments)]
  pub fn draw_border_rect_on(
    &mut self,
    canvas: &mut CanvasService,
    slice: SliceId,
    x: u16,
    y: u16,
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
      x,
      y,
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

  /// 在宿主层上绘制带样式的边框矩形。
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn draw_host_border_rect(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
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
      x,
      y,
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
    x: u16,
    y: u16,
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

    let fill_dp = DrawTextParams {
      x: 0,
      y: 0,
      text: String::new(),
      bg: fill_bg.clone(),
      ..Default::default()
    };

    self.draw_border_cell(canvas, target, x, y, lt_ch, &lt_s);
    self.draw_border_span(canvas, target, x.saturating_add(1), y, t_ch, mid_w, &t_s);
    self.draw_border_cell(canvas, target, x.saturating_add(width - 1), y, rt_ch, &rt_s);

    let space_str: String = std::iter::repeat(' ').take(mid_w as usize).collect();
    for row in 1..=mid_h {
      let cy = y.saturating_add(row);
      self.draw_border_cell(canvas, target, x, cy, l_ch, &l_s);
      self.draw_text_target(
        canvas,
        target,
        &DrawTextParams {
          x: x.saturating_add(1),
          y: cy,
          text: space_str.clone(),
          bg: fill_dp.bg.clone(),
          ..Default::default()
        },
      );
      self.draw_border_cell(canvas, target, x.saturating_add(width - 1), cy, r_ch, &r_s);
    }

    let bot_y = y.saturating_add(height - 1);
    self.draw_border_cell(canvas, target, x, bot_y, lb_ch, &lb_s);
    self.draw_border_span(
      canvas,
      target,
      x.saturating_add(1),
      bot_y,
      b_ch,
      mid_w,
      &b_s,
    );
    self.draw_border_cell(
      canvas,
      target,
      x.saturating_add(width - 1),
      bot_y,
      rb_ch,
      &rb_s,
    );
  }

  fn draw_border_cell(
    &mut self,
    canvas: &mut CanvasService,
    target: Target,
    x: u16,
    y: u16,
    ch: char,
    style: &TextStyle,
  ) {
    self.draw_text_target(
      canvas,
      target,
      &DrawTextParams {
        x,
        y,
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
    x: u16,
    y: u16,
    ch: char,
    count: u16,
    style: &TextStyle,
  ) {
    let text: String = std::iter::repeat(ch).take(count as usize).collect();
    self.draw_text_target(
      canvas,
      target,
      &DrawTextParams {
        x,
        y,
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
      Target::Host => canvas.host_text(params),
    }
  }
}
