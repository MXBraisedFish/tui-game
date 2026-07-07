use std::{cmp::Ordering, collections::HashSet, time::Duration};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, ImageConvertParams, ImageService, KeyState,
  LayoutService, MouseButton, Overflow, PackageAsset, PackageListEntry, PackageService, Rect,
  RenderService, RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner,
  ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarPolicy, ScrollbarVisibility,
  StorageService, TerminalColor, TextAlign, TextColor, TextInputCursorShape, TextInputEvent,
  TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams, TextInputService, TextStyle,
  UiEvent, UiObjectPool, UiObjectPoolOwner,
};

/// 游戏包详情页面的命令。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GamePackageCommand {
  Back,
  FocusSearch,
  BlurSearch,
  FocusJump,
  BlurJump,
  ScrollInfoUp,
  ScrollInfoDown,
  SubmitJump(String),
  ToggleEnabled,
  ToggleDebug,
  RequestToggleSafeMode,
}

/// 游戏包详情页面布局信息。
pub(crate) struct GamePackageLayout {
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
enum GameSortField {
  Title,
  Author,
  Status,
  Debug,
  SafeMode,
}

impl GameSortField {
  fn next(self) -> Self {
    match self {
      Self::Title => Self::Author,
      Self::Author => Self::Status,
      Self::Status => Self::Debug,
      Self::Debug => Self::SafeMode,
      Self::SafeMode => Self::Title,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Title => "game_pack.list.sort.title",
      Self::Author => "game_pack.list.sort.author",
      Self::Status => "game_pack.list.sort.status",
      Self::Debug => "game_pack.list.sort.debug",
      Self::SafeMode => "game_pack.list.sort.safe_mode",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SafeModeStatus {
  On,
  OffTemporary,
  OffPermanent,
}

impl SafeModeStatus {
  fn enabled(self) -> bool {
    self == Self::On
  }
}

/// 游戏包详情 UI：左右 33/67 分栏布局。
///
/// 左侧：搜索框 + 列表（翻页） + 翻页指示器，包裹在双线边框内。
/// 右侧：滚动信息盒，包裹在双线边框内。
/// 底部：操作提示栏。
pub struct GamePackageUi {
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
  sort_field: GameSortField,
  entries: Vec<PackageListEntry>,
  search_text: String,
  jump_text: String,
  simple_list: bool,
  temporary_safe_mode_disabled: HashSet<String>,
  needs_rebuild_areas: bool,
}

impl UiObjectPoolOwner for GamePackageUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for GamePackageUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl GamePackageUi {
  /// 初始化游戏包详情 UI。
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
      sort_field: GameSortField::Title,
      entries: Vec::new(),
      search_text: String::new(),
      jump_text: "1".to_string(),
      simple_list: false,
      temporary_safe_mode_disabled: HashSet::new(),
      needs_rebuild_areas: true,
    }
  }

  /// 返回按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "game_pack.flip_forward".to_string(),
        description: "Previous list page".to_string(),
        keys: vec![vec!["q".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.flip_backward".to_string(),
        description: "Next list page".to_string(),
        keys: vec![vec!["e".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.scroll_up".to_string(),
        description: "Scroll info up".to_string(),
        keys: vec![vec!["w".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.scroll_down".to_string(),
        description: "Scroll info down".to_string(),
        keys: vec![vec!["s".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.focus_up".to_string(),
        description: "Focus previous item".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.focus_down".to_string(),
        description: "Focus next item".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.confirm".to_string(),
        description: "Toggle selection".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.list.back".to_string(),
        description: "Go back to mods menu".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.debug".to_string(),
        description: "Toggle debug mode".to_string(),
        keys: vec![vec!["n".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.list".to_string(),
        description: "Toggle list style".to_string(),
        keys: vec![vec!["l".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.search".to_string(),
        description: "Search".to_string(),
        keys: vec![vec!["c".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.order".to_string(),
        description: "Toggle order".to_string(),
        keys: vec![vec!["z".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.sort".to_string(),
        description: "Toggle sort".to_string(),
        keys: vec![vec!["x".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.safe_mode".to_string(),
        description: "Toggle safe mode".to_string(),
        keys: vec![vec!["b".to_string()]],
      },
      ActionMapEntry {
        action: "game_pack.jump".to_string(),
        description: "Jump to page".to_string(),
        keys: vec![vec!["j".to_string()]],
      },
    ]
  }

  /// 处理 UI 事件，返回导航命令。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<GamePackageCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.handle_hover(*id);
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        Some(GamePackageCommand::FocusSearch)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.search_input => {
        Some(GamePackageCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        Some(GamePackageCommand::BlurSearch)
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search_text = value.clone();
        self.page = 1;
        self.selected_index = 0;
        self.needs_rebuild_areas = true;
        None
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.jump_input => {
        Some(GamePackageCommand::FocusJump)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.jump_input => {
        Some(GamePackageCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.jump_input => {
        Some(GamePackageCommand::BlurJump)
      }
      UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.jump_input => {
        Some(GamePackageCommand::SubmitJump(value.clone()))
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
      }) => Some(GamePackageCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "game_pack.focus_up" => {
          self.focus_previous();
          None
        }
        "game_pack.focus_down" => {
          self.focus_next();
          None
        }
        "game_pack.flip_forward" => {
          self.flip_page(-1);
          self.needs_rebuild_areas = true;
          None
        }
        "game_pack.flip_backward" => {
          self.flip_page(1);
          self.needs_rebuild_areas = true;
          None
        }
        "game_pack.search" => Some(GamePackageCommand::FocusSearch),
        "game_pack.jump" => Some(GamePackageCommand::FocusJump),
        "game_pack.order" => {
          self.toggle_order();
          None
        }
        "game_pack.sort" => {
          self.next_sort_field();
          None
        }
        "game_pack.scroll_up" => Some(GamePackageCommand::ScrollInfoUp),
        "game_pack.scroll_down" => Some(GamePackageCommand::ScrollInfoDown),
        "game_pack.list" => {
          self.toggle_list_style();
          None
        }
        "game_pack.safe_mode" => Some(GamePackageCommand::RequestToggleSafeMode),
        "game_pack.confirm" => Some(GamePackageCommand::ToggleEnabled),
        "game_pack.debug" => Some(GamePackageCommand::ToggleDebug),
        "game_pack.list.back" => Some(GamePackageCommand::Back),
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

  pub fn selected_safe_mode(&self) -> Option<bool> {
    self
      .selected_safe_mode_status()
      .map(SafeModeStatus::enabled)
  }

  fn selected_safe_mode_status(&self) -> Option<SafeModeStatus> {
    self
      .page_entries()
      .get(self.selected_index)
      .map(|entry| self.safe_mode_status(entry))
  }

  pub fn selected_mod_id(&self) -> Option<String> {
    self
      .page_entries()
      .get(self.selected_index)
      .map(|entry| entry.mod_id.clone())
  }

  pub fn toggle_selected_enabled(&mut self, storage: &StorageService) {
    let Some((mod_id, enabled)) =
      self.selected_entry_state(|entry| (entry.mod_id.clone(), !entry.enabled))
    else {
      return;
    };
    self.update_entry(&mod_id, |entry| entry.enabled = enabled);
    let _ = storage.update_game_package_state(&mod_id, |state| state.enabled = enabled);
  }

  pub fn toggle_selected_debug(&mut self, storage: &StorageService) {
    let Some((mod_id, debug)) =
      self.selected_entry_state(|entry| (entry.mod_id.clone(), !entry.debug))
    else {
      return;
    };
    self.update_entry(&mod_id, |entry| entry.debug = debug);
    let _ = storage.update_game_package_state(&mod_id, |state| state.debug = debug);
  }

  pub fn enable_selected_safe_mode(&mut self, storage: &StorageService) {
    let Some(mod_id) = self.selected_entry_state(|entry| entry.mod_id.clone()) else {
      return;
    };
    self.update_entry(&mod_id, |entry| entry.safe_mode = true);
    let _ = storage.update_game_package_state(&mod_id, |state| state.safe_mode = true);
  }

  pub fn disable_selected_safe_mode_temporary(&mut self) {
    let Some(mod_id) = self.selected_mod_id() else {
      return;
    };
    for item in &mut self.entries {
      if item.mod_id == mod_id {
        item.safe_mode = false;
      }
    }
    self.needs_rebuild_areas = true;
  }

  pub fn disable_selected_safe_mode_permanent(&mut self, storage: &StorageService) {
    let Some(mod_id) = self.selected_entry_state(|entry| entry.mod_id.clone()) else {
      return;
    };
    self.update_entry(&mod_id, |entry| entry.safe_mode = false);
    let _ = storage.update_game_package_state(&mod_id, |state| state.safe_mode = false);
  }

  pub fn scroll_info(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, lines: i32) {
    let _ = scroll_box.scroll_by(&mut self.objects, self.info_scroll, 0, lines, layout);
  }

  pub fn update(&mut self, dt: Duration) -> Option<GamePackageCommand> {
    let _ = dt;
    None
  }

  /// 渲染游戏包详情页面。
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
    temporary_safe_mode_disabled: &HashSet<String>,
    image: &mut ImageService,
    mouse_supported: bool,
  ) {
    self.sync_entries(package.mod_games(), storage, temporary_safe_mode_disabled);
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
    let info_content_height = self.info_content_height(i18n, positions.right_inner.width);
    scroll_box.set_content_size(
      &mut self.objects,
      self.info_scroll,
      positions.right_inner.width.max(1),
      info_content_height,
      layout,
    );
    canvas.prepare(&self.objects, layout);

    let info_scroll_y = scroll_box
      .scroll_y(&self.objects, self.info_scroll)
      .unwrap_or(0);
    self.draw_right_panel(
      render,
      canvas,
      layout,
      i18n,
      image,
      mouse_supported,
      &positions,
      info_scroll_y,
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
  ) -> GamePackageLayout {
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
    let list_item_height: u16 = if self.simple_list { 1 } else { 4 };
    let list_item_gap: u16 = if self.simple_list { 0 } else { 1 };
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
    let used_height = if visible_items == 0 {
      0
    } else {
      visible_items as u16 * list_item_height
        + visible_items.saturating_sub(1) as u16 * list_item_gap
    };
    let list_start_y = if self.simple_list {
      list_area_y
    } else {
      list_area_y.saturating_add(list_area_height.saturating_sub(used_height) / 2)
    };

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

    GamePackageLayout {
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
    image: &mut ImageService,
    pos: &GamePackageLayout,
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
      &i18n.get_runtime_text("game_pack", "game_pack.list"),
    );

    text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search_rect,
        placeholder: i18n.get_runtime_text("game_pack", "game_pack.list.search.placeholder"),
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
      if self.simple_list {
        self.draw_entry_simple(
          render,
          canvas,
          i18n,
          pos,
          entry,
          y,
          i == self.selected_index,
        );
      } else {
        self.draw_entry_card(
          render,
          canvas,
          layout,
          image,
          i18n,
          pos,
          entry,
          y,
          i == self.selected_index,
        );
      }
    }

    if entries.is_empty() {
      let text = i18n.get_runtime_text("game_pack", "game_pack.no.pack");
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
    let key_params = RichTextParams::from_action_map(&Self::action_map(), "game_pack.");
    if self.page > 1 {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.flip_forward_rect.x,
          y: pos.flip_forward_rect.y,
          text: format!(
            "f%<fg:bright_black>{}</fg>",
            i18n.get_runtime_text("game_pack", "game_pack.flip.forward")
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
            i18n.get_runtime_text("game_pack", "game_pack.flip.backward")
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
    layout: &LayoutService,
    image: &mut ImageService,
    i18n: &I18nService,
    pos: &GamePackageLayout,
    entry: &PackageListEntry,
    y: u16,
    focused: bool,
  ) {
    let marker_x = pos.left_inner.x;
    let image_x = marker_x.saturating_add(1);
    let text_x = image_x.saturating_add(9);
    let safe_x = pos
      .left_inner
      .x
      .saturating_add(pos.left_inner.width.saturating_sub(1));
    let text_width = safe_x.saturating_sub(text_x).max(1);

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
    let package_params = Self::package_rich_params(entry);
    self.draw_icon_asset(
      render,
      canvas,
      layout,
      image,
      &entry.icon,
      image_x,
      y,
      &package_params,
    );

    let status_key = if entry.enabled {
      "game_pack.list.status.on"
    } else {
      "game_pack.list.status.off"
    };
    let lines = [
      if entry.debug {
        format!(
          "f%[<fg:bright_magenta>{}</fg>]{}",
          i18n.get_runtime_text("game_pack", "game_pack.list.debug"),
          entry.title
        )
      } else {
        entry.title.clone()
      },
      format!(
        "f%<fg:bright_yellow>{}</fg>{}",
        i18n.get_runtime_text("game_pack", "game_pack.info.author"),
        entry.author
      ),
      format!(
        "f%<fg:bright_yellow>{}</fg>{}",
        i18n.get_runtime_text("game_pack", "game_pack.list.version"),
        entry.version
      ),
      format!(
        "f%<fg:bright_yellow>{}</fg>{}{}</fg>",
        i18n.get_runtime_text("game_pack", "game_pack.list.status"),
        if status_key == "game_pack.list.status.on" {
          "<fg:bright_green>"
        } else {
          "<fg:bright_red>"
        },
        i18n.get_runtime_text("game_pack", status_key)
      ),
    ];
    for (row, text) in lines.into_iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: text_x,
          y: y.saturating_add(row as u16),
          text,
          params: Some(package_params.clone()),
          wrap_mode: TextWrapMode::None,
          max_width: Some(text_width),
          overflow_marker: Some("...".to_string()),
          ..Default::default()
        },
      );
    }

    if !entry.safe_mode {
      for row in 0..4 {
        render.draw_host_text(
          canvas,
          &DrawTextParams {
            x: safe_x,
            y: y.saturating_add(row),
            text: "f%<fg:red>█</fg>".to_string(),
            ..Default::default()
          },
        );
      }
    }
  }

  fn draw_entry_simple(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &GamePackageLayout,
    entry: &PackageListEntry,
    y: u16,
    focused: bool,
  ) {
    let package_params = Self::package_rich_params(entry);
    let marker_x = pos.left_inner.x;
    let text_x = marker_x.saturating_add(1);
    let status_key = if entry.enabled {
      "game_pack.list.status.on"
    } else {
      "game_pack.list.status.off"
    };
    let status = i18n.get_runtime_text("game_pack", status_key);
    let right_width = status.width().saturating_add(3).min(u16::MAX as usize) as u16;
    let right_x = pos
      .left_inner
      .x
      .saturating_add(pos.left_inner.width.saturating_sub(right_width));
    let text_width = right_x.saturating_sub(text_x).max(1);

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
        ..Default::default()
      },
    );

    let title = if entry.debug {
      format!(
        "f%[<fg:bright_magenta>{}</fg>]{}",
        i18n.get_runtime_text("game_pack", "game_pack.list.debug"),
        entry.title
      )
    } else {
      entry.title.clone()
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: text_x,
        y,
        text: title,
        params: Some(package_params),
        wrap_mode: TextWrapMode::None,
        max_width: Some(text_width),
        overflow_marker: Some("...".to_string()),
        ..Default::default()
      },
    );

    let status_color = if entry.enabled {
      "bright_green"
    } else {
      "bright_red"
    };
    let safe_mark = if entry.safe_mode {
      " "
    } else {
      "<fg:red>█</fg>"
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: right_x,
        y,
        text: format!("f%[<fg:{}>{}</fg>]{}", status_color, status, safe_mark),
        max_width: Some(right_width),
        ..Default::default()
      },
    );
  }

  fn draw_icon_asset(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    image: &mut ImageService,
    asset: &PackageAsset,
    x: u16,
    y: u16,
    params: &RichTextParams,
  ) {
    if let PackageAsset::Image { path } = asset {
      if let Ok(text) = image.convert(ImageConvertParams {
        image_path: path.clone(),
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
            x,
            y,
            text,
            wrap_mode: TextWrapMode::Auto,
            max_width: Some(8),
            max_height: Some(4),
            ..Default::default()
          },
        );
        return;
      }
    }

    let fallback = PackageAsset::default_icon();
    let lines = match asset {
      PackageAsset::Text { lines, .. } => lines,
      PackageAsset::Image { .. } => match &fallback {
        PackageAsset::Text { lines, .. } => lines,
        PackageAsset::Image { .. } => return,
      },
    };
    for (row, line) in lines.iter().take(4).enumerate() {
      if line.trim_start().starts_with("f%") {
        render.draw_host_text(
          canvas,
          &DrawTextParams {
            x,
            y: y.saturating_add(row as u16),
            text: Self::fit_asset_rich_line(line, false, 8, layout, Some(params)),
            params: Some(params.clone()),
            wrap_mode: TextWrapMode::None,
            max_width: Some(8),
            max_height: Some(1),
            ..Default::default()
          },
        );
      } else {
        canvas.host_styled_text(x, y.saturating_add(row as u16), line, TextStyle::default());
      }
    }
  }

  fn draw_right_panel(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    image: &mut ImageService,
    mouse_supported: bool,
    pos: &GamePackageLayout,
    scroll_y: u16,
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
      &i18n.get_runtime_text("game_pack", "game_pack.info"),
    );

    let page_entries = self.page_entries();
    let Some(entry) = page_entries.get(self.selected_index) else {
      let text = i18n.get_runtime_text("game_pack", "game_pack.no.info");
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
      return;
    };

    self.draw_info_content(
      render,
      canvas,
      i18n,
      image,
      layout,
      entry,
      pos.right_inner,
      scroll_y,
      mouse_supported,
    );
  }

  fn draw_info_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    image: &mut ImageService,
    layout: &LayoutService,
    entry: &PackageListEntry,
    rect: Rect,
    scroll_y: u16,
    mouse_supported: bool,
  ) {
    let package_params = Self::package_rich_params(entry);
    let mut y = 0;
    self.draw_info_banner(
      render,
      canvas,
      image,
      layout,
      &entry.banner,
      &package_params,
      rect,
      scroll_y,
      y,
    );
    y += 15;
    self.draw_info_center_text(
      render,
      canvas,
      layout,
      rect,
      scroll_y,
      y,
      &entry.title,
      Some(&package_params),
    );
    y += 2;

    self.draw_info_subtitle(
      canvas,
      rect,
      scroll_y,
      y,
      i18n,
      "game_pack",
      "game_pack.info.subtitle.base",
    );
    y += 1;
    self.draw_info_pair(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.pack_name"),
      &entry.mod_id,
    );
    y += 1;
    self.draw_info_pair_rich_value(
      render,
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.author"),
      &entry.author,
      &package_params,
    );
    y += 1;
    self.draw_info_pair_rich_value(
      render,
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.version"),
      &entry.version,
      &package_params,
    );
    y += 2;

    self.draw_info_subtitle(
      canvas,
      rect,
      scroll_y,
      y,
      i18n,
      "game_pack",
      "game_pack.info.subtitle.config",
    );
    y += 1;
    let safe_mode_status = self.safe_mode_status(entry);
    self.draw_info_status(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.status"),
      i18n.get_runtime_text(
        "game_pack",
        if entry.enabled {
          "game_pack.info.status.on"
        } else {
          "game_pack.info.status.off"
        },
      ),
      if entry.enabled {
        Self::style(TerminalColor::BrightGreen)
      } else {
        Self::style(TerminalColor::BrightRed)
      },
    );
    y += 1;
    self.draw_info_status(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.debug"),
      i18n.get_runtime_text(
        "game_pack",
        if entry.debug {
          "game_pack.info.debug.on"
        } else {
          "game_pack.info.debug.off"
        },
      ),
      if entry.debug {
        Self::style(TerminalColor::BrightMagenta)
      } else {
        Self::hint_style()
      },
    );
    y += 1;
    let (mouse_key, mouse_color) = if !entry.mouse_required {
      ("game_pack.info.mouse.off", Self::hint_style())
    } else if mouse_supported {
      (
        "game_pack.info.mouse.on.support",
        Self::style(TerminalColor::BrightGreen),
      )
    } else {
      (
        "game_pack.info.mouse.on.unsupport",
        Self::style(TerminalColor::BrightRed),
      )
    };
    self.draw_info_status(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.mouse"),
      i18n.get_runtime_text("game_pack", mouse_key),
      mouse_color,
    );
    y += 1;
    self.draw_info_status(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.write"),
      i18n.get_runtime_text(
        "game_pack",
        if entry.write_required {
          "game_pack.info.write.on"
        } else {
          "game_pack.info.write.off"
        },
      ),
      if entry.write_required {
        Self::style(TerminalColor::BrightRed)
      } else {
        Self::hint_style()
      },
    );
    y += 1;
    self.draw_info_status(
      canvas,
      rect,
      scroll_y,
      y,
      i18n.get_runtime_text("game_pack", "game_pack.info.safe_mode"),
      i18n.get_runtime_text(
        "game_pack",
        match safe_mode_status {
          SafeModeStatus::On => "game_pack.info.safe_mode.on",
          SafeModeStatus::OffTemporary => "game_pack.info.safe_mode.off.temporary",
          SafeModeStatus::OffPermanent => "game_pack.info.safe_mode.off.permanent",
        },
      ),
      if safe_mode_status.enabled() {
        Self::hint_style()
      } else {
        Self::style(TerminalColor::BrightRed)
      },
    );
    y += 2;

    self.draw_info_subtitle(
      canvas,
      rect,
      scroll_y,
      y,
      i18n,
      "game_pack",
      "game_pack.info.subtitle.description",
    );
    y += 1;
    let description = Self::package_visible_text(entry, &entry.description);
    for (offset, line) in Self::wrap_plain_lines(&description, rect.width)
      .into_iter()
      .enumerate()
    {
      self.draw_info_text(
        canvas,
        rect,
        scroll_y,
        0,
        y.saturating_add(offset as u16),
        &line,
        TextStyle::default(),
      );
    }
  }

  fn draw_info_banner(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    image: &mut ImageService,
    layout: &LayoutService,
    asset: &PackageAsset,
    params: &RichTextParams,
    rect: Rect,
    scroll_y: u16,
    y: u16,
  ) {
    const BANNER_WIDTH: u16 = 60;
    const BANNER_HEIGHT: u16 = 14;

    let x = rect.width.saturating_sub(BANNER_WIDTH) / 2;
    if let PackageAsset::Image { path } = asset {
      if let Ok(text) = image.convert(ImageConvertParams {
        image_path: path.clone(),
        output_width: BANNER_WIDTH as u32,
        output_height: BANNER_HEIGHT as u32,
        square_crop: false,
        scale: 1.0,
        cache: true,
        ..Default::default()
      }) {
        let is_rich = text.starts_with("f%");
        for (row, line) in text.lines().take(BANNER_HEIGHT as usize).enumerate() {
          self.draw_info_rich_text(
            render,
            canvas,
            rect,
            scroll_y,
            x,
            y.saturating_add(row as u16),
            Self::fit_asset_rich_line(line, is_rich, BANNER_WIDTH, layout, Some(params)),
            Some(BANNER_WIDTH),
            Some(params),
          );
        }
        return;
      }
    }

    let fallback = PackageAsset::default_banner();
    let lines = match asset {
      PackageAsset::Text { lines, .. } => lines,
      PackageAsset::Image { .. } => match &fallback {
        PackageAsset::Text { lines, .. } => lines,
        PackageAsset::Image { .. } => return,
      },
    };
    for (row, line) in lines.iter().take(BANNER_HEIGHT as usize).enumerate() {
      self.draw_info_rich_text(
        render,
        canvas,
        rect,
        scroll_y,
        x,
        y.saturating_add(row as u16),
        Self::fit_asset_rich_line(line, false, BANNER_WIDTH, layout, Some(params)),
        Some(BANNER_WIDTH),
        Some(params),
      );
    }
  }

  fn fit_asset_rich_line(
    line: &str,
    full_text_is_rich: bool,
    width: u16,
    layout: &LayoutService,
    params: Option<&RichTextParams>,
  ) -> String {
    let source = if full_text_is_rich && !line.starts_with("f%") {
      format!("f%{}", line)
    } else {
      line.to_string()
    };
    let Some(body) = source.trim_start().strip_prefix("f%") else {
      return source;
    };

    let body = body.trim_end();
    let rich_body = format!("f%{}", body);
    let text_width = layout.get_text_width(&rich_body, params).min(width);
    let padding = width.saturating_sub(text_width);
    let left = padding.saturating_add(1) / 2;
    let right = padding.saturating_sub(left);
    format!(
      "f%{}{}{}",
      " ".repeat(left as usize),
      body,
      " ".repeat(right as usize)
    )
  }

  fn draw_info_subtitle(
    &self,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    y: u16,
    i18n: &I18nService,
    namespace: &str,
    key: &str,
  ) {
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      0,
      y,
      &i18n.get_runtime_text(namespace, key),
      Self::style(TerminalColor::BrightYellow),
    );
  }

  fn draw_info_pair(
    &self,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    y: u16,
    label: String,
    value: &str,
  ) {
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      0,
      y,
      &label,
      Self::style(TerminalColor::BrightBlue),
    );
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      label.width().min(u16::MAX as usize) as u16,
      y,
      value,
      TextStyle::default(),
    );
  }

  fn draw_info_pair_rich_value(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    y: u16,
    label: String,
    value: &str,
    params: &RichTextParams,
  ) {
    let x = label.width().min(u16::MAX as usize) as u16;
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      0,
      y,
      &label,
      Self::style(TerminalColor::BrightBlue),
    );
    self.draw_info_rich_text(
      render,
      canvas,
      rect,
      scroll_y,
      x,
      y,
      value.to_string(),
      Some(rect.width.saturating_sub(x)),
      Some(params),
    );
  }

  fn draw_info_status(
    &self,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    y: u16,
    label: String,
    value: String,
    value_style: TextStyle,
  ) {
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      0,
      y,
      &label,
      Self::style(TerminalColor::BrightBlue),
    );
    self.draw_info_text(
      canvas,
      rect,
      scroll_y,
      label.width().min(u16::MAX as usize) as u16,
      y,
      &value,
      value_style,
    );
  }

  fn draw_info_center_text(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    rect: Rect,
    scroll_y: u16,
    y: u16,
    text: &str,
    params: Option<&RichTextParams>,
  ) {
    let width = layout.get_text_width(text, params);
    let x = rect.width.saturating_sub(width.min(rect.width)) / 2;
    self.draw_info_rich_text(
      render,
      canvas,
      rect,
      scroll_y,
      x,
      y,
      text.to_string(),
      Some(rect.width.saturating_sub(x)),
      params,
    );
  }

  fn draw_info_text(
    &self,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    x: u16,
    y: u16,
    text: &str,
    style: TextStyle,
  ) {
    let Some(screen_y) = y.checked_sub(scroll_y) else {
      return;
    };
    if screen_y >= rect.height || x >= rect.width {
      return;
    }
    canvas.host_styled_text(
      rect.x.saturating_add(x),
      rect.y.saturating_add(screen_y),
      text,
      style,
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_info_rich_text(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    rect: Rect,
    scroll_y: u16,
    x: u16,
    y: u16,
    text: String,
    max_width: Option<u16>,
    params: Option<&RichTextParams>,
  ) {
    let Some(screen_y) = y.checked_sub(scroll_y) else {
      return;
    };
    if screen_y >= rect.height || x >= rect.width {
      return;
    }
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x.saturating_add(x),
        y: rect.y.saturating_add(screen_y),
        text,
        params: params.cloned(),
        wrap_mode: TextWrapMode::None,
        max_width,
        max_height: Some(1),
        ..Default::default()
      },
    );
  }

  fn wrap_plain_lines(text: &str, width: u16) -> Vec<String> {
    if width == 0 {
      return Vec::new();
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
      let mut line = String::new();
      let mut line_width = 0usize;
      for grapheme in raw_line.graphemes(true) {
        let grapheme_width = grapheme.width();
        if line_width > 0 && line_width + grapheme_width > width as usize {
          lines.push(std::mem::take(&mut line));
          line_width = 0;
        }
        line.push_str(grapheme);
        line_width += grapheme_width;
      }
      lines.push(line);
    }

    if lines.is_empty() {
      lines.push(String::new());
    }
    lines
  }

  fn style(color: TerminalColor) -> TextStyle {
    TextStyle {
      foreground: Some(TextColor::Terminal(color)),
      ..Default::default()
    }
  }

  fn hint_style() -> TextStyle {
    TextStyle {
      foreground: Some(TextColor::Rgb {
        r: 85,
        g: 87,
        b: 83,
      }),
      ..Default::default()
    }
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
    pos: &GamePackageLayout,
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
    pos: &GamePackageLayout,
  ) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "game_pack.");
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

  fn info_content_height(&self, _i18n: &I18nService, width: u16) -> u16 {
    let page_entries = self.page_entries();
    let Some(entry) = page_entries.get(self.selected_index) else {
      return 1;
    };
    let description_lines = Self::wrapped_plain_line_count(&entry.description, width).max(1);
    14 + 1 + 1 + 1 + 4 + 1 + 6 + 1 + 1 + description_lines
  }

  fn wrapped_plain_line_count(text: &str, width: u16) -> u16 {
    let limit = width as usize;
    if limit == 0 || text.is_empty() {
      return 1;
    }
    let mut lines = 1;
    let mut used = 0;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
      if grapheme == "\n" {
        lines += 1;
        used = 0;
        continue;
      }
      let w = grapheme.width();
      if used > 0 && used + w > limit {
        lines += 1;
        used = 0;
      }
      used += w.min(limit);
    }
    lines
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
      })
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
      .then(
        Self::package_visible_text(a, &a.title)
          .width()
          .cmp(&Self::package_visible_text(b, &b.title).width()),
      )
      .then(Self::package_visible_text(a, &a.title).cmp(&Self::package_visible_text(b, &b.title)))
      .then(a.mod_id.width().cmp(&b.mod_id.width()))
      .then(a.mod_id.cmp(&b.mod_id))
  }

  fn sort_value(&self, entry: &PackageListEntry) -> String {
    match self.sort_field {
      GameSortField::Title => Self::package_visible_text(entry, &entry.title),
      GameSortField::Author => Self::package_visible_text(entry, &entry.author),
      GameSortField::Status => format!("{}", entry.enabled),
      GameSortField::Debug => format!("{}", entry.debug),
      GameSortField::SafeMode => format!("{}", entry.safe_mode),
    }
  }

  fn safe_mode_status(&self, entry: &PackageListEntry) -> SafeModeStatus {
    if entry.safe_mode {
      SafeModeStatus::On
    } else if self.temporary_safe_mode_disabled.contains(&entry.mod_id) {
      SafeModeStatus::OffTemporary
    } else {
      SafeModeStatus::OffPermanent
    }
  }

  fn sync_entries(
    &mut self,
    mut entries: Vec<PackageListEntry>,
    storage: &StorageService,
    temporary_safe_mode_disabled: &HashSet<String>,
  ) {
    self.temporary_safe_mode_disabled = temporary_safe_mode_disabled.clone();
    let profile = storage.read_package_state_or_default();
    for entry in &mut entries {
      if let Some(state) = profile.games.get(&entry.mod_id) {
        entry.enabled = state.enabled;
        entry.debug = state.debug;
        entry.safe_mode = state.safe_mode;
      }
      if temporary_safe_mode_disabled.contains(&entry.mod_id) {
        entry.safe_mode = false;
      }
    }
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

  fn selected_entry_state<T>(&self, f: impl FnOnce(&PackageListEntry) -> T) -> Option<T> {
    self.page_entries().get(self.selected_index).map(f)
  }

  fn update_entry(&mut self, mod_id: &str, f: impl Fn(&mut PackageListEntry)) {
    for entry in &mut self.entries {
      if entry.mod_id == mod_id {
        f(entry);
      }
    }
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

  fn toggle_list_style(&mut self) {
    self.simple_list = !self.simple_list;
    self.needs_rebuild_areas = true;
  }

  fn order_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text(
      "game_pack",
      if self.ascending {
        "game_pack.list.order.ascending"
      } else {
        "game_pack.list.order.descending"
      },
    )
  }

  fn sort_bar_text(&self, i18n: &I18nService) -> String {
    i18n.get_runtime_text("game_pack", self.sort_field.key())
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
    let params = RichTextParams::from_action_map(&Self::action_map(), "game_pack.");
    let rich = RichTextService::new();
    let keys = if text_input.is_focused(&self.objects, self.search_input) {
      vec!["game_pack.action.search.back"]
    } else if text_input.is_focused(&self.objects, self.jump_input) {
      vec![
        "game_pack.action.jump.back",
        "game_pack.action.jump.confirm",
      ]
    } else {
      let safe_key = if self.selected_safe_mode().unwrap_or(true) {
        "game_pack.action.safe_mode.off"
      } else {
        "game_pack.action.safe_mode.on"
      };
      let debug_key = if self
        .page_entries()
        .get(self.selected_index)
        .is_some_and(|entry| entry.debug)
      {
        "game_pack.action.debug.on"
      } else {
        "game_pack.action.debug.off"
      };
      vec![
        "game_pack.action.select",
        "game_pack.action.flip",
        "game_pack.action.scroll",
        "game_pack.action.confirm",
        "game_pack.action.list.back",
        safe_key,
        debug_key,
        "game_pack.action.list.detail2simple",
        "game_pack.action.list.search",
        "game_pack.action.list.order",
        "game_pack.action.list.sort",
        "game_pack.action.list.jump",
      ]
    };
    let items = keys
      .iter()
      .map(|key| i18n.get_runtime_text("game_pack", key));

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
