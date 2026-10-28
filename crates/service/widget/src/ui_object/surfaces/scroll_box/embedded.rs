//! Dynamic scrollbox geometry, frame ownership and scrollbar interaction.

use super::{ScrollbarAxis, scrollbar_thumb_rect};
use crate::{PointerScopeId, ScrollbarStyle, UiObjectPool};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, ScrollDirection};
use tg_core_style::{TextColor, TextStyle};
use tg_service_canvas::{CanvasService, SliceId};
use tg_service_layout::{Rect, Size};

/// Configuration of a dynamically embedded scrollbox.
///
/// # Fields
///
/// * `width`, `height` - The configured frame or content dimensions.
/// * `background` - The background painted behind content and scrollbars.
/// * `mouse` - The component's own pointer input switch.
/// * `wheel_vertical`, `wheel_horizontal` - The accepted wheel directions.
/// * `scrollbar_swap_vertical`, `scrollbar_swap_horizontal` - The scrollbar placement switches.
/// * `include_scrollbar`, `always_show_scrollbar` - The sizing and visibility policies.
/// * `wheel_step_vertical`, `wheel_step_horizontal` - The wheel distances in cells.
/// * `style` - The characters and text styles used by the scrollbars.
#[derive(Clone, Debug)]
pub struct ScrollBoxWidgetOptions {
  /// The requested outer or content width, selected by `include_scrollbar`.
  pub width: u16,
  /// The requested outer or content height, selected by `include_scrollbar`.
  pub height: u16,
  /// The frame background.
  pub background: Option<TextColor>,
  /// Whether this scrollbox accepts wheel and scrollbar input.
  pub mouse: bool,
  /// Whether vertical wheel events are enabled.
  pub wheel_vertical: bool,
  /// Whether horizontal wheel events are enabled.
  pub wheel_horizontal: bool,
  /// Whether the vertical scrollbar is placed on the left.
  pub scrollbar_swap_vertical: bool,
  /// Whether the horizontal scrollbar is placed on the top.
  pub scrollbar_swap_horizontal: bool,
  /// Whether requested dimensions include visible scrollbars.
  pub include_scrollbar: bool,
  /// Whether both scrollbars are shown without overflow.
  pub always_show_scrollbar: bool,
  /// The number of rows per wheel step.
  pub wheel_step_vertical: u16,
  /// The number of columns per wheel step.
  pub wheel_step_horizontal: u16,
  /// The track/thumb characters and styling.
  pub style: ScrollbarStyle,
}
impl Default for ScrollBoxWidgetOptions {
  fn default() -> Self {
    Self {
      width: 1,
      height: 1,
      background: None,
      mouse: true,
      wheel_vertical: true,
      wheel_horizontal: true,
      scrollbar_swap_vertical: false,
      scrollbar_swap_horizontal: false,
      include_scrollbar: true,
      always_show_scrollbar: false,
      wheel_step_vertical: 2,
      wheel_step_horizontal: 3,
      style: ScrollbarStyle::default(),
    }
  }
}
/// Resolved scrollbox dimensions and local scrollbar rectangles.
///
/// # Fields
///
/// * `outer` - The entire component dimensions.
/// * `content` - The local visible content rectangle.
/// * `vertical`, `horizontal` - The optional track and thumb rectangles.
/// * `maximum` - The largest horizontal and vertical offsets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollBoxWidgetLayout {
  /// The complete frame size.
  pub outer: Size,
  /// The visible content rectangle within the frame.
  pub content: Rect,
  /// The vertical track and thumb.
  pub vertical: Option<(Rect, Rect)>,
  /// The horizontal track and thumb.
  pub horizontal: Option<(Rect, Rect)>,
  /// The largest horizontal and vertical offsets.
  pub maximum: (u16, u16),
}
/// A clipped content view returned after registering the frame in its parent scope.
///
/// # Fields
///
/// * `clip` - The physical rectangle remaining after ancestor clipping.
/// * `origin` - The physical position of logical content coordinate zero.
/// * `rank` - The interaction rank inherited from the root layer.
#[derive(Clone, Copy, Debug)]
pub struct ScrollBoxContentView {
  /// The physical ancestor-clipped content rectangle.
  pub clip: Rect,
  /// The physical position of unscrolled content coordinate zero.
  pub origin: (i32, i32),
  /// The inherited root layer rank.
  pub rank: usize,
}
#[derive(Clone, Copy)]
struct Frame {
  clip: Rect,
  parent_clip: Rect,
  origin: (i32, i32),
  rank: usize,
  order: u64,
}
#[derive(Clone, Copy)]
struct Drag {
  axis: ScrollbarAxis,
  anchor: i32,
}
/// An independent scrollbox component with a dynamically embedded content surface.
pub struct ScrollBoxWidget {
  scope: PointerScopeId,
  options: ScrollBoxWidgetOptions,
  extent: Size,
  measured: bool,
  offset: (u16, u16),
  pending: Option<(i64, i64)>,
  event: Option<(u16, u16)>,
  frame: Option<Frame>,
  drag: Option<Drag>,
}
impl ScrollBoxWidget {
  /// Create a scrollbox when its dimensions, steps and scrollbar characters are valid.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tg_service_widget::{ScrollBoxWidget,ScrollBoxWidgetOptions,UiObjectPool};
  /// use tg_service_layout::Size;
  /// let mut pool=UiObjectPool::new();
  /// let mut widget=ScrollBoxWidget::new(&mut pool,ScrollBoxWidgetOptions {
  ///   width:6,height:4,..Default::default()
  /// }).unwrap();
  /// widget.set_extent(Size{width:20,height:10});
  /// widget.scroll_to(65535,65535);
  /// assert_eq!(widget.offset(),(15,7));
  /// ```
  pub fn new(pool: &mut UiObjectPool, options: ScrollBoxWidgetOptions) -> Option<Self> {
    valid(&options).then(|| Self {
      scope: pool.allocate_pointer_scope(),
      options,
      extent: Size::default(),
      measured: false,
      offset: (0, 0),
      pending: None,
      event: None,
      frame: None,
      drag: None,
    })
  }
  /// Return the component's pointer ownership scope.
  pub fn scope(&self) -> PointerScopeId {
    self.scope
  }
  /// Return its current configuration.
  pub fn options(&self) -> &ScrollBoxWidgetOptions {
    &self.options
  }
  /// Replace valid configuration and cancel obsolete pointer capture.
  pub fn set_options(&mut self, options: ScrollBoxWidgetOptions) -> bool {
    if !valid(&options) {
      return false;
    }
    self.options = options;
    self.frame = None;
    self.drag = None;
    self.clamp();
    true
  }
  /// Return the complete logical content extent from the last frame.
  pub fn extent(&self) -> Size {
    self.extent
  }
  /// Return the current horizontal and vertical offsets.
  pub fn offset(&self) -> (u16, u16) {
    self.offset
  }
  /// Resolve dimensions against an explicitly measured content extent.
  pub fn layout_for(&self, extent: Size) -> ScrollBoxWidgetLayout {
    resolve(&self.options, extent, self.offset)
  }
  /// Return the current frame layout.
  pub fn layout(&self) -> ScrollBoxWidgetLayout {
    self.layout_for(self.extent)
  }
  /// Replace measured content bounds and apply pending absolute or endpoint scrolling.
  pub fn set_extent(&mut self, extent: Size) {
    self.measured = true;
    self.extent = extent;
    let desired = self
      .pending
      .take()
      .unwrap_or((i64::from(self.offset.0), i64::from(self.offset.1)));
    self.apply(desired.0, desired.1);
  }
  /// Set absolute cell offsets, retaining the request for the next content measurement.
  pub fn scroll_to(&mut self, x: i64, y: i64) {
    self.pending = Some((x, y));
    self.apply(x, y);
  }
  /// Replace selected absolute offsets while preserving pending requests on the other axis.
  pub fn scroll_to_axes(&mut self, horizontal: Option<i64>, vertical: Option<i64>) {
    let (x, y) = self
      .pending
      .unwrap_or((i64::from(self.offset.0), i64::from(self.offset.1)));
    self.scroll_to(horizontal.unwrap_or(x), vertical.unwrap_or(y));
  }
  /// Add signed cell offsets without arithmetic overflow.
  pub fn scroll_by(&mut self, x: i64, y: i64) {
    let current = (i64::from(self.offset.0), i64::from(self.offset.1));
    let base = if self.measured {
      current
    } else {
      self.pending.unwrap_or(current)
    };
    self.scroll_to(base.0.saturating_add(x), base.1.saturating_add(y));
  }
  /// Drain the latest coalesced scroll position change.
  pub fn take_scroll_event(&mut self) -> Option<(u16, u16)> {
    self.event.take()
  }
  /// Forget frame geometry while preserving configuration, offsets and active drag anchoring.
  pub fn begin_frame(&mut self) {
    self.frame = None;
  }
  /// Cancel pointer capture when focus or interaction ownership is lost.
  pub fn deactivate(&mut self) {
    self.frame = None;
    self.drag = None;
  }
  fn clamp(&mut self) {
    self.apply(i64::from(self.offset.0), i64::from(self.offset.1));
  }
  fn apply(&mut self, x: i64, y: i64) {
    let max = self.layout().maximum;
    let next = (
      x.clamp(0, i64::from(max.0)) as u16,
      y.clamp(0, i64::from(max.1)) as u16,
    );
    if next != self.offset {
      self.offset = next;
      self.event = Some(next);
    }
  }
  /// Paint and register this frame, entering its child pointer scope on success.
  ///
  /// Pair a successful call with [`UiObjectPool::end_pointer_scope`] after child replay.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning interaction pool.
  /// * `canvas` - The destination canvas.
  /// * `parent` - The parent slice, or the base.
  /// * `position` - The signed parent-local frame origin.
  pub fn begin_draw(
    &mut self,
    pool: &mut UiObjectPool,
    canvas: &mut CanvasService,
    parent: Option<SliceId>,
    position: (i32, i32),
  ) -> Option<ScrollBoxContentView> {
    let (parent_clip, parent_origin, rank) = canvas.surface_geometry(parent)?;
    let origin = (
      i64::from(parent_origin.0) + i64::from(position.0),
      i64::from(parent_origin.1) + i64::from(position.1),
    );
    let layout = self.layout();
    let clip = clip_rect(origin, layout.outer, parent_clip);
    if clip.width == 0 || clip.height == 0 {
      self.frame = None;
      return None;
    }
    let origin = (origin.0 as i32, origin.1 as i32);
    let content_origin = (
      origin.0 + i32::from(layout.content.x),
      origin.1 + i32::from(layout.content.y),
    );
    let content_clip = clip_rect(
      (i64::from(content_origin.0), i64::from(content_origin.1)),
      Size {
        width: layout.content.width,
        height: layout.content.height,
      },
      parent_clip,
    );
    if !pool.begin_pointer_scope(self.scope, content_clip) {
      return None;
    }
    let order = pool.register_pointer_region(rank, clip);
    self.frame = Some(Frame {
      clip,
      parent_clip,
      origin,
      rank,
      order,
    });
    paint(
      canvas,
      parent,
      clip,
      parent_origin,
      ' ',
      TextStyle {
        background: self.options.background.clone(),
        ..Default::default()
      },
    );
    for (track, thumb, vertical) in layout
      .vertical
      .map(|(a, b)| (a, b, true))
      .into_iter()
      .chain(layout.horizontal.map(|(a, b)| (a, b, false)))
    {
      let style = &self.options.style;
      for (rect, ch, mut text_style) in [
        (
          track,
          if vertical {
            style.track_char
          } else {
            style.h_track_char
          },
          if vertical {
            style.track_style.clone()
          } else {
            style.h_track_style.clone()
          },
        ),
        (
          thumb,
          if vertical {
            style.thumb_char
          } else {
            style.h_thumb_char
          },
          if vertical {
            style.thumb_style.clone()
          } else {
            style.h_thumb_style.clone()
          },
        ),
      ] {
        if text_style.background.is_none() {
          text_style.background.clone_from(&self.options.background);
        }
        let rect = clip_rect(
          (
            i64::from(origin.0) + i64::from(rect.x),
            i64::from(origin.1) + i64::from(rect.y),
          ),
          Size {
            width: rect.width,
            height: rect.height,
          },
          clip,
        );
        paint(canvas, parent, rect, parent_origin, ch, text_style);
      }
    }
    Some(ScrollBoxContentView {
      clip: content_clip,
      origin: (
        content_origin.0 - i32::from(self.offset.0),
        content_origin.1 - i32::from(self.offset.1),
      ),
      rank,
    })
  }
  /// Return the current root rank and painting order for front-to-back event routing.
  pub fn frame_order(&self) -> Option<(usize, u64)> {
    self.frame.map(|frame| (frame.rank, frame.order))
  }
  /// Apply pointer input when the top control belongs to this scope or one of its descendants.
  pub fn route_mouse(&mut self, pool: &UiObjectPool, event: MouseEvent) -> bool {
    let Some(frame) = self.frame else {
      self.drag = None;
      return false;
    };
    if !self.options.mouse {
      self.drag = None;
      return false;
    }
    let top = pool.pointer_order_at(event.x, event.y);
    let owns = top.is_some_and(|(rank, order)| {
      rank == frame.rank && (order == frame.order || pool.scope_contains_order(self.scope, order))
    });
    let unblocked = frame.parent_clip.contains(event.x, event.y)
      && top.is_none_or(|(rank, order)| {
        rank < frame.rank
          || (rank == frame.rank
            && (order <= frame.order || pool.scope_contains_order(self.scope, order)))
      });
    if event.kind == MouseEventKind::Release {
      return self.drag.take().is_some();
    }
    if matches!(event.kind, MouseEventKind::Drag | MouseEventKind::Hold)
      && let Some(drag) = self.drag
    {
      if !unblocked {
        self.drag = None;
        return false;
      }
      self.drag_to(frame, event, drag);
      return true;
    }
    if !owns || !frame.clip.contains(event.x, event.y) {
      return false;
    }
    if event.kind == MouseEventKind::Scroll {
      let layout = self.layout();
      let (x, y) = match event.scroll {
        Some(ScrollDirection::Up) if self.options.wheel_vertical && layout.maximum.1 > 0 => {
          (0, -i64::from(self.options.wheel_step_vertical))
        }
        Some(ScrollDirection::Down) if self.options.wheel_vertical && layout.maximum.1 > 0 => {
          (0, i64::from(self.options.wheel_step_vertical))
        }
        Some(ScrollDirection::Left) if self.options.wheel_horizontal && layout.maximum.0 > 0 => {
          (-i64::from(self.options.wheel_step_horizontal), 0)
        }
        Some(ScrollDirection::Right) if self.options.wheel_horizontal && layout.maximum.0 > 0 => {
          (i64::from(self.options.wheel_step_horizontal), 0)
        }
        _ => return false,
      };
      self.scroll_by(x, y);
      return true;
    }
    if event.kind == MouseEventKind::Press && event.button == Some(MouseButton::Left) {
      for (axis, pair) in [
        (ScrollbarAxis::Vertical, self.layout().vertical),
        (ScrollbarAxis::Horizontal, self.layout().horizontal),
      ] {
        let Some((track, thumb)) = pair else {
          continue;
        };
        let local = (
          i32::from(event.x) - frame.origin.0,
          i32::from(event.y) - frame.origin.1,
        );
        if let (Ok(x), Ok(y)) = (u16::try_from(local.0), u16::try_from(local.1))
          && track.contains(x, y)
        {
          let (point, start, length) = match axis {
            ScrollbarAxis::Vertical => (local.1, i32::from(thumb.y), i32::from(thumb.height)),
            ScrollbarAxis::Horizontal => (local.0, i32::from(thumb.x), i32::from(thumb.width)),
          };
          let drag = Drag {
            axis,
            anchor: if thumb.contains(x, y) {
              point - start
            } else {
              length / 2
            },
          };
          self.drag = Some(drag);
          self.drag_to(frame, event, drag);
          return true;
        }
      }
    }
    false
  }
  fn drag_to(&mut self, frame: Frame, event: MouseEvent, drag: Drag) {
    let layout = self.layout();
    let Some((track, thumb)) = (match drag.axis {
      ScrollbarAxis::Vertical => layout.vertical,
      ScrollbarAxis::Horizontal => layout.horizontal,
    }) else {
      return;
    };
    let (point, start, travel, max) = match drag.axis {
      ScrollbarAxis::Vertical => (
        i64::from(event.y) - i64::from(frame.origin.1),
        i64::from(track.y),
        i64::from(track.height.saturating_sub(thumb.height)),
        layout.maximum.1,
      ),
      ScrollbarAxis::Horizontal => (
        i64::from(event.x) - i64::from(frame.origin.0),
        i64::from(track.x),
        i64::from(track.width.saturating_sub(thumb.width)),
        layout.maximum.0,
      ),
    };
    let value = if travel == 0 {
      0
    } else {
      ((point - start - i64::from(drag.anchor)).clamp(0, travel) * i64::from(max) + travel / 2)
        / travel
    };
    match drag.axis {
      ScrollbarAxis::Vertical => self.scroll_to(i64::from(self.offset.0), value),
      ScrollbarAxis::Horizontal => self.scroll_to(value, i64::from(self.offset.1)),
    }
  }
}
fn valid(options: &ScrollBoxWidgetOptions) -> bool {
  options.width > 0
    && options.height > 0
    && options.wheel_step_vertical > 0
    && options.wheel_step_horizontal > 0
    && [
      options.style.track_char,
      options.style.thumb_char,
      options.style.h_track_char,
      options.style.h_thumb_char,
    ]
    .into_iter()
    .all(|ch| tg_core_unicode::char_width(ch) == 1)
}
fn resolve(
  options: &ScrollBoxWidgetOptions,
  extent: Size,
  offset: (u16, u16),
) -> ScrollBoxWidgetLayout {
  let mut vertical = options.always_show_scrollbar;
  let mut horizontal = vertical;
  for _ in 0..4 {
    let width = options
      .width
      .saturating_sub(u16::from(options.include_scrollbar && vertical));
    let height = options
      .height
      .saturating_sub(u16::from(options.include_scrollbar && horizontal));
    let next = (
      options.always_show_scrollbar || extent.height > height,
      options.always_show_scrollbar || extent.width > width,
    );
    if next == (vertical, horizontal) {
      break;
    }
    (vertical, horizontal) = next;
  }
  let view = Size {
    width: options
      .width
      .saturating_sub(u16::from(options.include_scrollbar && vertical)),
    height: options
      .height
      .saturating_sub(u16::from(options.include_scrollbar && horizontal)),
  };
  let outer = Size {
    width: options
      .width
      .saturating_add(u16::from(!options.include_scrollbar && vertical)),
    height: options
      .height
      .saturating_add(u16::from(!options.include_scrollbar && horizontal)),
  };
  let content = Rect {
    x: u16::from(vertical && options.scrollbar_swap_vertical),
    y: u16::from(horizontal && options.scrollbar_swap_horizontal),
    width: view.width,
    height: view.height,
  };
  let maximum = (
    if view.width == 0 {
      0
    } else {
      extent.width.saturating_sub(view.width)
    },
    if view.height == 0 {
      0
    } else {
      extent.height.saturating_sub(view.height)
    },
  );
  let vertical = vertical.then(|| {
    let track = Rect {
      x: if options.scrollbar_swap_vertical {
        0
      } else {
        outer.width.saturating_sub(1)
      },
      y: content.y,
      width: 1,
      height: view.height,
    };
    let thumb = scrollbar_thumb_rect(
      track,
      ScrollbarAxis::Vertical,
      offset.1,
      maximum.1,
      extent.height,
      options.style.minimum_thumb_height,
    );
    (track, thumb)
  });
  let horizontal = horizontal.then(|| {
    let track = Rect {
      x: content.x,
      y: if options.scrollbar_swap_horizontal {
        0
      } else {
        outer.height.saturating_sub(1)
      },
      width: view.width,
      height: 1,
    };
    let thumb = scrollbar_thumb_rect(
      track,
      ScrollbarAxis::Horizontal,
      offset.0,
      maximum.0,
      extent.width,
      options.style.minimum_thumb_height,
    );
    (track, thumb)
  });
  ScrollBoxWidgetLayout {
    outer,
    content,
    vertical,
    horizontal,
    maximum,
  }
}
/// Intersect a signed logical rectangle with a physical ancestor clip.
///
/// # Arguments
///
/// * `origin` - The signed physical top-left position of the logical rectangle.
/// * `size` - The logical dimensions in terminal cells.
/// * `clip` - The physical ancestor rectangle.
pub fn clip_rect(origin: (i64, i64), size: Size, clip: Rect) -> Rect {
  let left = origin
    .0
    .max(i64::from(clip.x))
    .min(i64::from(clip.x) + i64::from(clip.width));
  let top = origin
    .1
    .max(i64::from(clip.y))
    .min(i64::from(clip.y) + i64::from(clip.height));
  let right = (origin.0 + i64::from(size.width))
    .min(i64::from(clip.x) + i64::from(clip.width))
    .max(left);
  let bottom = (origin.1 + i64::from(size.height))
    .min(i64::from(clip.y) + i64::from(clip.height))
    .max(top);
  Rect {
    x: left as u16,
    y: top as u16,
    width: (right - left) as u16,
    height: (bottom - top) as u16,
  }
}

