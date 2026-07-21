use std::{cmp::Ordering, time::Duration};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DisplaySourceMode, DrawTextParams, HitAreaEvent,
  HitAreaId, HitAreaOptions, HitAreaService, I18nService, KeyState, LayoutService, LogService,
  MouseButton, Overflow, PackageListEntry, PackageService, PackageSource, Rect, RenderService,
  RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxId,
  ScrollBoxOptions, ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility,
  StorageService, TerminalColor, TextColor, TextInputEvent, TextInputId, TextInputMode,
  TextInputOptions, TextInputRenderParams, TextInputService, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const ACTIVE_BORDER: TextColor = TextColor::Rgb {
  r: 95,
  g: 215,
  b: 105,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveList {
  Disabled,
  Enabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortField {
  Name,
  Source,
}

impl SortField {
  fn next(self) -> Self {
    match self {
      Self::Name => Self::Source,
      Self::Source => Self::Name,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Name => "screensaver_list.sort.title",
      Self::Source => "screensaver_list.sort.source",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreensaverListCommand {
  Back,
  FocusSearch,
  BlurSearch,
  SetEnabled { id: String, enabled: bool },
  SaveOrder(Vec<String>),
  Scroll(i32),
}

struct ScreensaverListLayout {
  title_x: u16,
  title_y: u16,
  left: Rect,
  right: Rect,
  left_search: Rect,
  left_sort_y: u16,
  left_list: Rect,
  right_list: Rect,
  hint_y: u16,
  hint_lines: Vec<String>,
}

pub struct ScreensaverListUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  search_input: TextInputId,
  left_scroll: ScrollBoxId,
  right_scroll: ScrollBoxId,
  back_area: HitAreaId,
  left_panel_area: HitAreaId,
  right_panel_area: HitAreaId,
  search_area: HitAreaId,
  order_area: HitAreaId,
  sort_area: HitAreaId,
  left_areas: Vec<HitAreaId>,
  right_areas: Vec<HitAreaId>,
  entries: Vec<PackageListEntry>,
  enabled_order: Vec<String>,
  active: ActiveList,
  left_selected: usize,
  right_selected: usize,
  right_locked: bool,
  search_text: String,
  ascending: bool,
  sort_field: SortField,
  source_mode: DisplaySourceMode,
  initial_focus_resolved: bool,
}

impl UiObjectPoolOwner for ScreensaverListUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ScreensaverListUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl ScreensaverListUi {
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
    let make_scroll = |objects: &mut UiObjectPool| {
      scroll_box
        .create(
          objects,
          ScrollBoxOptions {
            rect: Rect::default(),
            content_width: 1,
            content_height: 1,
            overflow_x: Overflow::Hidden,
            overflow_y: Overflow::Auto,
            scrollbar: ScrollbarPolicy {
              vertical: ScrollbarVisibility::Auto,
              horizontal: ScrollbarVisibility::Never,
            },
            scrollbar_layout: ScrollbarLayout::Inside,
            ..Default::default()
          },
        )
        .expect("failed to create screensaver list scroll box")
    };
    let left_scroll = make_scroll(&mut objects);
    let right_scroll = make_scroll(&mut objects);
    Self {
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      left_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      right_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      search_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      order_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      sort_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      left_areas: Vec::new(),
      right_areas: Vec::new(),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      search_input,
      left_scroll,
      right_scroll,
      entries: Vec::new(),
      enabled_order: Vec::new(),
      active: ActiveList::Disabled,
      left_selected: 0,
      right_selected: 0,
      right_locked: false,
      search_text: String::new(),
      ascending: true,
      sort_field: SortField::Name,
      source_mode: DisplaySourceMode::All,
      initial_focus_resolved: false,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    [
      ("screensaver_list.focus_up.move_up", "up"),
      ("screensaver_list.focus_down.move_down", "down"),
      ("screensaver_list.scroll_up", "w"),
      ("screensaver_list.scroll_down", "s"),
      ("screensaver_list.confirm", "enter"),
      ("screensaver_list.back", "esc"),
      ("screensaver_list.lock_unlock", "b"),
      ("screensaver_list.order", "z"),
      ("screensaver_list.sort", "x"),
      ("screensaver_list.search", "c"),
      ("screensaver_list.switch", "tab"),
    ]
    .into_iter()
    .map(|(action, key)| ActionMapEntry {
      action: action.to_string(),
      description: action.to_string(),
      keys: vec![vec![key.to_string()]],
    })
    .collect()
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ScreensaverListCommand> {
    match event {
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        Some(ScreensaverListCommand::FocusSearch)
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search_text = value.clone();
        self.left_selected = 0;
        None
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        Some(ScreensaverListCommand::BlurSearch)
      }
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        if matches!(
          *id,
          id if id == self.left_panel_area
            || id == self.search_area
            || id == self.order_area
            || id == self.sort_area
        ) {
          self.active = ActiveList::Disabled;
          self.right_locked = false;
        } else if *id == self.right_panel_area {
          self.active = ActiveList::Enabled;
          self.right_locked = false;
        } else if let Some(index) = self.left_areas.iter().position(|area| area == id) {
          self.active = ActiveList::Disabled;
          self.left_selected = index;
        } else if let Some(index) = self.right_areas.iter().position(|area| area == id) {
          self.active = ActiveList::Enabled;
          if !self.right_locked {
            self.right_selected = index;
          }
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.search_area => Some(ScreensaverListCommand::FocusSearch),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.left_panel_area || *id == self.right_panel_area => {
        self.active = if *id == self.left_panel_area {
          ActiveList::Disabled
        } else {
          ActiveList::Enabled
        };
        self.right_locked = false;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if self.left_areas.contains(id) || self.right_areas.contains(id) => {
        if let Some(index) = self.left_areas.iter().position(|area| area == id) {
          self.active = ActiveList::Disabled;
          self.left_selected = index;
          return self.toggle_enabled();
        }
        if let Some(index) = self.right_areas.iter().position(|area| area == id) {
          self.active = ActiveList::Enabled;
          self.right_selected = index;
          return self.toggle_enabled();
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.order_area => {
        self.active = ActiveList::Disabled;
        self.right_locked = false;
        self.toggle_order();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.sort_area => {
        self.active = ActiveList::Disabled;
        self.right_locked = false;
        self.toggle_sort();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(ScreensaverListCommand::Back),
      UiEvent::Action(action) if action.state == KeyState::Pressed => {
        match action.action.as_str() {
          "screensaver_list.focus_up.move_up" => {
            if self.move_selection(-1) {
              return Some(self.take_order_command());
            }
          }
          "screensaver_list.focus_down.move_down" => {
            if self.move_selection(1) {
              return Some(self.take_order_command());
            }
          }
          "screensaver_list.scroll_up" => return Some(ScreensaverListCommand::Scroll(-3)),
          "screensaver_list.scroll_down" => return Some(ScreensaverListCommand::Scroll(3)),
          "screensaver_list.confirm" => return self.toggle_enabled(),
          "screensaver_list.back" => return Some(ScreensaverListCommand::Back),
          "screensaver_list.lock_unlock" if self.active == ActiveList::Enabled => {
            self.right_locked = !self.right_locked;
          }
          "screensaver_list.order" if self.active == ActiveList::Disabled => self.toggle_order(),
          "screensaver_list.sort" if self.active == ActiveList::Disabled => self.toggle_sort(),
          "screensaver_list.search" if self.active == ActiveList::Disabled => {
            return Some(ScreensaverListCommand::FocusSearch);
          }
          "screensaver_list.switch" => self.switch_list(),
          _ => {}
        }
        None
      }
      _ => None,
    }
  }

  pub fn update(&mut self, _dt: Duration) -> Option<ScreensaverListCommand> {
    None
  }

  pub fn focus_search(&mut self, text_input: &mut TextInputService) {
    self.active = ActiveList::Disabled;
    let _ = text_input.focus(&mut self.objects, self.search_input);
  }

  pub fn blur_search(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn scroll_active(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    let id = match self.active {
      ActiveList::Disabled => self.left_scroll,
      ActiveList::Enabled => self.right_scroll,
    };
    let _ = scroll_box.scroll_by(&mut self.objects, id, 0, dy, layout);
    let top = scroll_box.scroll_y(&self.objects, id).unwrap_or(0) as usize;
    let height = scroll_box
      .visible_content_height(&self.objects, id, layout)
      .unwrap_or(0) as usize;
    if height == 0 {
      return;
    }
    match self.active {
      ActiveList::Disabled => {
        let len = self.disabled_entries().len();
        if len > 0 {
          self.left_selected = self
            .left_selected
            .clamp(top, top.saturating_add(height - 1).min(len - 1));
        }
      }
      ActiveList::Enabled => {
        let len = self.enabled_entries().len();
        if len > 0 {
          self.right_selected = self
            .right_selected
            .clamp(top, top.saturating_add(height - 1).min(len - 1));
        }
      }
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) {
    let pos = self.compute_layout(layout, i18n, text_input);
    self.draw_frame(render, canvas, i18n, &pos);
    self.draw_search(text_input, canvas, i18n, &pos);
    self.draw_lists(render, canvas, layout, i18n, &pos);
    self.draw_hints(render, canvas, layout, i18n, text_input, &pos);
    self.register_hit_areas(hit_area, scroll_box, canvas, &pos);
  }

  pub(crate) fn prepare_surfaces(
    &mut self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
    package: &PackageService,
    storage: &StorageService,
    log: &mut LogService,
  ) {
    self.sync_entries(package, storage, log);
    let pos = self.compute_layout(layout, i18n, text_input);
    self.prepare_scroll_boxes(scroll_box, layout, &pos);
  }

  fn sync_entries(
    &mut self,
    package: &PackageService,
    storage: &StorageService,
    log: &mut LogService,
  ) {
    let old_left = self
      .disabled_entries()
      .get(self.left_selected)
      .map(|entry| entry.mod_id.clone());
    let old_right = self
      .enabled_entries()
      .get(self.right_selected)
      .map(|entry| entry.mod_id.clone());
    let profile = storage.read_package_state_or_default(log);
    self.source_mode = storage.display_settings_profile().screensaver_source;
    self.entries = package.screensaver_list();
    self.entries.retain(|entry| {
      entry.source != PackageSource::Mod
        || profile
          .screensavers
          .get(&entry.mod_id)
          .map_or(profile.defaults.enabled, |state| state.enabled)
    });
    for entry in &mut self.entries {
      let state = profile.screensavers.get(&entry.mod_id);
      // 此页面中的 enabled 表示局内屏保列表状态，不是包管理器总开关。
      entry.enabled = state.map_or(true, |state| {
        state.playlist_enabled || state.order.is_some()
      });
      entry.debug = state.map_or(profile.defaults.debug, |state| state.debug);
    }
    let left_ids: Vec<_> = self
      .disabled_entries()
      .into_iter()
      .map(|entry| entry.mod_id.clone())
      .collect();
    self.left_selected = old_left
      .and_then(|id| left_ids.iter().position(|entry_id| entry_id == &id))
      .unwrap_or_else(|| self.left_selected.min(left_ids.len().saturating_sub(1)));
    let right_ids: Vec<_> = self
      .enabled_entries_with_profile(&profile)
      .into_iter()
      .map(|entry| entry.mod_id.clone())
      .collect();
    self.right_selected = old_right
      .and_then(|id| right_ids.iter().position(|entry_id| entry_id == &id))
      .unwrap_or_else(|| self.right_selected.min(right_ids.len().saturating_sub(1)));
    self.enabled_order = right_ids;
    if !self.initial_focus_resolved {
      self.initial_focus_resolved = true;
      if left_ids.is_empty() && !self.enabled_order.is_empty() {
        self.active = ActiveList::Enabled;
      }
    }
  }

  fn disabled_entries(&self) -> Vec<&PackageListEntry> {
    let query = self.search_text.to_lowercase();
    let mut entries: Vec<_> = self
      .entries
      .iter()
      .filter(|entry| !entry.enabled)
      .filter(|entry| {
        query.is_empty()
          || RichTextService::new()
            .visible_text(&entry.screensaver_name, Some(&Self::package_params(entry)))
            .to_lowercase()
            .contains(&query)
      })
      .collect();
    entries.sort_by(|a, b| self.compare_disabled(a, b));
    entries
  }

  fn enabled_entries_with_profile<'a>(
    &'a self,
    profile: &crate::host_engine::services::PackageStateProfile,
  ) -> Vec<&'a PackageListEntry> {
    let mut entries: Vec<_> = self.entries.iter().filter(|entry| entry.enabled).collect();
    entries.sort_by(|a, b| {
      let a_order = profile
        .screensavers
        .get(&a.mod_id)
        .and_then(|state| state.order);
      let b_order = profile
        .screensavers
        .get(&b.mod_id)
        .and_then(|state| state.order);
      a_order
        .cmp(&b_order)
        .then_with(|| Self::visible_name(a).cmp(&Self::visible_name(b)))
    });
    entries
  }

  fn enabled_entries(&self) -> Vec<&PackageListEntry> {
    let mut entries: Vec<_> = self.entries.iter().filter(|entry| entry.enabled).collect();
    entries.sort_by_key(|entry| {
      self
        .enabled_order
        .iter()
        .position(|id| id == &entry.mod_id)
        .unwrap_or(usize::MAX)
    });
    entries
  }

  fn compare_disabled(&self, a: &PackageListEntry, b: &PackageListEntry) -> Ordering {
    let order = match self.sort_field {
      SortField::Name => Self::visible_name(a).cmp(&Self::visible_name(b)),
      SortField::Source => Self::source_rank(&a.source)
        .cmp(&Self::source_rank(&b.source))
        .then_with(|| Self::visible_name(a).cmp(&Self::visible_name(b))),
    };
    if self.ascending {
      order
    } else {
      order.reverse()
    }
  }

  fn source_rank(source: &PackageSource) -> u8 {
    match source {
      PackageSource::Official => 0,
      PackageSource::Mod => 1,
    }
  }

  fn visible_name(entry: &PackageListEntry) -> String {
    RichTextService::new().visible_text(&entry.screensaver_name, Some(&Self::package_params(entry)))
  }

  fn package_params(entry: &PackageListEntry) -> RichTextParams {
    RichTextParams::from_key_actions(&entry.key_actions)
  }

  fn toggle_enabled(&mut self) -> Option<ScreensaverListCommand> {
    match self.active {
      ActiveList::Disabled => {
        let id = self
          .disabled_entries()
          .get(self.left_selected)?
          .mod_id
          .clone();
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.mod_id == id) {
          entry.enabled = true;
        }
        self.enabled_order.push(id.clone());
        self.left_selected = self
          .left_selected
          .min(self.disabled_entries().len().saturating_sub(1));
        Some(ScreensaverListCommand::SetEnabled { id, enabled: true })
      }
      ActiveList::Enabled => {
        let id = self
          .enabled_entries()
          .get(self.right_selected)?
          .mod_id
          .clone();
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.mod_id == id) {
          entry.enabled = false;
        }
        self.enabled_order.retain(|entry_id| entry_id != &id);
        self.right_locked = false;
        self.right_selected = self
          .right_selected
          .saturating_sub(1)
          .min(self.enabled_entries().len().saturating_sub(1));
        Some(ScreensaverListCommand::SetEnabled { id, enabled: false })
      }
    }
  }

  fn move_selection(&mut self, delta: i32) -> bool {
    match self.active {
      ActiveList::Disabled => {
        self.left_selected = move_index(self.left_selected, self.disabled_entries().len(), delta);
        false
      }
      ActiveList::Enabled if self.right_locked => {
        let mut ids: Vec<String> = self
          .enabled_entries()
          .into_iter()
          .map(|entry| entry.mod_id.clone())
          .collect();
        if ids.is_empty() {
          return false;
        }
        let next = move_index(self.right_selected, ids.len(), delta);
        if next != self.right_selected {
          ids.swap(self.right_selected, next);
          self.right_selected = next;
          self.enabled_order = ids;
          return true;
        }
        false
      }
      ActiveList::Enabled => {
        self.right_selected = move_index(self.right_selected, self.enabled_entries().len(), delta);
        false
      }
    }
  }

  pub fn take_order_command(&self) -> ScreensaverListCommand {
    ScreensaverListCommand::SaveOrder(
      self
        .enabled_entries()
        .into_iter()
        .map(|entry| entry.mod_id.clone())
        .collect(),
    )
  }

  fn switch_list(&mut self) {
    self.active = match self.active {
      ActiveList::Disabled => ActiveList::Enabled,
      ActiveList::Enabled => ActiveList::Disabled,
    };
    self.right_locked = false;
  }

  fn toggle_order(&mut self) {
    let selected = self
      .disabled_entries()
      .get(self.left_selected)
      .map(|entry| entry.mod_id.clone());
    self.ascending = !self.ascending;
    self.restore_left_selection(selected);
  }

  fn toggle_sort(&mut self) {
    let selected = self
      .disabled_entries()
      .get(self.left_selected)
      .map(|entry| entry.mod_id.clone());
    self.sort_field = self.sort_field.next();
    self.restore_left_selection(selected);
  }

  fn restore_left_selection(&mut self, selected: Option<String>) {
    self.left_selected = selected
      .and_then(|id| {
        self
          .disabled_entries()
          .iter()
          .position(|entry| entry.mod_id == id)
      })
      .unwrap_or(0);
  }

  fn compute_layout(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> ScreensaverListLayout {
    let viewport = layout.developer_viewport_rect();
    let hint_lines = self.hint_lines(i18n, text_input, viewport.width);
    let hint_h = hint_lines.len().max(1) as u16;
    let title = i18n.get_runtime_text("screensaver_list", "screensaver_list.title");
    let title_w = layout.get_text_width(&title, None);
    let title_y = viewport.y;
    let content_y = viewport.y.saturating_add(1);
    let content_h = viewport.height.saturating_sub(1 + hint_h);
    let left_w = viewport.width / 2;
    let right_w = viewport.width.saturating_sub(left_w);
    let left = Rect {
      x: viewport.x,
      y: content_y,
      width: left_w,
      height: content_h,
    };
    let right = Rect {
      x: viewport.x.saturating_add(left_w),
      y: content_y,
      width: right_w,
      height: content_h,
    };
    let left_search = Rect {
      x: left.x.saturating_add(1),
      y: left.y.saturating_add(1),
      width: left.width.saturating_sub(2),
      height: 1,
    };
    let left_sort_y = left.y.saturating_add(2);
    let left_list = Rect {
      x: left.x.saturating_add(1),
      y: left.y.saturating_add(3),
      width: left.width.saturating_sub(2),
      height: left.height.saturating_sub(4),
    };
    let right_list = Rect {
      x: right.x.saturating_add(1),
      y: right.y.saturating_add(1),
      width: right.width.saturating_sub(2),
      height: right.height.saturating_sub(2),
    };
    ScreensaverListLayout {
      title_x: viewport
        .x
        .saturating_add(viewport.width.saturating_sub(title_w) / 2),
      title_y,
      left,
      right,
      left_search,
      left_sort_y,
      left_list,
      right_list,
      hint_y: viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(hint_h),
      hint_lines,
    }
  }

  fn prepare_scroll_boxes(
    &mut self,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
    pos: &ScreensaverListLayout,
  ) {
    let left_len = self.disabled_entries().len() as u16;
    let right_len = self.enabled_entries().len() as u16;
    let viewport = layout.developer_viewport_rect();
    let local_rect = |rect: Rect| Rect {
      x: rect.x.saturating_sub(viewport.x),
      y: rect.y.saturating_sub(viewport.y),
      width: rect.width,
      height: rect.height,
    };
    let left_rect = local_rect(pos.left_list);
    let right_rect = local_rect(pos.right_list);
    let _ = scroll_box.set_rect(&mut self.objects, self.left_scroll, left_rect, layout);
    let _ = scroll_box.set_rect(&mut self.objects, self.right_scroll, right_rect, layout);
    let _ = scroll_box.set_content_size(
      &mut self.objects,
      self.left_scroll,
      left_rect.width.saturating_sub(1).max(1),
      left_len.max(pos.left_list.height).max(1),
      layout,
    );
    let _ = scroll_box.set_content_size(
      &mut self.objects,
      self.right_scroll,
      right_rect.width.saturating_sub(1).max(1),
      right_len.max(pos.right_list.height).max(1),
      layout,
    );
    self.ensure_active_selection_visible(scroll_box, layout);
  }

  fn ensure_active_selection_visible(
    &mut self,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let (id, selected) = match self.active {
      ActiveList::Disabled => (self.left_scroll, self.left_selected),
      ActiveList::Enabled => (self.right_scroll, self.right_selected),
    };
    let height = scroll_box
      .visible_content_height(&self.objects, id, layout)
      .unwrap_or(0);
    if height == 0 {
      return;
    }
    let top = scroll_box.scroll_y(&self.objects, id).unwrap_or(0) as usize;
    let bottom = top.saturating_add(height as usize);
    let target = if selected < top {
      Some(selected)
    } else if selected >= bottom {
      Some(selected.saturating_add(1).saturating_sub(height as usize))
    } else {
      None
    };
    if let Some(y) = target {
      let _ = scroll_box.scroll_to(&mut self.objects, id, 0, y as u16, layout);
    }
  }

  fn draw_frame(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &ScreensaverListLayout,
  ) {
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.title_x,
        y: pos.title_y,
        text: format!(
          "f%<fg:bright_magenta><b>{}</b></fg>",
          i18n.get_runtime_text("screensaver_list", "screensaver_list.title")
        ),
        ..Default::default()
      },
    );
    for (rect, active) in [
      (pos.left, self.active == ActiveList::Disabled),
      (pos.right, self.active == ActiveList::Enabled),
    ] {
      render.draw_host_border_rect(
        canvas,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        &BorderStyle::Line,
        Some(if active {
          ACTIVE_BORDER.clone()
        } else {
          TextColor::Terminal(TerminalColor::BrightWhite)
        }),
        None,
        None,
        None,
      );
    }
    self.draw_panel_title(
      render,
      canvas,
      pos.left,
      i18n.get_runtime_text("screensaver_list", "screensaver_list.left.title"),
    );
    self.draw_panel_title(
      render,
      canvas,
      pos.right,
      i18n.get_runtime_text("screensaver_list", "screensaver_list.right.title"),
    );
    let order = i18n.get_runtime_text(
      "screensaver_list",
      if self.ascending {
        "screensaver_list.order.ascending"
      } else {
        "screensaver_list.order.descending"
      },
    );
    let sort = i18n.get_runtime_text("screensaver_list", self.sort_field.key());
    let label_w = UnicodeWidthStr::width(format!("[{}]{}", order, sort).as_str()) as u16;
    let line_w = pos.left.width.saturating_sub(label_w + 2);
    let line_color = if self.active == ActiveList::Disabled {
      "bright_green"
    } else {
      "bright_white"
    };
    render.draw_host_text(canvas, &DrawTextParams {
      x: pos.left.x,
      y: pos.left_sort_y,
      text: format!("f%<fg:{line_color}>├[</fg><fg:bright_yellow>{}</fg><fg:{line_color}>]</fg><fg:bright_green>{}</fg><fg:{line_color}>{}┤</fg>", order, sort, "─".repeat(line_w as usize)),
      max_width: Some(pos.left.width),
      ..Default::default()
    });
  }

  fn draw_panel_title(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    rect: Rect,
    title: String,
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

  fn draw_search(
    &mut self,
    text_input: &TextInputService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &ScreensaverListLayout,
  ) {
    text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.left_search,
        placeholder: i18n
          .get_runtime_text("screensaver_list", "screensaver_list.search.placeholder"),
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
  }

  fn draw_lists(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &ScreensaverListLayout,
  ) {
    let left = self.disabled_entries();
    let right = self.enabled_entries();
    for (index, entry) in left.iter().enumerate() {
      self.draw_entry(
        render,
        canvas,
        self.left_scroll,
        entry,
        index,
        None,
        index == self.left_selected,
        false,
        pos.left_list.width.saturating_sub(1),
        i18n,
      );
    }
    for (index, entry) in right.iter().enumerate() {
      self.draw_entry(
        render,
        canvas,
        self.right_scroll,
        entry,
        index,
        Some(index + 1),
        index == self.right_selected,
        self.right_locked,
        pos.right_list.width.saturating_sub(1),
        i18n,
      );
    }
    if left.is_empty() {
      self.draw_empty(
        render,
        canvas,
        layout,
        Rect {
          width: pos.left_list.width.saturating_sub(1),
          ..pos.left_list
        },
        i18n.get_runtime_text("screensaver_list", "screensaver_list.left.no"),
      );
    }
    if right.is_empty() {
      self.draw_empty(
        render,
        canvas,
        layout,
        Rect {
          width: pos.right_list.width.saturating_sub(1),
          ..pos.right_list
        },
        i18n.get_runtime_text("screensaver_list", "screensaver_list.right.no"),
      );
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_entry(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    scroll: ScrollBoxId,
    entry: &PackageListEntry,
    y: usize,
    number: Option<usize>,
    selected: bool,
    locked: bool,
    width: u16,
    i18n: &I18nService,
  ) {
    let show_source = match self.source_mode {
      DisplaySourceMode::All => true,
      DisplaySourceMode::Mod => entry.source == PackageSource::Mod,
      DisplaySourceMode::Official => entry.source == PackageSource::Official,
      DisplaySourceMode::No => false,
    };
    let source_key = match entry.source {
      PackageSource::Official => "screensaver_list.source.official",
      PackageSource::Mod => "screensaver_list.source.mod",
    };
    let source_color = match entry.source {
      PackageSource::Official => "bright_magenta",
      PackageSource::Mod => "bright_yellow",
    };
    let source = i18n.get_runtime_text("screensaver_list", source_key);
    let source_w = if show_source {
      UnicodeWidthStr::width(format!("[{}]", source).as_str()) as u16
    } else {
      0
    };
    let number_text = number.map(|value| value.to_string()).unwrap_or_default();
    let number_w = if number.is_some() { 4 } else { 0 };
    let name_x = number_w + 2;
    let name_w = width.saturating_sub(name_x + source_w);
    if number.is_some() {
      render.draw_text_in_scroll_box(
        canvas,
        scroll,
        &DrawTextParams {
          x: 0,
          y: y as u16,
          text: format!(
            "f%<bg:rgb(85,87,83)>{:>width$}</bg>",
            number_text,
            width = number_w as usize
          ),
          max_width: Some(number_w),
          ..Default::default()
        },
      );
    }
    if selected {
      let color = if locked { "bright_red" } else { "bright_cyan" };
      render.draw_text_in_scroll_box(
        canvas,
        scroll,
        &DrawTextParams {
          x: number_w,
          y: y as u16,
          text: format!("f%<fg:{}>▌</fg>", color),
          ..Default::default()
        },
      );
    }
    render.draw_text_in_scroll_box(
      canvas,
      scroll,
      &DrawTextParams {
        x: name_x,
        y: y as u16,
        text: entry.screensaver_name.clone(),
        params: Some(Self::package_params(entry)),
        max_width: Some(name_w),
        max_height: Some(1),
        overflow_marker: Some("...".to_string()),
        ..Default::default()
      },
    );
    if show_source {
      render.draw_text_in_scroll_box(
        canvas,
        scroll,
        &DrawTextParams {
          x: width.saturating_sub(source_w),
          y: y as u16,
          text: format!("f%[<fg:{}>{}</fg>]", source_color, source),
          max_width: Some(source_w),
          ..Default::default()
        },
      );
    }
  }

  fn draw_empty(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    rect: Rect,
    text: String,
  ) {
    let width = layout.get_text_width(&text, None).min(rect.width);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x.saturating_add(rect.width.saturating_sub(width) / 2),
        y: rect.y.saturating_add(rect.height.saturating_sub(1) / 2),
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", text),
        max_width: Some(rect.width),
        ..Default::default()
      },
    );
  }

  fn draw_hints(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _i18n: &I18nService,
    _text_input: &TextInputService,
    pos: &ScreensaverListLayout,
  ) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "screensaver_list.");
    for (index, line) in pos.hint_lines.iter().enumerate() {
      let visible = RichTextService::new().visible_text(line, Some(&params));
      let width = UnicodeWidthStr::width(visible.as_str()) as u16;
      let viewport = layout.developer_viewport_rect();
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: viewport
            .x
            .saturating_add(viewport.width.saturating_sub(width) / 2),
          y: pos.hint_y.saturating_add(index as u16),
          text: format!("f%<fg:rgb(85,87,83)>{}</fg>", line),
          params: Some(params.clone()),
          max_width: Some(viewport.width),
          ..Default::default()
        },
      );
    }
  }

  fn hint_lines(
    &self,
    i18n: &I18nService,
    text_input: &TextInputService,
    width: u16,
  ) -> Vec<String> {
    let keys: Vec<&str> = if text_input.is_focused(&self.objects, self.search_input) {
      vec!["screensaver_list.action.search.back"]
    } else {
      match (self.active, self.right_locked) {
        (ActiveList::Disabled, _) => vec![
          "screensaver_list.action.scroll",
          "screensaver_list.action.select",
          "screensaver_list.action.confirm.enable",
          "screensaver_list.action.list.order",
          "screensaver_list.action.list.sort",
          "screensaver_list.action.list.search",
          "screensaver_list.action.switch",
          "screensaver_list.action.back",
        ],
        (ActiveList::Enabled, false) => vec![
          "screensaver_list.action.scroll",
          "screensaver_list.action.select",
          "screensaver_list.action.confirm.disable",
          "screensaver_list.action.lock",
          "screensaver_list.action.switch",
          "screensaver_list.action.back",
        ],
        (ActiveList::Enabled, true) => vec![
          "screensaver_list.action.scroll",
          "screensaver_list.action.move",
          "screensaver_list.action.confirm.disable",
          "screensaver_list.action.unlock",
          "screensaver_list.action.switch",
          "screensaver_list.action.back",
        ],
      }
    };
    let params = RichTextParams::from_action_map(&Self::action_map(), "screensaver_list.");
    let rich = RichTextService::new();
    let mut lines = vec![String::new()];
    let mut line_width = 0usize;
    for key in keys {
      let item = i18n.get_runtime_text("screensaver_list", key);
      let item_width = UnicodeWidthStr::width(rich.visible_text(&item, Some(&params)).as_str());
      let gap = usize::from(!lines.last().is_some_and(String::is_empty)) * 2;
      if line_width > 0 && line_width + gap + item_width > width as usize {
        lines.push(String::new());
        line_width = 0;
      }
      if !lines.last().unwrap().is_empty() {
        lines.last_mut().unwrap().push_str("  ");
        line_width += 2;
      }
      lines.last_mut().unwrap().push_str(&item);
      line_width += item_width;
    }
    lines
  }

  fn register_hit_areas(
    &mut self,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    canvas: &mut CanvasService,
    pos: &ScreensaverListLayout,
  ) {
    let left_len = self.disabled_entries().len();
    let right_len = self.enabled_entries().len();
    resize_hit_areas(&mut self.objects, hit_area, &mut self.left_areas, left_len);
    resize_hit_areas(
      &mut self.objects,
      hit_area,
      &mut self.right_areas,
      right_len,
    );
    hit_area.render_host(
      &mut self.objects,
      self.back_area,
      Rect {
        x: pos.left.x,
        y: pos.left.y,
        width: pos.left.width.saturating_add(pos.right.width),
        height: pos.left.height,
      },
      canvas,
    );
    hit_area.render_host(&mut self.objects, self.left_panel_area, pos.left, canvas);
    hit_area.render_host(&mut self.objects, self.right_panel_area, pos.right, canvas);
    hit_area.render_host(&mut self.objects, self.search_area, pos.left_search, canvas);
    let order_width = 12.min(pos.left.width.saturating_sub(2));
    hit_area.render_host(
      &mut self.objects,
      self.order_area,
      Rect {
        x: pos.left.x.saturating_add(1),
        y: pos.left_sort_y,
        width: order_width,
        height: 1,
      },
      canvas,
    );
    hit_area.render_host(
      &mut self.objects,
      self.sort_area,
      Rect {
        x: pos.left.x.saturating_add(1 + order_width),
        y: pos.left_sort_y,
        width: pos.left.width.saturating_sub(order_width + 2),
        height: 1,
      },
      canvas,
    );
    let left_top = scroll_box
      .scroll_y(&self.objects, self.left_scroll)
      .unwrap_or(0) as usize;
    let right_top = scroll_box
      .scroll_y(&self.objects, self.right_scroll)
      .unwrap_or(0) as usize;
    for (index, id) in self
      .left_areas
      .iter()
      .enumerate()
      .skip(left_top)
      .take(pos.left_list.height as usize)
    {
      hit_area.render_host(
        &mut self.objects,
        *id,
        Rect {
          x: pos.left_list.x,
          y: pos.left_list.y.saturating_add((index - left_top) as u16),
          width: pos.left_list.width.saturating_sub(1),
          height: 1,
        },
        canvas,
      );
    }
    for (index, id) in self
      .right_areas
      .iter()
      .enumerate()
      .skip(right_top)
      .take(pos.right_list.height as usize)
    {
      hit_area.render_host(
        &mut self.objects,
        *id,
        Rect {
          x: pos.right_list.x,
          y: pos.right_list.y.saturating_add((index - right_top) as u16),
          width: pos.right_list.width.saturating_sub(1),
          height: 1,
        },
        canvas,
      );
    }
  }
}

fn move_index(current: usize, len: usize, delta: i32) -> usize {
  if len == 0 {
    return 0;
  }
  (current as i32 + delta).clamp(0, len.saturating_sub(1) as i32) as usize
}

fn resize_hit_areas(
  pool: &mut UiObjectPool,
  service: &HitAreaService,
  areas: &mut Vec<HitAreaId>,
  len: usize,
) {
  while areas.len() > len {
    if let Some(id) = areas.pop() {
      service.remove(pool, id);
    }
  }
  while areas.len() < len {
    areas.push(service.create(pool, HitAreaOptions::default()));
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, path::PathBuf};

  use super::*;
  use crate::host_engine::services::{PackageAsset, PackageType};

  fn entry(id: &str, enabled: bool) -> PackageListEntry {
    PackageListEntry {
      mod_id: id.to_string(),
      source: PackageSource::Mod,
      package_type: PackageType::Screensaver,
      key_actions: HashMap::new(),
      title: id.to_string(),
      game_name: String::new(),
      screensaver_name: id.to_string(),
      game_detail: String::new(),
      description: String::new(),
      author: String::new(),
      version: String::new(),
      icon: PackageAsset::default_icon(),
      icon_path: None,
      banner: PackageAsset::default_banner(),
      path: PathBuf::new(),
      enabled,
      debug: false,
      safe_mode: true,
      mouse_required: false,
      truecolor_required: false,
      high_privilege_required: false,
      score_enabled: false,
      score_empty_text: String::new(),
      min_width: 0,
      min_height: 0,
      screensaver_command: String::new(),
    }
  }

  fn ui() -> ScreensaverListUi {
    ScreensaverListUi::init(
      &HitAreaService::new(),
      &TextInputService::new(),
      &ScrollBoxService::new(),
    )
  }

  #[test]
  fn list_scrollbars_only_show_when_content_overflows() {
    let ui = ui();

    for id in [ui.left_scroll, ui.right_scroll] {
      let options = &ui.objects.scroll_boxes.boxes[&id].options;
      assert_eq!(options.scrollbar.vertical, ScrollbarVisibility::Auto);
      assert_eq!(options.scrollbar.horizontal, ScrollbarVisibility::Never);
    }
  }

  #[test]
  fn disabling_enabled_item_moves_focus_up() {
    let mut ui = ui();
    ui.entries = vec![entry("a", true), entry("b", true), entry("c", true)];
    ui.enabled_order = vec!["a".into(), "b".into(), "c".into()];
    ui.active = ActiveList::Enabled;
    ui.right_selected = 2;

    assert_eq!(
      ui.toggle_enabled(),
      Some(ScreensaverListCommand::SetEnabled {
        id: "c".into(),
        enabled: false,
      })
    );
    assert_eq!(ui.right_selected, 1);
    assert_eq!(ui.active, ActiveList::Enabled);
  }

  #[test]
  fn disabling_last_enabled_item_stays_on_empty_right_list() {
    let mut ui = ui();
    ui.entries = vec![entry("only", true)];
    ui.enabled_order = vec!["only".into()];
    ui.active = ActiveList::Enabled;

    assert!(ui.toggle_enabled().is_some());
    assert!(ui.enabled_entries().is_empty());
    assert_eq!(ui.right_selected, 0);
    assert_eq!(ui.active, ActiveList::Enabled);
  }

  #[test]
  fn empty_lists_can_switch_focus() {
    let mut ui = ui();

    ui.switch_list();
    assert_eq!(ui.active, ActiveList::Enabled);
    ui.switch_list();
    assert_eq!(ui.active, ActiveList::Disabled);
  }

  #[test]
  fn panel_and_search_clicks_switch_focus() {
    let mut ui = ui();
    ui.active = ActiveList::Disabled;

    assert_eq!(
      ui.handle_event(&UiEvent::HitArea(HitAreaEvent::Click {
        id: ui.right_panel_area,
        button: MouseButton::Left,
        x: 0,
        y: 0,
      })),
      None
    );
    assert_eq!(ui.active, ActiveList::Enabled);
    assert_eq!(
      ui.handle_event(&UiEvent::HitArea(HitAreaEvent::Click {
        id: ui.search_area,
        button: MouseButton::Left,
        x: 0,
        y: 0,
      })),
      Some(ScreensaverListCommand::FocusSearch)
    );
  }
}
