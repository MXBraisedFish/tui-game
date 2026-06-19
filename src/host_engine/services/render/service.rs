use super::BorderStyle;
use crate::host_engine::services::unicode::char_width;
use crate::host_engine::services::{CanvasService, DrawTextParams, TextColor, TextStyle};

/// 渲染服务 —— 当前为薄壳，直接委托给 CanvasService。
///
/// 未来可在此层加入视口裁剪、坐标变换等宿主侧渲染逻辑。
pub struct RenderService;

impl RenderService {
  pub fn new() -> Self {
    Self
  }

  /// 唯一的绘制入口。
  /// 委托给 `canvas.text()`，由其内部完成 f% 路由和样式解析。
  pub fn draw_text(&mut self, canvas: &mut CanvasService, params: &DrawTextParams) {
    canvas.text(params);
  }

  // ── 矩形绘制 ──

  /// 绘制填充矩形。
  ///
  /// `fill_char`: 填充字符（可选）。空串→空格；长度>1→取首字符并校验显示宽度==1，否则回退空格。
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
    let ch = match fill_char {
      Some(ref s) if !s.is_empty() => {
        let c = s.chars().next().unwrap_or(' ');
        if char_width(c) == 1 { c } else { ' ' }
      }
      _ => ' ',
    };

    let fill_str: String = std::iter::repeat(ch).take(width as usize).collect();
    for row in 0..height {
      self.draw_text(
        canvas,
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

  /// 绘制带边框矩形。
  ///
  /// 边框样式可为固定样式（Line/Bold/Double/Circle/None）或自定义 8 位置表。
  /// `border_attrs` 为边框字符的 TextStyle 属性（bold/italic 等），
  /// 会被各位置的 per-position style 覆盖。
  /// `fill_bg` 控制矩形内部的背景色。
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

    // 解析各位置最终样式
    let lt_s = custom.left_top.resolve(fg_ref, bg_ref, attrs_ref);
    let t_s = custom.top.resolve(fg_ref, bg_ref, attrs_ref);
    let rt_s = custom.right_top.resolve(fg_ref, bg_ref, attrs_ref);
    let r_s = custom.right.resolve(fg_ref, bg_ref, attrs_ref);
    let rb_s = custom.right_bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let b_s = custom.bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let lb_s = custom.left_bottom.resolve(fg_ref, bg_ref, attrs_ref);
    let l_s = custom.left.resolve(fg_ref, bg_ref, attrs_ref);

    // 边框字符
    let lt_ch = custom.left_top.char.unwrap_or(' ');
    let t_ch = custom.top.char.unwrap_or(' ');
    let rt_ch = custom.right_top.char.unwrap_or(' ');
    let r_ch = custom.right.char.unwrap_or(' ');
    let rb_ch = custom.right_bottom.char.unwrap_or(' ');
    let b_ch = custom.bottom.char.unwrap_or(' ');
    let lb_ch = custom.left_bottom.char.unwrap_or(' ');
    let l_ch = custom.left.char.unwrap_or(' ');

    // 内部填充空格样式
    let fill_dp = DrawTextParams {
      x: 0,
      y: 0,
      text: String::new(),
      bg: fill_bg.clone(),
      ..Default::default()
    };

    // 顶行
    self.draw_border_cell(canvas, x, y, lt_ch, &lt_s);
    self.draw_border_span(canvas, x.saturating_add(1), y, t_ch, mid_w, &t_s);
    self.draw_border_cell(canvas, x.saturating_add(width - 1), y, rt_ch, &rt_s);

    // 中间行
    let space_str: String = std::iter::repeat(' ').take(mid_w as usize).collect();
    for row in 1..=mid_h {
      let cy = y.saturating_add(row);
      self.draw_border_cell(canvas, x, cy, l_ch, &l_s);
      self.draw_text(
        canvas,
        &DrawTextParams {
          x: x.saturating_add(1),
          y: cy,
          text: space_str.clone(),
          bg: fill_dp.bg.clone(),
          ..Default::default()
        },
      );
      self.draw_border_cell(canvas, x.saturating_add(width - 1), cy, r_ch, &r_s);
    }

    // 底行
    let bot_y = y.saturating_add(height - 1);
    self.draw_border_cell(canvas, x, bot_y, lb_ch, &lb_s);
    self.draw_border_span(canvas, x.saturating_add(1), bot_y, b_ch, mid_w, &b_s);
    self.draw_border_cell(canvas, x.saturating_add(width - 1), bot_y, rb_ch, &rb_s);
  }

  // ── 内部辅助 ──

  /// 绘制单个边框字符。
  fn draw_border_cell(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    ch: char,
    style: &TextStyle,
  ) {
    self.draw_text(
      canvas,
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

  /// 绘制重复的边框跨度（用于水平边框线）。
  fn draw_border_span(
    &mut self,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    ch: char,
    count: u16,
    style: &TextStyle,
  ) {
    let text: String = std::iter::repeat(ch).take(count as usize).collect();
    self.draw_text(
      canvas,
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
}
