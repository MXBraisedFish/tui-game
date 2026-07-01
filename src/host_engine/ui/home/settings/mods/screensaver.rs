use std::{cmp::Ordering, time::Duration};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, ImageConvertParams, ImageService, KeyState,
  LayoutService, MouseButton, Overflow, PackageListEntry, PackageService, Rect, RenderService,
  RichTextParams, RichTextService, ScrollBoxId, ScrollBoxOptions, ScrollBoxService,
  ScrollbarPolicy, ScrollbarVisibility, TerminalColor, TextAlign, TextColor, TextInputCursorShape,
  TextInputEvent, TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams,
  TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

/// 屏保包详情页面的命令。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreensaverPackageCommand {
  Back,
  FocusSearch,
  BlurSearch,
  FocusJump,
  BlurJump,
  SubmitJump(String),
}

/// 屏保包详情页面布局信息。
pub(crate) struct ScreensaverPackageLayout {
  pub left_rect: Rect,
  pub left_inner: Rect,
  pub right_rect: Rect,
  pub right_inner: Rect,
  pub search_rect: Rect,
  pub sort_bar_y: u16,
  pub order_rect: Rect,
  pub sort_rect: Rect,
  pub list_area_y: u16,
  pub list_area_height: u16,
  pub list_start_y: u16,
  pub list_item_height: u16,
  pub visible_items: usize,
  pub page_y: u16,
  pub flip_forward_rect: Rect,
  pub flip_backward_rect: Rect,
  pub jump_rect: Rect,
  pub page_separator_x: u16,
  pub total_page_x: u16,
  pub hint_x: u16,
  pub hint_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreensaverSortField {
  Title,
  Author,
  Status,
  Debug,
}

impl ScreensaverSortField {
  fn next(self) -> Self {
    match self {
      Self::Title => Self::Author,
      Self::Author => Self::Status,
      Self::Status => Self::Debug,
      Self::Debug => Self::Title,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Title => "screensaver_pack.list.sort.title",
      Self::Author => "screensaver_pack.list.sort.author",
      Self::Status => "screensaver_pack.list.sort.status",
      Self::Debug => "screensaver_pack.list.sort.debug",
    }
  }
}

/// 屏保包详情 UI：左右 33/67 分栏布局。
///
/// 左侧：搜索框 + 列表（翻页） + 翻页指示器，包裹在双线边框内。
/// 右侧：滚动信息盒，包裹在双线边框内。
/// 底部：操作提示栏。
pub struct ScreensaverPackageUi {
  objects: UiObjectPool,
  search_input: TextInputId,
  jump_input: TextInputId,
  info_scroll: ScrollBoxId,
  page_area: HitAreaId,
  flip_forward_area: HitAreaId,
  flip_backward_area: HitAreaId,
  order_area: HitAreaId,
  sort_area: HitAreaId,
  list_item_areas: Vec<HitAreaId>,
  selected_index: usize,
  page: usize,
  per_page: usize,
  ascending: bool,
  sort_field: ScreensaverSortField,
  entries: Vec<PackageListEntry>,
  search_text: String,
  jump_text: String,
  needs_rebuild_areas: bool,
}

impl UiObjectPoolOwner for ScreensaverPackageUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl ScreensaverPackageUi {
  /// 初始化屏保包详情 UI。
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();

