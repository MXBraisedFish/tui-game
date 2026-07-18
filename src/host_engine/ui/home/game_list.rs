use std::{cmp::Ordering, collections::HashSet, time::Duration};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DisplaySourceMode, DrawTextParams, HitAreaEvent,
  HitAreaId, HitAreaOptions, HitAreaService, I18nService, ImageService, KeyState, LayoutService,
  LogService, MouseButton, Overflow, PackageListEntry, PackageService, PackageSource, Rect,
  RenderService, RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner,
  ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarPolicy, ScrollbarVisibility,
  StorageService, TerminalColor, TextAlign, TextColor, TextInputCursorShape, TextInputEvent,
  TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams, TextInputService, TextStyle,
  UiEvent, UiObjectPool, UiObjectPoolOwner,
};

/// 游戏列表页面的命令。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameListCommand {
  Back,
  FocusSearch,
  BlurSearch,
  FocusJump,
  BlurJump,
  ScrollInfoUp,
  ScrollInfoDown,
  SubmitJump(String),
  Confirm,
}

/// 游戏列表页面布局信息。
pub(crate) struct GameListLayout {
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
  pub list_item_gap: u16,
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
enum GameListSortField {
  Title,
  Author,
  Source,
}

impl GameListSortField {
  fn next(self) -> Self {
    match self {
      Self::Title => Self::Author,
      Self::Author => Self::Source,
      Self::Source => Self::Title,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Title => "game_list.list.sort.title",
      Self::Author => "game_list.list.sort.author",
      Self::Source => "game_list.list.sort.source",
    }
  }
}

/// 游戏列表 UI：左右 33/67 分栏布局。
///
/// 左侧：搜索框 + 列表（翻页） + 翻页指示器，包裹在双线边框内。
/// 右侧：滚动信息盒，包裹在双线边框内。
/// 底部：操作提示栏。
pub struct GameListUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
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
  sort_field: GameListSortField,
  entries: Vec<PackageListEntry>,
  search_text: String,
  jump_text: String,
  simple_list: bool,
  source_mode: DisplaySourceMode,
  show_warnings: bool,
  temporary_safe_mode_disabled: HashSet<String>,
  needs_rebuild_areas: bool,
}

impl UiObjectPoolOwner for GameListUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for GameListUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl GameListUi {
  /// 初始化游戏列表 UI。
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
          content_width: 160,
          content_height: 120,
          overflow_y: Overflow::Auto,
          overflow_x: Overflow::Hidden,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .expect("failed to create game info scroll box");