fn paint(
  canvas: &mut CanvasService,
  parent: Option<SliceId>,
  rect: Rect,
  origin: (i32, i32),
  ch: char,
  style: TextStyle,
) {
  let text = ch.to_string().repeat(rect.width as usize);
  for y in 0..rect.height {
    let x = i32::from(rect.x) - origin.0;
    let y = i32::from(rect.y) + i32::from(y) - origin.1;
    match parent {
      None => canvas.styled_text(x, y, &text, style.clone()),
      Some(id) => {
        canvas.styled_text_on(id, x, y, &text, style.clone());
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn one_bar_can_require_the_other_and_empty_content_hides_both() {
    let mut pool = UiObjectPool::new();
    let mut widget = ScrollBoxWidget::new(
      &mut pool,
      ScrollBoxWidgetOptions {
        width: 5,
        height: 3,
        ..Default::default()
      },
    )
    .unwrap();
    widget.set_extent(Size {
      width: 5,
      height: 4,
    });
    assert_eq!(
      widget.layout().content,
      Rect {
        x: 0,
        y: 0,
        width: 4,
        height: 2
      }
    );
    assert!(widget.layout().vertical.is_some() && widget.layout().horizontal.is_some());
    widget.scroll_to(65535, 65535);
    widget.set_extent(Size::default());
    assert_eq!(widget.offset(), (0, 0));
    assert_eq!(
      widget.layout().content,
      Rect {
        x: 0,
        y: 0,
        width: 5,
        height: 3
      }
    );
  }

  #[test]
  fn dragging_uses_thumb_anchor_and_cancels_on_deactivation() {
    let mut pool = UiObjectPool::new();
    let mut widget = ScrollBoxWidget::new(
      &mut pool,
      ScrollBoxWidgetOptions {
        width: 6,
        height: 6,
        ..Default::default()
      },
    )
    .unwrap();
    widget.set_extent(Size {
      width: 1,
      height: 60,
    });
    let mut layout = tg_service_layout::LayoutService::new();
    layout.resize_physical(12, 10);
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    pool.begin_render();
    pool.prepare_canvas(&mut canvas, &layout);
    widget
      .begin_draw(&mut pool, &mut canvas, None, (1, 1))
      .unwrap();
    pool.end_pointer_scope();
    let mouse = |kind, x, y| MouseEvent {
      kind,
      x,
      y,
      button: Some(MouseButton::Left),
      scroll: None,
    };
    assert!(widget.route_mouse(&pool, mouse(MouseEventKind::Press, 6, 1)));
    assert_eq!(widget.offset(), (0, 0));
    assert!(widget.route_mouse(&pool, mouse(MouseEventKind::Drag, 7, 6)));
    assert_eq!(widget.offset(), (0, 54));
    assert!(widget.route_mouse(&pool, mouse(MouseEventKind::Release, 7, 6)));
    assert!(!widget.route_mouse(&pool, mouse(MouseEventKind::Drag, 7, 1)));
    widget.deactivate();
    assert!(!widget.route_mouse(&pool, mouse(MouseEventKind::Press, 6, 1)));
  }

  #[test]
  fn relative_scroll_reverses_immediately_after_clamping() {
    let mut pool = UiObjectPool::new();
    let mut widget = ScrollBoxWidget::new(
      &mut pool,
      ScrollBoxWidgetOptions {
        width: 5,
        height: 3,
        ..Default::default()
      },
    )
    .unwrap();
    widget.scroll_by(3, 2);
    widget.scroll_by(3, 2);
    widget.set_extent(Size {
      width: 20,
      height: 10,
    });
    assert_eq!(widget.offset(), (6, 4));
    widget.scroll_to(65535, 65535);
    widget.scroll_by(-1, -1);
    assert_eq!(widget.offset(), (15, 7));
    widget.scroll_by(100, 100);
    widget.scroll_by(-1, -1);
    assert_eq!(widget.offset(), (15, 7));
  }

  #[test]
  fn pending_axes_are_independent_and_one_cell_frames_remain_valid() {
    let mut pool = UiObjectPool::new();
    let mut widget = ScrollBoxWidget::new(
      &mut pool,
      ScrollBoxWidgetOptions {
        width: 1,
        height: 1,
        always_show_scrollbar: true,
        ..Default::default()
      },
    )
    .unwrap();
    widget.scroll_to_axes(Some(65535), None);
    widget.scroll_to_axes(None, Some(65535));
    widget.set_extent(Size {
      width: 100,
      height: 100,
    });
    assert_eq!(widget.offset(), (0, 0));
    assert_eq!(widget.layout().content.width, 0);
    let mut options = widget.options().clone();
    options.width = 5;
    options.height = 5;
    widget.set_options(options);
    widget.scroll_to_axes(Some(65535), None);
    widget.scroll_to_axes(None, Some(65535));
    widget.set_extent(Size {
      width: 100,
      height: 100,
    });
    assert_eq!(widget.offset(), (96, 96));
  }
}