    let search_input = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text: String::new(),
        max_chars: Some(64),
        mode: TextInputMode::SingleLine,
        mouse: true,
      },
    );
    let jump_input = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text: "1".to_string(),
        max_chars: Some(4),
        mode: TextInputMode::SingleLine,
        mouse: true,
      },
    );

    let info_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect::default(),
          content_width: 40,
          content_height: 60,
          overflow_y: Overflow::Auto,
          overflow_x: Overflow::Hidden,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .expect("failed to create screensaver info scroll box");

    let page_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_forward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_backward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let order_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let sort_area = hit_area.create(&mut objects, HitAreaOptions::default());

    Self {
      objects,
      search_input,
      jump_input,
      info_scroll,
      page_area,
      flip_forward_area,
      flip_backward_area,
      order_area,
      sort_area,
      list_item_areas: Vec::new(),
      selected_index: 0,
      page: 1,
      per_page: 1,
      ascending: true,
      sort_field: ScreensaverSortField::Title,
      entries: Vec::new(),
      search_text: String::new(),
      jump_text: "1".to_string(),
      needs_rebuild_areas: true,
    }
  }

  /// 返回按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "screensaver_pack.flip_forward".to_string(),
        description: "Previous list page".to_string(),
        keys: vec![vec!["q".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.flip_backward".to_string(),
        description: "Next list page".to_string(),
        keys: vec![vec!["e".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.scroll_up".to_string(),
        description: "Scroll info up".to_string(),
        keys: vec![vec!["w".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.scroll_down".to_string(),
        description: "Scroll info down".to_string(),
        keys: vec![vec!["s".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.focus_up".to_string(),
        description: "Focus previous item".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.focus_down".to_string(),
        description: "Focus next item".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.confirm".to_string(),
        description: "Toggle selection".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.list.back".to_string(),
        description: "Go back to mods menu".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.debug".to_string(),
        description: "Toggle debug mode".to_string(),
        keys: vec![vec!["n".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.list".to_string(),
        description: "Toggle list style".to_string(),
        keys: vec![vec!["l".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.search".to_string(),
        description: "Search".to_string(),
        keys: vec![vec!["c".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.order".to_string(),
        description: "Toggle order".to_string(),
        keys: vec![vec!["z".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.sort".to_string(),
        description: "Toggle sort".to_string(),
        keys: vec![vec!["x".to_string()]],
      },
      ActionMapEntry {
        action: "screensaver_pack.jump".to_string(),
        description: "Jump to page".to_string(),
        keys: vec![vec!["j".to_string()]],
      },
    ]
  }

  /// 处理 UI 事件，返回导航命令。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ScreensaverPackageCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.handle_hover(*id);
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        Some(ScreensaverPackageCommand::FocusSearch)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.search_input => {
        Some(ScreensaverPackageCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        Some(ScreensaverPackageCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search_text = value.clone();
        self.page = 1;
        self.selected_index = 0;
        self.needs_rebuild_areas = true;
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.jump_input => {
        Some(ScreensaverPackageCommand::FocusJump)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.jump_input => {
        Some(ScreensaverPackageCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.jump_input => {
        Some(ScreensaverPackageCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.jump_input => {
        Some(ScreensaverPackageCommand::SubmitJump(value.clone()))
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.jump_input => {
        self.jump_text = value.clone();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        if *id == self.flip_forward_area {
          self.flip_page(-1);
          self.needs_rebuild_areas = true;
          return None;
        }
        if *id == self.flip_backward_area {
          self.flip_page(1);
          self.needs_rebuild_areas = true;
          return None;
        }
        if *id == self.order_area {
          self.toggle_order();
          return None;
        }
        if *id == self.sort_area {
          self.next_sort_field();
          return None;
        }
        if let Some(pos) = self.list_item_areas.iter().position(|a| a == id) {
          self.selected_index = pos;
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        button: MouseButton::Right,
        ..
      }) => Some(ScreensaverPackageCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "screensaver_pack.focus_up" => {
          self.focus_previous();
          None
        }
        "screensaver_pack.focus_down" => {
          self.focus_next();
          None
        }
        "screensaver_pack.flip_forward" => {
          self.flip_page(-1);
          self.needs_rebuild_areas = true;
          None
        }
        "screensaver_pack.flip_backward" => {
          self.flip_page(1);
          self.needs_rebuild_areas = true;
          None
        }
        "screensaver_pack.search" => Some(ScreensaverPackageCommand::FocusSearch),
        "screensaver_pack.jump" => Some(ScreensaverPackageCommand::FocusJump),
        "screensaver_pack.order" => {
          self.toggle_order();
          None
        }
        "screensaver_pack.sort" => {
          self.next_sort_field();
          None
        }
        "screensaver_pack.confirm"
        | "screensaver_pack.scroll_up"
        | "screensaver_pack.scroll_down"
        | "screensaver_pack.debug"
        | "screensaver_pack.list" => None,
        "screensaver_pack.list.back" => Some(ScreensaverPackageCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn focus_search(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.focus(&mut self.objects, self.search_input);
  }

  pub fn blur_search(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn focus_jump(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.set_text(&mut self.objects, self.jump_input, self.page.to_string());
    self.jump_text = self.page.to_string();
    let _ = text_input.focus(&mut self.objects, self.jump_input);
  }

  pub fn blur_jump(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.set_text(&mut self.objects, self.jump_input, self.page.to_string());
    self.jump_text = self.page.to_string();
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn submit_jump(&mut self, text_input: &mut TextInputService, value: String) {
    if let Ok(page) = value.trim().parse::<usize>() {
      self.page = page.clamp(1, self.total_pages());
      self.selected_index = 0;
      self.needs_rebuild_areas = true;
    }
    let _ = text_input.set_text(&mut self.objects, self.jump_input, self.page.to_string());
    self.jump_text = self.page.to_string();
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn update(&mut self, dt: Duration) -> Option<ScreensaverPackageCommand> {
    let _ = dt;
    None
  }

  /// 渲染屏保包详情页面。
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
    package: &PackageService,
    image: &mut ImageService,
  ) {
    self.sync_entries(package.mod_screensavers());
    let positions = self.compute_positions(layout, i18n);

    self.per_page = positions.visible_items;

    hit_area.render_host(
      &mut self.objects,
      self.page_area,
      layout.developer_viewport_rect(),
      canvas,
    );

    scroll_box.set_rect(
      &mut self.objects,
      self.info_scroll,
      positions.right_inner,
      layout,
    );

    self.draw_right_panel(render, canvas, i18n, &positions);
    self.draw_left_panel(render, canvas, i18n, image, &positions, text_input);
    self.draw_action_hint(render, canvas, i18n, &positions);

    if self.page > 1 {
      hit_area.render_host(
        &mut self.objects,
        self.flip_forward_area,
        positions.flip_forward_rect,
        canvas,
      );
    }
    if self.page < self.total_pages() {
      hit_area.render_host(
        &mut self.objects,
        self.flip_backward_area,
        positions.flip_backward_rect,
        canvas,
      );
    }
    hit_area.render_host(
      &mut self.objects,
      self.order_area,
      positions.order_rect,
      canvas,
    );
    hit_area.render_host(
      &mut self.objects,
      self.sort_area,
      positions.sort_rect,
      canvas,
    );

    if self.needs_rebuild_areas {
      self.rebuild_list_areas(hit_area);
      self.needs_rebuild_areas = false;
    }

    let entries_len = self.page_entries().len();
    for (i, area_id) in self.list_item_areas.iter().enumerate() {
      if i >= entries_len {
        break;
      }
      let item_y = positions
        .list_start_y
        .saturating_add(i as u16 * (positions.list_item_height + 1));
      hit_area.render_host(
        &mut self.objects,
        *area_id,
        Rect {
          x: positions.left_inner.x,
          y: item_y,
          width: positions.left_inner.width,
          height: positions.list_item_height,
        },
        canvas,
      );
    }
  }

  // ─── 布局计算 ──────────────────────────────────────────

  pub fn compute_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> ScreensaverPackageLayout {
    let viewport = layout.developer_viewport_rect();

    let hint_lines = self.action_hint_lines(i18n, viewport.width);
    let hint_h = hint_lines.len().max(1) as u16;
    let content_h = viewport.height.saturating_sub(hint_h);
    let left_w = viewport
      .width
      .saturating_mul(33)
      .saturating_div(100)
      .max(20);
    let right_w = viewport.width.saturating_sub(left_w).max(20);

    let left_rect = Rect {
      x: viewport.x,
      y: viewport.y,
      width: left_w,
      height: content_h,
    };
    let right_rect = Rect {
      x: viewport.x.saturating_add(left_w),
      y: viewport.y,
      width: right_w,
      height: content_h,
    };

    let left_inner = Rect {
      x: left_rect.x.saturating_add(1),
      y: left_rect.y.saturating_add(1),
      width: left_rect.width.saturating_sub(2).max(1),
      height: left_rect.height.saturating_sub(2).max(1),
    };
    let right_inner = Rect {
      x: right_rect.x.saturating_add(1),
      y: right_rect.y.saturating_add(1),
      width: right_rect.width.saturating_sub(2).max(1),
      height: right_rect.height.saturating_sub(2).max(1),
    };

    let search_rect = Rect {
      x: left_inner.x,
      y: left_inner.y,
      width: left_inner.width,
      height: 1,
    };

    let sort_text = self.sort_bar_text(i18n);
    let order_w = self.order_bar_text(i18n).width().min(u16::MAX as usize) as u16 + 2;
    let sort_w = sort_text.width().min(u16::MAX as usize) as u16;
    let sort_bar_y = search_rect.y.saturating_add(1);
    let order_rect = Rect {
      x: left_rect.x.saturating_add(1),
      y: sort_bar_y,
      width: order_w.min(left_rect.width.saturating_sub(2)),
      height: 1,
    };
    let sort_rect = Rect {
      x: order_rect.x.saturating_add(order_rect.width),
      y: sort_bar_y,
      width: sort_w.min(
        left_rect
          .x
          .saturating_add(left_rect.width)
          .saturating_sub(order_rect.x.saturating_add(order_rect.width))
          .saturating_sub(1),
      ),
      height: 1,
    };

    let list_area_y = search_rect.y.saturating_add(3);
    let list_item_height: u16 = 4;
    let page_y = left_inner
      .y
      .saturating_add(left_inner.height)
      .saturating_sub(1);
    let list_area_height = page_y.saturating_sub(list_area_y);
    let visible_items = if list_area_height >= list_item_height {
      ((list_area_height + 1) / (list_item_height + 1)) as usize
    } else {
      0
    };
    let used_height = if visible_items == 0 {
      0
    } else {
      visible_items as u16 * list_item_height + visible_items.saturating_sub(1) as u16
    };
    let list_start_y = list_area_y.saturating_add(list_area_height.saturating_sub(used_height) / 2);

    let page_separator_x = left_inner.x.saturating_add(left_inner.width / 2);
    let jump_width: u16 = 4;
    let jump_rect = Rect {
      x: page_separator_x.saturating_sub(jump_width),
      y: page_y,
      width: jump_width,
      height: 1,
    };
    let total_page_x = page_separator_x.saturating_add(1);
    let flip_forward_text = "❮ Q";
    let flip_backward_text = "E ❯";
    let flip_forward_rect = Rect {
      x: left_inner.x,
      y: page_y,
      width: flip_forward_text.len() as u16,
      height: 1,
    };
    let flip_backward_rect = Rect {
      x: left_inner.x.saturating_add(
        left_inner
          .width
          .saturating_sub(flip_backward_text.len() as u16),
      ),
      y: page_y,
      width: flip_backward_text.len() as u16,
      height: 1,
    };

    let hint_x = viewport.x;
    let hint_y = viewport.y.saturating_add(content_h);

    ScreensaverPackageLayout {
      left_rect,
      left_inner,
      right_rect,
      right_inner,
      search_rect,
      sort_bar_y,
      order_rect,
      sort_rect,
      list_area_y,
      list_area_height,
      list_start_y,
      list_item_height,
      visible_items,
      page_y,
      flip_forward_rect,
      flip_backward_rect,
      jump_rect,
      page_separator_x,
      total_page_x,
      hint_x,
      hint_y,
    }
  }

  // ─── 绘制 ──────────────────────────────────────────────

  fn draw_left_panel(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    image: &mut ImageService,
    pos: &ScreensaverPackageLayout,
    text_input: &TextInputService,
  ) {
    render.draw_host_border_rect(
      canvas,
      pos.left_rect.x,
      pos.left_rect.y,
      pos.left_rect.width,
      pos.left_rect.height,
      &BorderStyle::Double,
      Some(TextColor::Terminal(TerminalColor::BrightBlack)),
      None,
      None,
      None,
    );
    self.draw_panel_title(
      render,
      canvas,
      pos.left_rect,
      &i18n.get_runtime_text("screensaver_pack", "screensaver_pack.list"),
    );

    text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search_rect,
        placeholder: i18n.get_runtime_text(
          "screensaver_pack",
          "screensaver_pack.list.search.placeholder",
        ),
        fg: Some(TextColor::Terminal(TerminalColor::BrightWhite)),
        bg: Some(TextColor::Rgb {
          r: 24,
          g: 28,
          b: 36,
        }),
        placeholder_fg: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
        ..Default::default()
      },
      canvas,
    );
    self.draw_sort_separator(render, canvas, i18n, pos);

    let entries = self.page_entries();
    for (i, entry) in entries.iter().enumerate() {
      let y = pos
        .list_start_y
        .saturating_add(i as u16 * (pos.list_item_height + 1));
      self.draw_entry_card(
        render,
        canvas,
        image,
        i18n,
        pos,
        entry,
        y,
        i == self.selected_index,
      );
    }

    if entries.is_empty() {
      let text = i18n.get_runtime_text("screensaver_pack", "screensaver_pack.no.pack");
      let width = text.width().min(pos.left_inner.width as usize) as u16;
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos
            .left_inner
            .x
            .saturating_add(pos.left_inner.width.saturating_sub(width) / 2),
          y: pos
            .list_area_y
            .saturating_add(pos.list_area_height.saturating_sub(1) / 2),
          text: format!("f%<fg:rgb(85,87,83)>{}</fg>", text),
          max_width: Some(pos.left_inner.width),
          ..Default::default()
        },
      );
    }

    let total = self.total_pages_for(pos.visible_items).max(1);
    if !text_input.is_focused(&self.objects, self.jump_input)
      && self.jump_text != self.page.to_string()
    {
      let _ = text_input.set_text(&mut self.objects, self.jump_input, self.page.to_string());
      self.jump_text = self.page.to_string();
    }
    let key_params = RichTextParams::from_action_map(&Self::action_map(), "screensaver_pack.");
    if self.page > 1 {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.flip_forward_rect.x,
          y: pos.flip_forward_rect.y,
          text: format!(
            "f%<fg:bright_black>{}</fg>",
            i18n.get_runtime_text("screensaver_pack", "screensaver_pack.flip.forward")
          ),
          params: Some(key_params.clone()),
          max_width: Some(pos.flip_forward_rect.width),
          ..Default::default()
        },
      );
    }
    if self.page < total {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.flip_backward_rect.x,
          y: pos.flip_backward_rect.y,
          text: format!(
            "f%<fg:bright_black>{}</fg>",
            i18n.get_runtime_text("screensaver_pack", "screensaver_pack.flip.backward")
          ),
          params: Some(key_params),
          max_width: Some(pos.flip_backward_rect.width),
          ..Default::default()
        },
      );
    }

    let jump_focused = text_input.is_focused(&self.objects, self.jump_input);
    text_input.render_host(
      &mut self.objects,
      self.jump_input,
      &TextInputRenderParams {
        rect: pos.jump_rect,
        fg: Some(if jump_focused {
          TextColor::Terminal(TerminalColor::Black)
        } else {
          TextColor::Terminal(TerminalColor::BrightWhite)
        }),
        bg: if jump_focused {
          Some(TextColor::Terminal(TerminalColor::Yellow))
        } else {
          None
        },
        placeholder: String::new(),
        cursor_shape: Some(TextInputCursorShape::None),
        text_align: TextAlign::Right,
        ..Default::default()
      },
      canvas,
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.page_separator_x,
        y: pos.page_y,
        text: "f%<fg:bright_black>|</fg>".to_string(),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.total_page_x,
        y: pos.page_y,
        text: format!("f%<fg:bright_black>{}</fg>", total),
        ..Default::default()
      },
    );
  }

  fn draw_entry_card(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    image: &mut ImageService,
    i18n: &I18nService,
    pos: &ScreensaverPackageLayout,
    entry: &PackageListEntry,
    y: u16,
    focused: bool,
  ) {
    let marker_x = pos.left_inner.x;
    let image_x = marker_x.saturating_add(1);
    let text_x = image_x.saturating_add(9);
    let text_width = pos
      .left_inner
      .x
      .saturating_add(pos.left_inner.width)
      .saturating_sub(text_x)
      .max(1);

    for row in 0..4 {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: marker_x,
          y: y.saturating_add(row),
          text: if focused {
            "f%<fg:bright_cyan>▌</fg>".to_string()
          } else {
            " ".to_string()
          },
          ..Default::default()
        },
      );
    }

    render.draw_host_filled_rect(
      canvas,
      image_x,
      y,
      8,
      4,
      Some(" ".to_string()),
      None,
      Some(TextColor::Rgb {
        r: 85,
        g: 87,
        b: 83,
      }),
    );
    if let Some(icon) = &entry.icon_path {
      if let Ok(text) = image.convert(ImageConvertParams {
        image_path: icon.clone(),
        output_width: 8,
        output_height: 4,
        square_crop: true,
        scale: 1.0,
        cache: true,
        ..Default::default()
      }) {
        render.draw_host_text(
          canvas,
          &DrawTextParams {
            x: image_x,
            y,
            text,
            wrap_mode: TextWrapMode::None,
            max_width: Some(8),
            max_height: Some(4),
            ..Default::default()
          },
        );
      }
    }

    let status_key = if entry.enabled {
      "screensaver_pack.list.status.on"
    } else {
      "screensaver_pack.list.status.off"
    };
    let lines = [
      if entry.debug {
        format!(
          "f%[<fg:bright_magenta>{}</fg>]{}",
          i18n.get_runtime_text("screensaver_pack", "screensaver_pack.list.debug"),
          entry.title
        )
      } else {
        entry.title.clone()
      },
      format!(
        "f%<fg:bright_yellow>{}</fg>{}",
        i18n.get_runtime_text("screensaver_pack", "screensaver_pack.info.author"),
        entry.author
      ),
      format!(
        "f%<fg:bright_yellow>{}</fg>{}",
        i18n.get_runtime_text("screensaver_pack", "screensaver_pack.list.version"),
        entry.version
      ),
      format!(
        "f%<fg:bright_yellow>{}</fg>{}",
        i18n.get_runtime_text("screensaver_pack", "screensaver_pack.list.status"),
        i18n.get_runtime_text("screensaver_pack", status_key)
      ),
    ];
    for (row, text) in lines.into_iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: text_x,
          y: y.saturating_add(row as u16),
          text,
          wrap_mode: TextWrapMode::None,
          max_width: Some(text_width),
          overflow_marker: Some("...".to_string()),
          ..Default::default()
        },
      );
    }
  }

  fn draw_right_panel(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &ScreensaverPackageLayout,
  ) {
    render.draw_host_border_rect(
      canvas,
      pos.right_rect.x,
      pos.right_rect.y,
      pos.right_rect.width,
      pos.right_rect.height,
      &BorderStyle::Double,
      Some(TextColor::Terminal(TerminalColor::BrightBlack)),
      None,
      None,
      None,
    );
    self.draw_panel_title(
      render,
      canvas,
      pos.right_rect,
      &i18n.get_runtime_text("screensaver_pack", "screensaver_pack.info"),
    );

    if self.page_entries().get(self.selected_index).is_some() {
      return;
    }

    let text = i18n.get_runtime_text("screensaver_pack", "screensaver_pack.no.info");
    let width = text.width().min(pos.right_inner.width as usize) as u16;
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos
          .right_inner
          .x
          .saturating_add(pos.right_inner.width.saturating_sub(width) / 2),
        y: pos
          .right_inner
          .y
          .saturating_add(pos.right_inner.height.saturating_sub(1) / 2),
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", text),
        max_width: Some(pos.right_inner.width),
        ..Default::default()
      },
    );
  }

  fn draw_panel_title(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    rect: Rect,
    title: &str,
  ) {
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x.saturating_add(1),
        y: rect.y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        max_width: Some(rect.width.saturating_sub(2)),
        ..Default::default()
      },
    );
  }

  fn draw_sort_separator(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &ScreensaverPackageLayout,
  ) {
    if pos.left_rect.width < 2 {
      return;
    }
    let order = self.order_bar_text(i18n);
    let sort = self.sort_bar_text(i18n);
    let label = format!("[{}]{}", order, sort);
    let label_w = label
      .width()
      .min(pos.left_rect.width.saturating_sub(2) as usize);
    let line_w = pos.left_rect.width.saturating_sub(2 + label_w as u16) as usize;
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.left_rect.x,
        y: pos.sort_bar_y,
        text: format!(
          "f%<fg:bright_black>╟[</fg><fg:bright_yellow>{}</fg><fg:bright_black>]</fg><fg:bright_green>{}</fg><fg:bright_black>{}╢</fg>",
          order,
          sort,
          "─".repeat(line_w)
        ),
        max_width: Some(pos.left_rect.width),
        ..Default::default()
      },
    );
  }

  fn draw_action_hint(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &ScreensaverPackageLayout,
  ) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "screensaver_pack.");
    for (index, line) in self
      .action_hint_lines(i18n, pos.left_rect.width + pos.right_rect.width)
      .iter()
      .enumerate()
    {
      let width = UnicodeWidthStr::width(
        RichTextService::new()
          .visible_text(line, Some(&params))
          .as_str(),
      ) as u16;
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos
            .hint_x
            .saturating_add((pos.left_rect.width + pos.right_rect.width).saturating_sub(width) / 2),
          y: pos.hint_y.saturating_add(index as u16),
          text: format!("f%<fg:rgb(85,87,83)>{}</fg>", line),
          params: Some(params.clone()),
          max_width: Some(pos.left_rect.width + pos.right_rect.width),
          ..Default::default()
        },
      );
    }
  }

  // ─── 辅助方法 ──────────────────────────────────────────

  fn focus_previous(&mut self) {
    let page_len = self.page_entries().len();
    if page_len == 0 {
      return;
    }
    if self.selected_index == 0 {
      self.selected_index = page_len.saturating_sub(1);
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    let page_len = self.page_entries().len();
    if page_len == 0 {
      return;
    }
    if self.selected_index >= page_len.saturating_sub(1) {
      self.selected_index = 0;
    } else {
      self.selected_index += 1;
    }
  }

  fn handle_hover(&mut self, id: HitAreaId) {
    if let Some(pos) = self.list_item_areas.iter().position(|a| *a == id) {
      self.selected_index = pos;
    }
  }

  fn flip_page(&mut self, delta: i32) {
    let total = self.total_pages();
    if delta > 0 && self.page < total {
      self.page += 1;
    } else if delta < 0 && self.page > 1 {
      self.page -= 1;
    }
    self.selected_index = 0;
  }

  fn total_pages(&self) -> usize {
    self.total_pages_for(self.per_page)
  }

  fn total_pages_for(&self, per_page: usize) -> usize {
    if per_page == 0 {
      return 1;
    }
    self.filtered_entries().len().div_ceil(per_page).max(1)
  }

  fn page_entries(&self) -> Vec<PackageListEntry> {
    if self.per_page == 0 {
      return vec![];
    }
    let start = (self.page.saturating_sub(1)).saturating_mul(self.per_page);
    self
      .filtered_entries()
      .into_iter()
      .skip(start)
      .take(self.per_page)
      .collect()
  }

  fn filtered_entries(&self) -> Vec<PackageListEntry> {
    let query = self.search_text.trim().to_lowercase();
    let mut entries = self
      .entries
      .iter()
      .filter(|entry| query.is_empty() || entry.title.to_lowercase().contains(&query))
      .cloned()
      .collect::<Vec<_>>();
    entries.sort_by(|a, b| self.compare_entries(a, b));
    if !self.ascending {
      entries.reverse();
    }
    entries
  }

  fn compare_entries(&self, a: &PackageListEntry, b: &PackageListEntry) -> Ordering {
    self
      .sort_value(a)
      .cmp(&self.sort_value(b))
      .then(a.title.width().cmp(&b.title.width()))
      .then(a.title.cmp(&b.title))
      .then(a.mod_id.width().cmp(&b.mod_id.width()))
      .then(a.mod_id.cmp(&b.mod_id))
  }

  fn sort_value(&self, entry: &PackageListEntry) -> String {
    match self.sort_field {
      ScreensaverSortField::Title => entry.title.clone(),
      ScreensaverSortField::Author => entry.author.clone(),
      ScreensaverSortField::Status => format!("{}", entry.enabled),
      ScreensaverSortField::Debug => format!("{}", entry.debug),
    }
  }

  fn sync_entries(&mut self, entries: Vec<PackageListEntry>) {
    if self
      .entries
      .iter()
      .map(|entry| &entry.mod_id)
      .eq(entries.iter().map(|entry| &entry.mod_id))
    {
      self.entries = entries;
      return;
    }
    self.entries = entries;
    self.page = 1;
    self.selected_index = 0;
    self.needs_rebuild_areas = true;
  }

  fn toggle_order(&mut self) {
    self.ascending = !self.ascending;
    self.page = 1;
    self.selected_index = 0;
    self.needs_rebuild_areas = true;
  }

  fn next_sort_field(&mut self) {
    self.sort_field = self.sort_field.next();
    self.page = 1;
    self.selected_index = 0;
    self.needs_rebuild_areas = true;
  }

  fn order_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text(
      "screensaver_pack",
      if self.ascending {
        "screensaver_pack.list.order.ascending"
      } else {
        "screensaver_pack.list.order.descending"
      },
    )
  }

  fn sort_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text("screensaver_pack", self.sort_field.key())
  }

  fn rebuild_list_areas(&mut self, hit_area: &HitAreaService) {
    for area_id in self.list_item_areas.drain(..) {
      hit_area.remove(&mut self.objects, area_id);
    }
    let count = self.page_entries().len();
    for _ in 0..count {
      let id = hit_area.create(&mut self.objects, HitAreaOptions::default());
      self.list_item_areas.push(id);
    }
  }

  fn action_hint_lines(&self, i18n: &I18nService, max_width: u16) -> Vec<String> {
    let params = RichTextParams::from_action_map(&Self::action_map(), "screensaver_pack.");
    let rich = RichTextService::new();
    let items = [
      "screensaver_pack.action.select",
      "screensaver_pack.action.flip",
      "screensaver_pack.action.scroll",
      "screensaver_pack.action.confirm",
      "screensaver_pack.action.list.back",
      "screensaver_pack.action.debug.off",
      "screensaver_pack.action.list.detail2simple",
      "screensaver_pack.action.list.search",
      "screensaver_pack.action.list.order",
      "screensaver_pack.action.list.sort",
      "screensaver_pack.action.list.jump",
    ]
    .map(|key| i18n.get_runtime_text("screensaver_pack", key));

    let mut lines = vec![String::new()];
    let mut widths = vec![0usize];
    let limit = max_width as usize;
    for item in items {
      let item_w = UnicodeWidthStr::width(rich.visible_text(&item, Some(&params)).as_str());
      let gap = if lines.last().is_some_and(|line| line.is_empty()) {
        0
      } else {
        2
      };
      if lines.len() == 1 && widths[0] > 0 && widths[0] + gap + item_w > limit {
        lines.push(String::new());
        widths.push(0);
      }
      let last = lines.len() - 1;
      if !lines[last].is_empty() {
        lines[last].push_str("  ");
        widths[last] += 2;
      }
      lines[last].push_str(&item);
      widths[last] += item_w;
    }
    lines
  }
}