    let page_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_forward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_backward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let order_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let sort_area = hit_area.create(&mut objects, HitAreaOptions::default());

    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
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
      sort_field: GameListSortField::Title,
      entries: Vec::new(),
      search_text: String::new(),
      jump_text: "1".to_string(),
      simple_list: false,
      source_mode: DisplaySourceMode::All,
      show_warnings: true,
      temporary_safe_mode_disabled: HashSet::new(),
      needs_rebuild_areas: true,
    }
  }

  /// 返回按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "game_list.flip_forward".to_string(),
        description: "Previous list page".to_string(),
        keys: vec![vec!["q".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.flip_backward".to_string(),
        description: "Next list page".to_string(),
        keys: vec![vec!["e".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.scroll_up".to_string(),
        description: "Scroll info up".to_string(),
        keys: vec![vec!["w".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.scroll_down".to_string(),
        description: "Scroll info down".to_string(),
        keys: vec![vec!["s".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.focus_up".to_string(),
        description: "Focus previous item".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.focus_down".to_string(),
        description: "Focus next item".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.confirm".to_string(),
        description: "Toggle selection".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.list.back".to_string(),
        description: "Go back to mods menu".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.list".to_string(),
        description: "Toggle list style".to_string(),
        keys: vec![vec!["l".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.search".to_string(),
        description: "Search".to_string(),
        keys: vec![vec!["c".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.order".to_string(),
        description: "Toggle order".to_string(),
        keys: vec![vec!["z".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.sort".to_string(),
        description: "Toggle sort".to_string(),
        keys: vec![vec!["x".to_string()]],
      },
      ActionMapEntry {
        action: "game_list.jump".to_string(),
        description: "Jump to page".to_string(),
        keys: vec![vec!["j".to_string()]],
      },
    ]
  }

  /// 处理 UI 事件，返回导航命令。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<GameListCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.handle_hover(*id);
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        Some(GameListCommand::FocusSearch)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.search_input => {
        Some(GameListCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        Some(GameListCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search_text = value.clone();
        self.page = 1;
        self.selected_index = 0;
        self.needs_rebuild_areas = true;
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.jump_input => {
        Some(GameListCommand::FocusJump)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.jump_input => {
        Some(GameListCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.jump_input => {
        Some(GameListCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.jump_input => {
        Some(GameListCommand::SubmitJump(value.clone()))
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
      }) => Some(GameListCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "game_list.focus_up" => {
          self.focus_previous();
          None
        }
        "game_list.focus_down" => {
          self.focus_next();
          None
        }
        "game_list.flip_forward" => {
          self.flip_page(-1);
          self.needs_rebuild_areas = true;
          None
        }
        "game_list.flip_backward" => {
          self.flip_page(1);
          self.needs_rebuild_areas = true;
          None
        }
        "game_list.search" => Some(GameListCommand::FocusSearch),
        "game_list.jump" => Some(GameListCommand::FocusJump),
        "game_list.order" => {
          self.toggle_order();
          None
        }
        "game_list.sort" => {
          self.next_sort_field();
          None
        }
        "game_list.scroll_up" => Some(GameListCommand::ScrollInfoUp),
        "game_list.scroll_down" => Some(GameListCommand::ScrollInfoDown),
        "game_list.list" => {
          self.toggle_list_style();
          None
        }
        "game_list.confirm" => Some(GameListCommand::Confirm),
        "game_list.list.back" => Some(GameListCommand::Back),
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

  pub fn scroll_info(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, lines: i32) {
    let _ = scroll_box.scroll_by(&mut self.objects, self.info_scroll, 0, lines, layout);
  }

  pub fn update(&mut self, dt: Duration) -> Option<GameListCommand> {
    let _ = dt;
    None
  }

  /// 渲染游戏列表页面。
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
    storage: &StorageService,
    log: &mut LogService,
    temporary_safe_mode_disabled: &HashSet<String>,
    image: &mut ImageService,
    mouse_supported: bool,
    truecolor_supported: bool,
  ) {
    self.sync_display_settings(storage);
    self.sync_entries(
      package.game_list(),
      storage,
      log,
      temporary_safe_mode_disabled,
    );
    let positions = self.compute_positions(layout, i18n, text_input);

    self.sync_selection_for_per_page(positions.visible_items);

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
    let info_content_height = self.info_content_height(
      layout,
      i18n,
      positions.right_inner.width,
      mouse_supported,
      truecolor_supported,
    );
    scroll_box.set_content_size(
      &mut self.objects,
      self.info_scroll,
      positions.right_inner.width.max(1),
      info_content_height,
      layout,
    );
    canvas.prepare(&self.objects, layout);

    self.draw_right_panel(
      render,
      canvas,
      layout,
      i18n,
      image,
      mouse_supported,
      truecolor_supported,
      &positions,
    );
    self.draw_left_panel(render, canvas, layout, i18n, image, &positions, text_input);
    self.draw_action_hint(render, canvas, i18n, text_input, &positions);

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

    let entries_len = self.page_entries().len();
    if self.needs_rebuild_areas || self.list_item_areas.len() != entries_len {
      self.rebuild_list_areas(hit_area);
      self.needs_rebuild_areas = false;
    }

    for (i, area_id) in self.list_item_areas.iter().enumerate() {
      if i >= entries_len {
        break;
      }
      let item_y = positions
        .list_start_y
        .saturating_add(i as u16 * (positions.list_item_height + positions.list_item_gap));
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
    text_input: &TextInputService,
  ) -> GameListLayout {
    let viewport = layout.developer_viewport_rect();

    let hint_lines = self.action_hint_lines(i18n, text_input, viewport.width);
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
    let order_w = layout
      .get_text_width(&self.order_bar_text(i18n), None)
      .saturating_add(2);
    let sort_w = layout.get_text_width(&sort_text, None);
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

    let list_area_y = sort_bar_y.saturating_add(1);
    let list_item_height: u16 = 1;
    let list_item_gap: u16 = 0;
    let page_y = left_inner
      .y
      .saturating_add(left_inner.height)
      .saturating_sub(1);
    let list_area_height = page_y.saturating_sub(list_area_y);
    let visible_items = if list_area_height >= list_item_height {
      ((list_area_height + list_item_gap) / (list_item_height + list_item_gap)) as usize
    } else {
      0
    };
    let list_start_y = list_area_y;

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

    GameListLayout {
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
      list_item_gap,
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
    layout: &LayoutService,
    i18n: &I18nService,
    _image: &mut ImageService,
    pos: &GameListLayout,
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
      &i18n.get_runtime_text("game_list", "game_list.list"),
    );

    text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search_rect,
        placeholder: i18n.get_runtime_text("game_list", "game_list.list.search.placeholder"),
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
        .saturating_add(i as u16 * (pos.list_item_height + pos.list_item_gap));
      self.draw_entry_row(
        render,
        canvas,
        layout,
        i18n,
        pos,
        entry,
        y,
        i == self.selected_index,
      );
    }

    if entries.is_empty() {
      let text = i18n.get_runtime_text("game_list", "game_list.no.pack");
      let width = layout.get_text_width(&text, None).min(pos.left_inner.width);
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
    let key_params = RichTextParams::from_action_map(&Self::action_map(), "game_list.");
    if self.page > 1 {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.flip_forward_rect.x,
          y: pos.flip_forward_rect.y,
          text: format!(
            "f%<fg:bright_black>{}</fg>",
            i18n.get_runtime_text("game_list", "game_list.flip.forward")
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
            i18n.get_runtime_text("game_list", "game_list.flip.backward")
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

  fn draw_entry_row(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameListLayout,
    entry: &PackageListEntry,
    y: u16,
    focused: bool,
  ) {
    let package_params = Self::package_rich_params(entry);
    let show_source = self.shows_source_label(&entry.source);
    let source_key = match entry.source {
      PackageSource::Official => "game_list.list.source.official",
      PackageSource::Mod => "game_list.list.source.mod",
    };
    let source = i18n.get_runtime_text("game_list", source_key);
    let source_color = match entry.source {
      PackageSource::Official => "bright_magenta",
      PackageSource::Mod => "bright_yellow",
    };
    let source_text = if show_source {
      format!("f%[<fg:{}>{}</fg>]", source_color, source)
    } else {
      String::new()
    };
    let source_width = layout
      .get_text_width(&source_text, None)
      .min(pos.left_inner.width);
    let source_x = pos
      .left_inner
      .x
      .saturating_add(pos.left_inner.width.saturating_sub(source_width));
    let marker_x = pos.left_inner.x;
    let title_x = marker_x.saturating_add(1);
    let title_width = source_x.saturating_sub(title_x).max(1);

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: marker_x,
        y,
        text: if focused {
          "f%<fg:bright_cyan>▌</fg>".to_string()
        } else {
          " ".to_string()
        },
        wrap_mode: TextWrapMode::None,
        max_width: Some(1),
        ..Default::default()
      },
    );

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y,
        text: Self::game_display_name(entry).to_string(),
        params: Some(package_params),
        wrap_mode: TextWrapMode::None,
        max_width: Some(title_width),
        overflow_marker: Some("...".to_string()),
        ..Default::default()
      },
    );
    if show_source {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: source_x,
          y,
          text: source_text,
          wrap_mode: TextWrapMode::None,
          max_width: Some(source_width),
          ..Default::default()
        },
      );
    }
  }

  fn draw_right_panel(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    _image: &mut ImageService,
    mouse_supported: bool,
    truecolor_supported: bool,
    pos: &GameListLayout,
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
      &i18n.get_runtime_text("game_list", "game_list.info"),
    );

    let Some(entry) = self.selected_entry() else {
      return;
    };

    let width = pos.right_inner.width.max(1);
    let params = Self::package_rich_params(&entry);
    let mut y = 0;

    self.draw_info_value(
      render,
      canvas,
      width,
      &mut y,
      Self::game_display_name(&entry),
      &params,
    );
    self.draw_info_separator(canvas, width, &mut y);
    self.draw_info_pair(
      render,
      canvas,
      layout,
      width,
      &mut y,
      &i18n.get_runtime_text("game_list", "game_list.info.author"),
      &entry.author,
      &params,
    );
    self.draw_info_pair(
      render,
      canvas,
      layout,
      width,
      &mut y,
      &i18n.get_runtime_text("game_list", "game_list.info.version"),
      &entry.version,
      &params,
    );
    self.draw_info_pair(
      render,
      canvas,
      layout,
      width,
      &mut y,
      &i18n.get_runtime_text("game_list", "game_list.info.pack_title"),
      &entry.title,
      &params,
    );
    self.draw_info_separator(canvas, width, &mut y);

    let warnings = self.info_warnings(i18n, &entry, mouse_supported, truecolor_supported);
    if !warnings.is_empty() {
      for warning in warnings {
        self.draw_info_warning(render, canvas, width, &mut y, warning);
      }
      self.draw_info_separator(canvas, width, &mut y);
    }

    if entry.score_enabled {
      let score = self.info_score_text(i18n, &entry);
      self.draw_info_wrapped(
        render,
        canvas,
        layout,
        width,
        &mut y,
        score,
        Some(&params),
        None,
      );
      self.draw_info_separator(canvas, width, &mut y);
    }

    self.draw_info_description_label(canvas, &mut y, i18n);
    self.draw_info_wrapped(
      render,
      canvas,
      layout,
      width,
      &mut y,
      entry.game_detail.clone(),
      Some(&params),
      None,
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
    pos: &GameListLayout,
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
    text_input: &TextInputService,
    pos: &GameListLayout,
  ) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "game_list.");
    for (index, line) in self
      .action_hint_lines(i18n, text_input, pos.left_rect.width + pos.right_rect.width)
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

  fn package_rich_params(entry: &PackageListEntry) -> RichTextParams {
    RichTextParams::from_key_actions(&entry.key_actions)
  }

  fn package_visible_text(entry: &PackageListEntry, text: &str) -> String {
    RichTextService::new().visible_text(text, Some(&Self::package_rich_params(entry)))
  }

  fn selected_entry(&self) -> Option<PackageListEntry> {
    self.page_entries().get(self.selected_index).cloned()
  }

  fn game_display_name(entry: &PackageListEntry) -> &str {
    if entry.game_name.trim().is_empty() {
      &entry.title
    } else {
      &entry.game_name
    }
  }

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

  fn sync_selection_for_per_page(&mut self, per_page: usize) {
    let selected = self.selected_global_index();
    self.per_page = per_page;
    self.apply_global_selection(selected);
  }

  fn selected_global_index(&self) -> usize {
    if self.per_page == 0 {
      return 0;
    }
    (self.page.saturating_sub(1))
      .saturating_mul(self.per_page)
      .saturating_add(self.selected_index)
  }

  fn apply_global_selection(&mut self, index: usize) {
    let len = self.filtered_entries().len();
    if self.per_page == 0 || len == 0 {
      self.page = 1;
      self.selected_index = 0;
      return;
    }
    let index = index.min(len - 1);
    self.page = index / self.per_page + 1;
    self.selected_index = index % self.per_page;
  }

  fn selected_entry_key(&self) -> Option<(PackageSource, String)> {
    self
      .page_entries()
      .get(self.selected_index)
      .map(|entry| (entry.source.clone(), entry.mod_id.clone()))
  }

  fn restore_selection(&mut self, key: Option<(PackageSource, String)>) {
    let Some((source, mod_id)) = key else {
      self.apply_global_selection(0);
      return;
    };
    let entries = self.filtered_entries();
    let Some(index) = entries
      .iter()
      .position(|entry| entry.source == source && entry.mod_id == mod_id)
    else {
      self.apply_global_selection(0);
      return;
    };
    self.apply_global_selection(index);
  }

  fn info_content_height(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    width: u16,
    mouse_supported: bool,
    truecolor_supported: bool,
  ) -> u16 {
    let Some(entry) = self.selected_entry() else {
      return 1;
    };

    let width = width.max(1);
    let params = Self::package_rich_params(&entry);
    let mut height = 6;
    let warnings = self.info_warnings(i18n, &entry, mouse_supported, truecolor_supported);
    if !warnings.is_empty() {
      height += warnings.len() as u16 + 1;
    }
    if entry.score_enabled {
      height += self.measure_info_text(
        layout,
        width,
        self.info_score_text(i18n, &entry),
        TextWrapMode::Auto,
        Some(&params),
      ) + 1;
    }
    height += 1;
    height += self.measure_info_text(
      layout,
      width,
      entry.game_detail.clone(),
      TextWrapMode::Auto,
      Some(&params),
    );
    height.max(1)
  }

  fn measure_info_text(
    &self,
    layout: &LayoutService,
    width: u16,
    text: String,
    wrap_mode: TextWrapMode,
    params: Option<&RichTextParams>,
  ) -> u16 {
    layout
      .get_draw_text_size(&DrawTextParams {
        text,
        params: params.cloned(),
        wrap_mode,
        max_width: Some(width),
        ..Default::default()
      })
      .height
      .max(1)
  }

  fn info_warnings(
    &self,
    i18n: &I18nService,
    entry: &PackageListEntry,
    mouse_supported: bool,
    truecolor_supported: bool,
  ) -> Vec<String> {
    if !self.show_warnings {
      return Vec::new();
    }
    let mut warnings = Vec::new();
    if entry.mouse_required && !mouse_supported {
      warnings.push(i18n.get_runtime_text("game_list", "game_list.info.mouse.error"));
    }
    if entry.truecolor_required && !truecolor_supported {
      warnings.push(i18n.get_runtime_text("game_list", "game_list.info.true_color.error"));
    }
    if entry.high_privilege_required && entry.safe_mode {
      warnings.push(i18n.get_runtime_text("game_list", "game_list.info.high_privilege.error"));
    }
    warnings
  }

  fn info_score_text(&self, i18n: &I18nService, entry: &PackageListEntry) -> String {
    if entry.score_empty_text.trim().is_empty() {
      i18n.get_runtime_text("game_list", "game_list.info.high_score.no")
    } else {
      entry.score_empty_text.clone()
    }
  }

  fn draw_info_separator(&self, canvas: &mut CanvasService, width: u16, y: &mut u16) {
    canvas.styled_text_in_scroll_box(
      self.info_scroll,
      0,
      *y,
      &"─".repeat(width as usize),
      TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
        ..Default::default()
      },
    );
    *y = (*y).saturating_add(1);
  }

  fn draw_info_value(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    width: u16,
    y: &mut u16,
    value: &str,
    params: &RichTextParams,
  ) {
    render.draw_text_in_scroll_box(
      canvas,
      self.info_scroll,
      &DrawTextParams {
        y: *y,
        text: value.to_string(),
        params: Some(params.clone()),
        wrap_mode: TextWrapMode::None,
        max_width: Some(width),
        overflow_marker: Some("...".to_string()),
        ..Default::default()
      },
    );
    *y = (*y).saturating_add(1);
  }

  fn draw_info_pair(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    width: u16,
    y: &mut u16,
    label: &str,
    value: &str,
    params: &RichTextParams,
  ) {
    let label_text = format!("f%<fg:bright_blue>{}</fg>", label);
    let label_width = layout.get_text_width(&label_text, None).min(width);
    render.draw_text_in_scroll_box(
      canvas,
      self.info_scroll,
      &DrawTextParams {
        y: *y,
        text: label_text,
        wrap_mode: TextWrapMode::None,
        max_width: Some(label_width),
        ..Default::default()
      },
    );
    if label_width < width {
      render.draw_text_in_scroll_box(
        canvas,
        self.info_scroll,
        &DrawTextParams {
          x: label_width,
          y: *y,
          text: value.to_string(),
          params: Some(params.clone()),
          wrap_mode: TextWrapMode::None,
          max_width: Some(width - label_width),
          overflow_marker: Some("...".to_string()),
          ..Default::default()
        },
      );
    }
    *y = (*y).saturating_add(1);
  }

  fn draw_info_warning(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    width: u16,
    y: &mut u16,
    text: String,
  ) {
    render.draw_text_in_scroll_box(
      canvas,
      self.info_scroll,
      &DrawTextParams {
        y: *y,
        text,
        fg: Some(TextColor::Terminal(TerminalColor::BrightRed)),
        bold: true,
        wrap_mode: TextWrapMode::None,
        max_width: Some(width),
        overflow_marker: Some("...".to_string()),
        ..Default::default()
      },
    );
    *y = (*y).saturating_add(1);
  }

  fn draw_info_description_label(
    &self,
    canvas: &mut CanvasService,
    y: &mut u16,
    i18n: &I18nService,
  ) {
    let label = i18n.get_runtime_text("game_list", "game_list.info.description");
    canvas.styled_text_in_scroll_box(
      self.info_scroll,
      0,
      *y,
      &label,
      TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 255,
          g: 190,
          b: 120,
        }),
        bold: true,
        ..Default::default()
      },
    );
    *y = (*y).saturating_add(1);
  }

  fn draw_info_wrapped(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    width: u16,
    y: &mut u16,
    text: String,
    params: Option<&RichTextParams>,
    fg: Option<TextColor>,
  ) {
    let draw_params = DrawTextParams {
      y: *y,
      text,
      params: params.cloned(),
      fg,
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(width),
      ..Default::default()
    };
    render.draw_text_in_scroll_box(canvas, self.info_scroll, &draw_params);
    *y = (*y).saturating_add(layout.get_draw_text_size(&draw_params).height.max(1));
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
      .filter(|entry| {
        query.is_empty()
          || Self::package_visible_text(entry, &entry.title)
            .to_lowercase()
            .contains(&query)
          || Self::package_visible_text(entry, Self::game_display_name(entry))
            .to_lowercase()
            .contains(&query)
      })
      .cloned()
      .collect::<Vec<_>>();
    entries.sort_by(|a, b| self.compare_entries(a, b));
    if !self.ascending {
      entries.reverse();
    }
    entries
  }

  fn sync_display_settings(&mut self, storage: &StorageService) {
    let profile = storage.display_settings_profile();
    self.show_warnings = profile.game_list_warnings;
    self.source_mode = profile.game_list_source;
  }

  fn shows_source_label(&self, source: &PackageSource) -> bool {
    match self.source_mode {
      DisplaySourceMode::All => true,
      DisplaySourceMode::Mod => source == &PackageSource::Mod,
      DisplaySourceMode::Official => source == &PackageSource::Official,
      DisplaySourceMode::No => false,
    }
  }

  fn compare_entries(&self, a: &PackageListEntry, b: &PackageListEntry) -> Ordering {
    self
      .sort_value(a)
      .cmp(&self.sort_value(b))
      .then(
        Self::package_visible_text(a, Self::game_display_name(a))
          .width()
          .cmp(&Self::package_visible_text(b, Self::game_display_name(b)).width()),
      )
      .then(
        Self::package_visible_text(a, Self::game_display_name(a))
          .cmp(&Self::package_visible_text(b, Self::game_display_name(b))),
      )
      .then(a.mod_id.width().cmp(&b.mod_id.width()))
      .then(a.mod_id.cmp(&b.mod_id))
  }

  fn sort_value(&self, entry: &PackageListEntry) -> String {
    match self.sort_field {
      GameListSortField::Title => Self::package_visible_text(entry, Self::game_display_name(entry)),
      GameListSortField::Author => Self::package_visible_text(entry, &entry.author),
      GameListSortField::Source => format!("{:?}", entry.source),
    }
  }

  fn sync_entries(
    &mut self,
    mut entries: Vec<PackageListEntry>,
    storage: &StorageService,
    log: &mut LogService,
    temporary_safe_mode_disabled: &HashSet<String>,
  ) {
    self.temporary_safe_mode_disabled = temporary_safe_mode_disabled.clone();
    let profile = storage.read_package_state_or_default(log);
    for entry in &mut entries {
      if let Some(state) = profile.games.get(&entry.mod_id) {
        entry.enabled = state.enabled;
        entry.debug = state.debug;
        entry.safe_mode = state.safe_mode;
      } else {
        entry.enabled = profile.defaults.enabled;
        entry.debug = profile.defaults.debug;
        entry.safe_mode = matches!(
          profile.defaults.safe_mode,
          crate::host_engine::services::SafeModeDefault::On
        );
      }
      if temporary_safe_mode_disabled.contains(&entry.mod_id) {
        entry.safe_mode = false;
      }
    }
    let selected = self.selected_entry_key();
    if self
      .entries
      .iter()
      .map(|entry| &entry.mod_id)
      .eq(entries.iter().map(|entry| &entry.mod_id))
    {
      self.entries = entries;
      self.restore_selection(selected);
      return;
    }
    self.entries = entries;
    self.restore_selection(selected);
    self.needs_rebuild_areas = true;
  }

  fn toggle_order(&mut self) {
    let selected = self.selected_entry_key();
    self.ascending = !self.ascending;
    self.restore_selection(selected);
    self.needs_rebuild_areas = true;
  }

  fn next_sort_field(&mut self) {
    let selected = self.selected_entry_key();
    self.sort_field = self.sort_field.next();
    self.restore_selection(selected);
    self.needs_rebuild_areas = true;
  }

  fn toggle_list_style(&mut self) {
    self.simple_list = !self.simple_list;
    self.needs_rebuild_areas = true;
  }

  fn order_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text(
      "game_list",
      if self.ascending {
        "game_list.list.order.ascending"
      } else {
        "game_list.list.order.descending"
      },
    )
  }

  fn sort_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text("game_list", self.sort_field.key())
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

  fn action_hint_lines(
    &self,
    i18n: &I18nService,
    text_input: &TextInputService,
    max_width: u16,
  ) -> Vec<String> {
    let params = RichTextParams::from_action_map(&Self::action_map(), "game_list.");
    let rich = RichTextService::new();
    let keys = if text_input.is_focused(&self.objects, self.search_input) {
      vec!["game_list.action.search.back"]
    } else if text_input.is_focused(&self.objects, self.jump_input) {
      vec![
        "game_list.action.jump.back",
        "game_list.action.jump.confirm",
      ]
    } else {
      vec![
        "game_list.action.select",
        "game_list.action.flip",
        "game_list.action.scroll",
        "game_list.action.confirm",
        "game_list.action.list.back",
        "game_list.action.list.search",
        "game_list.action.list.order",
        "game_list.action.list.sort",
        "game_list.action.list.jump",
      ]
    };
    let items = keys
      .iter()
      .map(|key| i18n.get_runtime_text("game_list", key));

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
