use std::{
  fs, io,
  marker::PhantomData,
  path::Path,
  time::{Duration, SystemTime},
};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Overflow,
  Rect, RenderService, RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner,
  ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarLayout, ScrollbarPolicy,
  ScrollbarVisibility, TerminalColor, TextColor, TextInputEvent, TextInputId, TextInputMode,
  TextInputOptions, TextInputRenderParams, TextInputService, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const HINT_COLOR: TextColor = TextColor::Rgb {
  r: 85,
  g: 87,
  b: 83,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaListCommand {
  Back,
  FocusSearch,
  BlurSearch,
  ScrollList(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActivePanel {
  List,
  Info,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortField {
  Name,
  Time,
  Duration,
}

#[derive(Clone, Debug)]
struct MediaEntry {
  name: String,
  modified: SystemTime,
  duration_us: u64,
}

#[derive(Clone, Debug)]
struct MediaListLayout {
  left: Rect,
  right: Rect,
  search: Rect,
  sort_y: u16,
  list: Rect,
  hint_y: u16,
  hint_lines: Vec<String>,
}

pub trait MediaListSpec {
  const NS: &'static str;
  const SUPPORTS_DURATION: bool;
  fn action_map() -> Vec<ActionMapEntry>;
  fn left_hint_keys() -> &'static [&'static str];
  fn right_hint_keys() -> &'static [&'static str];
}

pub struct MediaListUi<S: MediaListSpec> {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  search_input: TextInputId,
  list_scroll: ScrollBoxId,
  list_panel_area: HitAreaId,
  info_panel_area: HitAreaId,
  order_area: HitAreaId,
  sort_area: HitAreaId,
  item_areas: Vec<HitAreaId>,
  entries: Vec<MediaEntry>,
  search: String,
  selected: usize,
  active: ActivePanel,
  ascending: bool,
  sort_field: SortField,
  marker: PhantomData<S>,
}

impl<S: MediaListSpec> UiObjectPoolOwner for MediaListUi<S> {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl<S: MediaListSpec> RuntimeObjectPoolOwner for MediaListUi<S> {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl<S: MediaListSpec> MediaListUi<S> {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let search_input = text_input.create(
      &mut objects,
      TextInputOptions {
        max_chars: Some(128),
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    let list_scroll = scroll_box
      .create(
        &mut objects,
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
      .expect("failed to create media list scroll box");
    Self {
      search_input,
      list_scroll,
      list_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      info_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      order_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      sort_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      item_areas: Vec::new(),
      entries: Vec::new(),
      search: String::new(),
      selected: 0,
      active: ActivePanel::List,
      ascending: true,
      sort_field: SortField::Name,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      marker: PhantomData,
    }
  }

  pub fn reload(&mut self, directory: &Path) -> io::Result<()> {
    let selected = self
      .filtered_entries()
      .get(self.selected)
      .map(|entry| entry.name.clone());
    let mut entries = Vec::new();
    for item in fs::read_dir(directory)? {
      let item = item?;
      let path = item.path();
      if !item.file_type()?.is_file()
        || path.extension().and_then(|value| value.to_str()) != Some("json")
      {
        continue;
      }
      let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
      let metadata = item.metadata()?;
      let duration_us = S::SUPPORTS_DURATION
        .then(|| read_duration_us(&path))
        .flatten()
        .unwrap_or(0);
      entries.push(MediaEntry {
        name,
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        duration_us,
      });
    }
    self.entries = entries;
    self.search.clear();
    self.selected = selected
      .and_then(|name| {
        self
          .filtered_entries()
          .iter()
          .position(|entry| entry.name == name)
      })
      .unwrap_or(0);
    self.active = ActivePanel::List;
    Ok(())
  }

  pub fn reset_search(&mut self, text_input: &mut TextInputService) {
    self.search.clear();
    let _ = text_input.clear(&mut self.objects, self.search_input);
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    S::action_map()
  }

  pub fn focus_search(&mut self, text_input: &mut TextInputService) {
    self.active = ActivePanel::List;
    let _ = text_input.focus(&mut self.objects, self.search_input);
  }

  pub fn blur_search(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn scroll_list(&mut self, service: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    let _ = service.scroll_by(&mut self.objects, self.list_scroll, 0, dy, layout);
  }

  pub fn update(&mut self, _dt: Duration) {}

  pub fn prepare_surfaces(
    &mut self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) {
    let pos = self.compute_layout(layout, i18n, text_input);
    self.prepare_scroll_box(scroll_box, layout, &pos);
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<MediaListCommand> {
    match event {
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        self.active = ActivePanel::List;
        return Some(MediaListCommand::FocusSearch);
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.search_input => {
        return Some(MediaListCommand::BlurSearch);
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        return Some(MediaListCommand::BlurSearch);
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search = value.clone();
        self.selected = 0;
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.handle_pointer_target(*id, false);
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.handle_pointer_target(*id, true);
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => return Some(MediaListCommand::Back),
      _ => {}
    }
    let UiEvent::Action(event) = event else {
      return None;
    };
    if event.state != KeyState::Pressed {
      return None;
    }
    let prefix = S::NS;
    match event.action.strip_prefix(prefix).unwrap_or_default() {
      ".back" => Some(MediaListCommand::Back),
      ".switch" => {
        self.active = match self.active {
          ActivePanel::List => ActivePanel::Info,
          ActivePanel::Info => ActivePanel::List,
        };
        None
      }
      ".search" if self.active == ActivePanel::List => Some(MediaListCommand::FocusSearch),
      ".focus_up" if self.active == ActivePanel::List => {
        self.move_selection(-1);
        None
      }
      ".focus_down" if self.active == ActivePanel::List => {
        self.move_selection(1);
        None
      }
      ".scroll_up" if self.active == ActivePanel::List => Some(MediaListCommand::ScrollList(-3)),
      ".scroll_down" if self.active == ActivePanel::List => Some(MediaListCommand::ScrollList(3)),
      ".order" if self.active == ActivePanel::List => {
        self.ascending = !self.ascending;
        None
      }
      ".sort" if self.active == ActivePanel::List => {
        self.sort_field = match self.sort_field {
          SortField::Name => SortField::Time,
          SortField::Time if S::SUPPORTS_DURATION => SortField::Duration,
          _ => SortField::Name,
        };
        None
      }
      _ => None,
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
  ) -> Option<(u16, u16)> {
    let pos = self.compute_layout(layout, i18n, text_input);
    self.draw_frames(render, canvas, i18n, &pos);

    hit_area.render_host(&mut self.objects, self.list_panel_area, pos.left, canvas);
    hit_area.render_host(&mut self.objects, self.info_panel_area, pos.right, canvas);

    let cursor = text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search,
        placeholder: i18n.get_runtime_text(S::NS, &format!("{}.list.search.placeholder", S::NS)),
        placeholder_fg: Some(HINT_COLOR.clone()),
        ..Default::default()
      },
      canvas,
    );
    self.draw_sort_bar(render, canvas, i18n, hit_area, &pos);
    self.draw_entries(render, canvas, layout, i18n, hit_area, scroll_box, &pos);
    self.draw_hints(render, canvas, layout, i18n, text_input, &pos);
    cursor
  }

  fn compute_layout(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> MediaListLayout {
    let viewport = layout.developer_viewport_rect();
    let hint_lines = self.hint_lines(i18n, text_input, viewport.width);
    let hint_h = hint_lines.len().max(1) as u16;
    let content_h = viewport.height.saturating_sub(hint_h);
    let left_w = viewport.width / 3;
    let left = Rect {
      x: viewport.x,
      y: viewport.y,
      width: left_w,
      height: content_h,
    };
    let right = Rect {
      x: viewport.x.saturating_add(left_w),
      y: viewport.y,
      width: viewport.width.saturating_sub(left_w),
      height: content_h,
    };
    MediaListLayout {
      search: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(1),
        width: left.width.saturating_sub(2),
        height: 1,
      },
      sort_y: left.y.saturating_add(2),
      list: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(3),
        width: left.width.saturating_sub(2),
        height: left.height.saturating_sub(4),
      },
      left,
      right,
      hint_y: viewport.y.saturating_add(content_h),
      hint_lines,
    }
  }

  fn prepare_scroll_box(
    &mut self,
    service: &ScrollBoxService,
    layout: &LayoutService,
    pos: &MediaListLayout,
  ) {
    let viewport = layout.developer_viewport_rect();
    let rect = Rect {
      x: pos.list.x.saturating_sub(viewport.x),
      y: pos.list.y.saturating_sub(viewport.y),
      width: pos.list.width,
      height: pos.list.height,
    };
    let len = self.filtered_entries().len() as u16;
    let _ = service.set_rect(&mut self.objects, self.list_scroll, rect, layout);
    let _ = service.set_content_size(
      &mut self.objects,
      self.list_scroll,
      rect.width.saturating_sub(1).max(1),
      len.max(rect.height).max(1),
      layout,
    );
    if self.active == ActivePanel::List {
      self.ensure_selection_visible(service, layout);
    }
  }

  fn ensure_selection_visible(&mut self, service: &ScrollBoxService, layout: &LayoutService) {
    let height = service
      .visible_content_height(&self.objects, self.list_scroll, layout)
      .unwrap_or(0) as usize;
    if height == 0 {
      return;
    }
    let top = service
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(0) as usize;
    let target = if self.selected < top {
      Some(self.selected)
    } else if self.selected >= top.saturating_add(height) {
      Some(self.selected + 1 - height)
    } else {
      None
    };
    if let Some(y) = target {
      let _ = service.scroll_to(&mut self.objects, self.list_scroll, 0, y as u16, layout);
    }
  }

  fn draw_frames(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &MediaListLayout,
  ) {
    for (rect, active) in [
      (pos.left, self.active == ActivePanel::List),
      (pos.right, self.active == ActivePanel::Info),
    ] {
      render.draw_host_border_rect(
        canvas,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        &BorderStyle::Line,
        Some(if active {
          TextColor::Terminal(TerminalColor::Green)
        } else {
          TextColor::Terminal(TerminalColor::BrightWhite)
        }),
        None,
        None,
        None,
      );
    }
    for (rect, key) in [
      (pos.left, format!("{}.list", S::NS)),
      (pos.right, format!("{}.info", S::NS)),
    ] {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x.saturating_add(1),
          y: rect.y,
          text: format!(
            "f%<fg:bright_magenta><b>{}</b></fg>",
            i18n.get_runtime_text(S::NS, &key)
          ),
          max_width: Some(rect.width.saturating_sub(2)),
          ..Default::default()
        },
      );
    }
  }

  fn draw_sort_bar(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    pos: &MediaListLayout,
  ) {
    if pos.left.width < 2 {
      return;
    }
    let order = i18n.get_runtime_text(
      S::NS,
      &format!(
        "{}.list.order.{}",
        S::NS,
        if self.ascending {
          "ascending"
        } else {
          "descending"
        }
      ),
    );
    let sort = i18n.get_runtime_text(
      S::NS,
      &format!(
        "{}.list.sort.{}",
        S::NS,
        match self.sort_field {
          SortField::Name => "name",
          SortField::Time => "time",
          SortField::Duration => "duration",
        }
      ),
    );
    let label_width = format!("[{order}]{sort}")
      .width()
      .min(pos.left.width.saturating_sub(2) as usize);
    let line = "─".repeat(pos.left.width.saturating_sub(2 + label_width as u16) as usize);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.left.x,
        y: pos.sort_y,
        text: format!(
          "f%<fg:bright_black>╟[</fg><fg:bright_yellow>{order}</fg><fg:bright_black>]</fg><fg:bright_green>{sort}</fg><fg:bright_black>{line}╢</fg>"
        ),
        max_width: Some(pos.left.width),
        max_height: Some(1),
        ..Default::default()
      },
    );
    let order_width = format!("[{order}]").width() as u16;
    hit_area.render_host(
      &mut self.objects,
      self.order_area,
      Rect {
        x: pos.left.x.saturating_add(1),
        y: pos.sort_y,
        width: order_width.min(pos.left.width.saturating_sub(2)),
        height: 1,
      },
      canvas,
    );
    hit_area.render_host(
      &mut self.objects,
      self.sort_area,
      Rect {
        x: pos.left.x.saturating_add(1).saturating_add(order_width),
        y: pos.sort_y,
        width: sort.width().min(u16::MAX as usize) as u16,
        height: 1,
      },
      canvas,
    );
  }

  fn draw_entries(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    pos: &MediaListLayout,
  ) {
    let entries: Vec<_> = self.filtered_entries().into_iter().cloned().collect();
    self.resize_item_areas(hit_area, entries.len());
    if entries.is_empty() {
      let text = i18n.get_runtime_text(S::NS, &format!("{}.no.image", S::NS));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.list.x
            + pos
              .list
              .width
              .saturating_sub(layout.get_text_width(&text, None))
              / 2,
          y: pos.list.y + pos.list.height / 2,
          text: format!("f%<fg:rgb(85,87,83)>{text}</fg>"),
          ..Default::default()
        },
      );
      return;
    }
    for (index, entry) in entries.iter().enumerate() {
      if self.active == ActivePanel::List && index == self.selected {
        render.draw_text_in_scroll_box(
          canvas,
          self.list_scroll,
          &DrawTextParams {
            x: 0,
            y: index as u16,
            text: "f%<fg:bright_cyan>▌</fg>".to_string(),
            ..Default::default()
          },
        );
      }
      render.draw_text_in_scroll_box(
        canvas,
        self.list_scroll,
        &DrawTextParams {
          x: 2,
          y: index as u16,
          text: entry.name.clone(),
          max_width: Some(pos.list.width.saturating_sub(4)),
          max_height: Some(1),
          overflow_marker: Some("...".to_string()),
          ..Default::default()
        },
      );
    }
    let top = scroll_box
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(0) as usize;
    let height = scroll_box
      .visible_content_height(&self.objects, self.list_scroll, layout)
      .unwrap_or(0) as usize;
    for index in top..entries.len().min(top.saturating_add(height)) {
      hit_area.render_host(
        &mut self.objects,
        self.item_areas[index],
        Rect {
          x: pos.list.x,
          y: pos.list.y.saturating_add((index - top) as u16),
          width: pos.list.width.saturating_sub(1),
          height: 1,
        },
        canvas,
      );
    }
  }

  fn draw_hints(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    _text_input: &TextInputService,
    pos: &MediaListLayout,
  ) {
    let params = RichTextParams::from_action_map(&S::action_map(), &format!("{}.", S::NS));
    for (index, line) in pos.hint_lines.iter().enumerate() {
      let width = layout.get_text_width(line, Some(&params));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.left.x + (pos.left.width + pos.right.width).saturating_sub(width) / 2,
          y: pos.hint_y.saturating_add(index as u16),
          text: format!("f%<fg:rgb(85,87,83)>{line}</fg>"),
          params: Some(params.clone()),
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
    let keys = if text_input.is_focused(&self.objects, self.search_input) {
      &["action.cancel", "action.confirm"][..]
    } else if self.active == ActivePanel::List {
      S::left_hint_keys()
    } else {
      S::right_hint_keys()
    };
    let params = RichTextParams::from_action_map(&S::action_map(), &format!("{}.", S::NS));
    let rich = RichTextService::new();
    let mut lines = vec![String::new()];
    let mut current_width = 0usize;
    for suffix in keys {
      let key = format!("{}.{}", S::NS, suffix);
      let item = i18n.get_runtime_text(S::NS, &key);
      let item_width = rich.visible_text(&item, Some(&params)).width();
      let gap = usize::from(current_width > 0) * 2;
      if current_width > 0 && current_width + gap + item_width > width as usize {
        lines.push(String::new());
        current_width = 0;
      }
      if current_width > 0 {
        lines.last_mut().unwrap().push_str("  ");
        current_width += 2;
      }
      lines.last_mut().unwrap().push_str(&item);
      current_width += item_width;
    }
    lines
  }

  fn filtered_entries(&self) -> Vec<&MediaEntry> {
    let query = self.search.to_lowercase();
    let mut entries: Vec<_> = self
      .entries
      .iter()
      .filter(|entry| query.is_empty() || entry.name.to_lowercase().contains(&query))
      .collect();
    entries.sort_by(|a, b| {
      let order = match self.sort_field {
        SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortField::Time => a.modified.cmp(&b.modified),
        SortField::Duration => a.duration_us.cmp(&b.duration_us),
      };
      if self.ascending {
        order
      } else {
        order.reverse()
      }
    });
    entries
  }

  fn move_selection(&mut self, delta: isize) {
    let len = self.filtered_entries().len();
    if len == 0 {
      self.selected = 0;
      return;
    }
    self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
  }

  fn handle_pointer_target(&mut self, id: HitAreaId, clicked: bool) {
    if id == self.list_panel_area {
      self.active = ActivePanel::List;
    } else if id == self.info_panel_area {
      self.active = ActivePanel::Info;
    } else if id == self.order_area && clicked {
      self.ascending = !self.ascending;
    } else if id == self.sort_area && clicked {
      self.sort_field = match self.sort_field {
        SortField::Name => SortField::Time,
        SortField::Time if S::SUPPORTS_DURATION => SortField::Duration,
        _ => SortField::Name,
      };
    } else if let Some(index) = self.item_areas.iter().position(|area| *area == id) {
      self.active = ActivePanel::List;
      self.selected = index;
    }
  }

  fn resize_item_areas(&mut self, service: &HitAreaService, len: usize) {
    while self.item_areas.len() < len {
      self
        .item_areas
        .push(service.create(&mut self.objects, HitAreaOptions::default()));
    }
    while self.item_areas.len() > len {
      if let Some(id) = self.item_areas.pop() {
        let _ = service.remove(&mut self.objects, id);
      }
    }
  }
}

fn action(name: &str, key: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: name.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}

pub fn actions(entries: &[(&str, &str)]) -> Vec<ActionMapEntry> {
  entries
    .iter()
    .map(|(name, key)| action(name, key))
    .collect()
}

fn read_duration_us(path: &Path) -> Option<u64> {
  let value: serde_json::Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
  value.get("duration_us")?.get("active")?.as_u64()
}
