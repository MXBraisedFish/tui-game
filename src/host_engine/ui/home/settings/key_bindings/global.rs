//! Global page state, user commands, and terminal-cell presentation.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, InputService, Key, KeyBindingsProfile, KeyEventKind, KeyState,
  LayoutService, MouseButton, Rect, RenderService, RichTextParams, RichTextService,
  RuntimeObjectPool, RuntimeObjectPoolOwner, TerminalColor, TextColor, UiEvent, UiObjectPool,
  UiObjectPoolOwner, format_key_display, key_token,
};

const ROW_COUNT: usize = 7;
const CAPTURE_DELAY: Duration = Duration::from_millis(80);
const GRAY: TextColor = TextColor::Rgb {
  r: 85,
  g: 87,
  b: 83,
};
const LIGHT_GRAY: TextColor = TextColor::Rgb {
  r: 170,
  g: 172,
  b: 168,
};
const CYAN: TextColor = TextColor::Terminal(TerminalColor::BrightCyan);
const MAGENTA: TextColor = TextColor::Terminal(TerminalColor::BrightMagenta);
const YELLOW: TextColor = TextColor::Terminal(TerminalColor::BrightYellow);
const BLUE: TextColor = TextColor::Terminal(TerminalColor::Blue);
const WHITE: TextColor = TextColor::Terminal(TerminalColor::BrightWhite);
const BLACK: TextColor = TextColor::Terminal(TerminalColor::Black);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditMode {
  Edit,
  Delete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BindingRow {
  action: String,
  description: String,
  keys: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct CaptureState {
  slot: usize,
  keys: Vec<Key>,
  elapsed: Option<Duration>,
}

/// An application request produced by global key bindings interactions.
#[derive(Clone, Debug)]
pub enum GlobalKeyBindingsCommand {
  /// A request to back.
  Back(KeyBindingsProfile),
  /// The capture started setting for global key bindings command.
  CaptureStarted,
  /// Refuse to save malformed bindings within a single action.
  InvalidBindings,
}

/// The state and owned widgets of the global key bindings view.
pub struct GlobalKeyBindingsUi {
  selected: usize,
  mode: EditMode,
  show_color_doc: bool,
  rows: Vec<BindingRow>,
  profile: KeyBindingsProfile,
  capture: Option<CaptureState>,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  cell_areas: [[HitAreaId; 3]; ROW_COUNT],
}

impl UiObjectPoolOwner for GlobalKeyBindingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for GlobalKeyBindingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl GlobalKeyBindingsUi {
  /// Create the global view and allocate its owned UI objects.
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected: 0,
      mode: EditMode::Edit,
      show_color_doc: false,
      rows: Vec::new(),
      profile: KeyBindingsProfile::default(),
      capture: None,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      cell_areas: std::array::from_fn(|_| {
        std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default()))
      }),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  /// Load the global shortcut profile and reset the editing selection.
  pub fn load(&mut self, entries: Vec<ActionMapEntry>, profile: KeyBindingsProfile) {
    self.selected = 0;
    self.mode = EditMode::Edit;
    self.show_color_doc = false;
    self.capture = None;
    self.rows = entries
      .into_iter()
      .map(|entry| BindingRow {
        action: entry.action,
        description: entry.description,
        keys: entry.keys,
      })
      .collect();
    self.profile = profile;
  }

  /// Return the shortcuts currently enabled by the global view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action(
        "key_bindings_global.focus_up",
        "up",
        "Focus previous action",
      ),
      action(
        "key_bindings_global.focus_down",
        "down",
        "Focus next action",
      ),
      action(
        "key_bindings_global.key_edit_del_1",
        "1",
        "Edit or delete first key",
      ),
      action(
        "key_bindings_global.key_edit_del_2",
        "2",
        "Edit or delete second key",
      ),
      action("key_bindings_global.key_switch", "b", "Switch edit mode"),
      action(
        "key_bindings_global.reset.only",
        "r",
        "Reset selected action",
      ),
      action("key_bindings_global.reset.all", "t", "Reset all actions"),
      action("key_bindings_global.back", "esc", "Save and go back"),
      action("key_bindings_global.color_doc", "f", "Toggle color guide"),
    ]
  }

  /// Report whether this global key bindings ui is capturing.
  pub fn is_capturing(&self) -> bool {
    self.capture.is_some()
  }

  /// Interpret a global UI event and return the requested application command.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<GlobalKeyBindingsCommand> {
    if self.capture.is_some() {
      return None;
    }
    if self.show_color_doc {
      if let UiEvent::Action(action) = event
        && action.state == KeyState::Pressed
        && action.action == "key_bindings_global.color_doc"
      {
        self.show_color_doc = false;
      }
      return None;
    }
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        let (row, _) = self.area_position(*id)?;
        self.selected = row;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        let (row, column) = self.area_position(*id)?;
        self.selected = row;
        (column > 0)
          .then(|| self.activate_slot(column - 1))
          .flatten()
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => self.try_back(),
      UiEvent::Action(action) if action.state == KeyState::Pressed => {
        match action.action.as_str() {
          "key_bindings_global.focus_up" => {
            self.selected = (self.selected + self.rows.len().saturating_sub(1)) % self.rows.len();
            None
          }
          "key_bindings_global.focus_down" => {
            self.selected = (self.selected + 1) % self.rows.len();
            None
          }
          "key_bindings_global.key_edit_del_1" => self.activate_slot(0),
          "key_bindings_global.key_edit_del_2" => self.activate_slot(1),
          "key_bindings_global.key_switch" => {
            self.mode = match self.mode {
              EditMode::Edit => EditMode::Delete,
              EditMode::Delete => EditMode::Edit,
            };
            None
          }
          "key_bindings_global.reset.only" => {
            self.reset_selected();
            None
          }
          "key_bindings_global.reset.all" => {
            self.reset_all();
            None
          }
          "key_bindings_global.color_doc" => {
            self.show_color_doc = true;
            None
          }
          "key_bindings_global.back" => self.try_back(),
          _ => None,
        }
      }
      _ => None,
    }
  }

  fn area_position(&self, id: HitAreaId) -> Option<(usize, usize)> {
    self.cell_areas.iter().enumerate().find_map(|(row, cells)| {
      cells
        .iter()
        .position(|area| *area == id)
        .map(|column| (row, column))
    })
  }

  fn activate_slot(&mut self, requested_slot: usize) -> Option<GlobalKeyBindingsCommand> {
    let row = self.rows.get_mut(self.selected)?;
    match self.mode {
      EditMode::Delete => {
        if requested_slot < row.keys.len() {
          row.keys.remove(requested_slot);
          self.sync_selected_to_profile();
        }
        None
      }
      EditMode::Edit => {
        let slot = if row.keys.is_empty() {
          0
        } else {
          requested_slot
        };
        self.capture = Some(CaptureState {
          slot,
          keys: Vec::new(),
          elapsed: None,
        });
        Some(GlobalKeyBindingsCommand::CaptureStarted)
      }
    }
  }

  fn try_back(&self) -> Option<GlobalKeyBindingsCommand> {
    if crate::host_engine::services::translate_action_map(
      &self
        .rows
        .iter()
        .map(|row| ActionMapEntry {
          action: row.action.clone(),
          description: row.description.clone(),
          keys: row.keys.clone(),
          priority: 0,
        })
        .collect::<Vec<_>>(),
    )
    .is_err()
    {
      return Some(GlobalKeyBindingsCommand::InvalidBindings);
    }
    Some(GlobalKeyBindingsCommand::Back(self.profile.clone()))
  }

  /// Consume raw key transitions for the active capture or confirmation interaction.
  pub fn handle_raw_key_events(&mut self, input: &mut InputService, dt: Duration) -> bool {
    let Some(capture) = &mut self.capture else {
      return false;
    };
    for event in input.take_raw_key_events() {
      if event.kind == KeyEventKind::Press && !capture.keys.contains(&event.key) {
        capture.keys.push(event.key);
        if capture.elapsed.is_none() {
          capture.elapsed = Some(Duration::ZERO);
        }
      }
    }

    let Some(elapsed) = &mut capture.elapsed else {
      return false;
    };
    *elapsed = elapsed.saturating_add(dt);
    if *elapsed < CAPTURE_DELAY {
      return false;
    }

    capture.keys.sort();
    let pattern = capture
      .keys
      .iter()
      .copied()
      .map(key_token)
      .collect::<Vec<_>>();
    let slot = capture.slot;
    if !pattern.is_empty()
      && let Some(row) = self.rows.get_mut(self.selected)
    {
      if slot < row.keys.len() {
        row.keys[slot] = pattern;
      } else {
        row.keys.push(pattern);
      }
    }
    self.capture = None;
    self.sync_selected_to_profile();
    true
  }

  fn sync_selected_to_profile(&mut self) {
    let Some(row) = self.rows.get(self.selected) else {
      return;
    };
    self
      .profile
      .user
      .global
      .insert(row.action.clone(), row.keys.clone());
  }

  fn reset_selected(&mut self) {
    let Some(row) = self.rows.get_mut(self.selected) else {
      return;
    };
    row.keys = self
      .profile
      .default
      .global
      .get(&row.action)
      .cloned()
      .unwrap_or_default();
    self.sync_selected_to_profile();
  }

  fn reset_all(&mut self) {
    for row in &mut self.rows {
      row.keys = self
        .profile
        .default
        .global
        .get(&row.action)
        .cloned()
        .unwrap_or_default();
      self
        .profile
        .user
        .global
        .insert(row.action.clone(), row.keys.clone());
    }
  }

  fn global_conflict_actions(&self) -> HashSet<String> {
    let mut owners: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    for row in &self.rows {
      for pattern in &row.keys {
        owners
          .entry(normalized_pattern(pattern))
          .or_default()
          .push(row.action.clone());
      }
    }
    owners
      .into_values()
      .filter(|actions| actions.len() > 1)
      .flatten()
      .collect()
  }

  fn game_patterns(&self) -> HashSet<Vec<String>> {
    self
      .profile
      .user
      .games
      .values()
      .flat_map(BTreeMap::values)
      .flatten()
      .map(|pattern| normalized_pattern(pattern))
      .collect()
  }

  /// Draw the global view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `hit_area` - The hit area.
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    let title = i18n.get_runtime_text("key_bindings_global", "key_bindings_global.title");
    draw_centered(
      render,
      canvas,
      layout,
      viewport.x,
      viewport.width,
      viewport.y.saturating_add(1),
      &format!("f%<fg:bright_magenta>{title}</fg>"),
      None,
      None,
      None,
      true,
    );

    let hint_lines = self.action_hint_lines(i18n, viewport.width);
    let hint_height = hint_lines.len().max(1) as u16;
    let table_width = self.table_width(layout, viewport.width);
    let table_height = ROW_COUNT as u16 + 4;
    let table_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, table_width, 0));
    let hint_y = viewport
      .y
      .saturating_add(viewport.height.saturating_sub(hint_height));
    let available_top = viewport.y.saturating_add(3);
    let available_height = hint_y.saturating_sub(available_top);
    let table_y = available_top.saturating_add(available_height.saturating_sub(table_height) / 2);
    let table = Rect {
      x: table_x,
      y: table_y,
      width: table_width,
      height: table_height,
    };
    self.draw_table(render, canvas, layout, i18n, hit_area, table);
    self.draw_hint(render, canvas, layout, viewport, hint_y, &hint_lines);
    if self.show_color_doc {
      self.draw_color_doc(render, canvas, layout, i18n, table);
    }
  }

  fn table_width(&self, layout: &LayoutService, viewport_width: u16) -> u16 {
    let action_content = self
      .rows
      .iter()
      .map(|row| layout.get_text_width(&row.description, None))
      .max()
      .unwrap_or_default()
      .saturating_add(6);
    let key_content = self
      .rows
      .iter()
      .flat_map(|row| row.keys.iter())
      .map(|pattern| {
        layout.get_text_width(&format_key_display(std::slice::from_ref(pattern)), None)
      })
      .max()
      .unwrap_or_default()
      .saturating_add(6);
    let proportional_action = ceil_percent_width(action_content, 40);
    let proportional_key = ceil_percent_width(key_content, 30);
    let desired_inner = proportional_action.max(proportional_key).max(60);
    desired_inner
      .saturating_add(2)
      .min(viewport_width.saturating_sub(12).max(40))
      .min(viewport_width)
  }

  fn draw_table(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    table: Rect,
  ) {
    render.draw_host_border_rect(
      canvas,
      table.x,
      table.y,
      table.width,
      table.height,
      &crate::host_engine::services::BorderStyle::Line,
      Some(WHITE),
      None,
      Some(BLACK),
      None,
    );
    if table.width > 1 {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: table.x,
          y: table.y.saturating_add(2),
          text: format!("├{}┤", "─".repeat(table.width.saturating_sub(2) as usize)),
          fg: Some(WHITE),
          ..Default::default()
        },
      );
    }

    let inner_width = table.width.saturating_sub(2);
    let action_width = inner_width.saturating_mul(40) / 100;
    let key1_width = inner_width.saturating_mul(30) / 100;
    let key2_width = inner_width.saturating_sub(action_width + key1_width);
    let columns = [
      (table.x.saturating_add(1), action_width),
      (table.x.saturating_add(1 + action_width), key1_width),
      (
        table.x.saturating_add(1 + action_width + key1_width),
        key2_width,
      ),
    ];
    let headers = [
      i18n.get_runtime_text("key_bindings_global", "key_bindings_global.action"),
      i18n.get_runtime_text("key_bindings_global", "key_bindings_global.key1"),
      i18n.get_runtime_text("key_bindings_global", "key_bindings_global.key2"),
    ];
    for ((x, width), header) in columns.iter().copied().zip(headers) {
      draw_centered(
        render,
        canvas,
        layout,
        x,
        width,
        table.y + 1,
        &header,
        None,
        None,
        None,
        true,
      );
    }

    let shared_actions = self.global_conflict_actions();
    let game_patterns = self.game_patterns();
    for index in 0..self.rows.len().min(ROW_COUNT) {
      let y = table.y.saturating_add(3 + index as u16);
      let row = &self.rows[index];
      let conflict_color = if shared_actions.contains(row.action.as_str()) {
        Some(YELLOW.clone())
      } else if row
        .keys
        .iter()
        .any(|pattern| game_patterns.contains(&normalized_pattern(pattern)))
      {
        Some(YELLOW.clone())
      } else {
        None
      };
      if let Some(color) = conflict_color {
        for x in [
          table.x.saturating_add(2),
          table.x.saturating_add(3),
          table.x.saturating_add(table.width.saturating_sub(3)),
          table.x.saturating_add(table.width.saturating_sub(4)),
        ] {
          render.draw_host_filled_rect(
            canvas,
            x,
            y,
            1,
            1,
            Some(" ".to_string()),
            None,
            Some(color.clone()),
          );
        }
      }
      if index == self.selected {
        let color = match self.mode {
          EditMode::Edit => CYAN.clone(),
          EditMode::Delete => MAGENTA.clone(),
        };
        for x in [
          table.x.saturating_add(1),
          table.x.saturating_add(table.width.saturating_sub(2)),
        ] {
          render.draw_host_filled_rect(
            canvas,
            x,
            y,
            1,
            1,
            Some(" ".to_string()),
            None,
            Some(color.clone()),
          );
        }
      }

      draw_left(render, canvas, columns[0], y, &row.description, layout);
      for slot in 0..2 {
        let text = row.keys.get(slot).map_or_else(String::new, |pattern| {
          format_key_display(std::slice::from_ref(pattern))
        });
        let capturing = index == self.selected
          && self
            .capture
            .as_ref()
            .is_some_and(|capture| capture.slot == slot);
        if capturing {
          render.draw_host_filled_rect(
            canvas,
            columns[slot + 1].0.saturating_add(3),
            y,
            columns[slot + 1].1.saturating_sub(6),
            1,
            Some(" ".to_string()),
            None,
            Some(BLUE.clone()),
          );
        }
        draw_centered(
          render,
          canvas,
          layout,
          columns[slot + 1].0.saturating_add(3),
          columns[slot + 1].1.saturating_sub(6),
          y,
          &text,
          Some(WHITE.clone()),
          capturing.then(|| BLUE.clone()),
          None,
          false,
        );
      }
      for (column, (x, width)) in columns.iter().copied().enumerate() {
        hit_area.render_host(
          &mut self.objects,
          self.cell_areas[index][column],
          Rect {
            x,
            y,
            width,
            height: 1,
          },
          canvas,
        );
      }
    }
  }

  fn draw_hint(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    viewport: Rect,
    y: u16,
    lines: &[String],
  ) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_global.");
    for (index, line) in lines.iter().enumerate() {
      let text = format!("f%<fg:rgb(85,87,83)>{line}</fg>");
      let width = layout.get_text_width(&text, Some(&params));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: viewport
            .x
            .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0)),
          y: y.saturating_add(index as u16),
          text,
          params: Some(params.clone()),
          max_width: Some(viewport.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
  }

  fn action_hint_lines(&self, i18n: &I18nService, max_width: u16) -> Vec<String> {
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_global.");
    let keys = if self.capture.is_some() {
      vec!["key_bindings_global.action.any"]
    } else {
      vec![
        "key_bindings_global.action.select",
        match self.mode {
          EditMode::Edit => "key_bindings_global.action.key.edit",
          EditMode::Delete => "key_bindings_global.action.key.del",
        },
        "key_bindings_global.action.switch",
        "key_bindings_global.action.reset.only",
        "key_bindings_global.action.reset.all",
        "key_bindings_global.action.back",
        if self.show_color_doc {
          "key_bindings_global.action.color_doc.out"
        } else {
          "key_bindings_global.action.color_doc.in"
        },
      ]
    };
    wrap_hint_items(
      keys
        .into_iter()
        .map(|key| i18n.get_runtime_text("key_bindings_global", key)),
      &params,
      max_width,
    )
  }

  fn draw_color_doc(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    table: Rect,
  ) {
    let keys = [
      "key_bindings_global.color_doc.yellow",
      "key_bindings_global.color_doc.shared",
      "key_bindings_global.color_doc.gray",
      "key_bindings_global.color_doc.composite",
      "key_bindings_global.color_doc.priority",
      "key_bindings_global.action.color_doc.out",
    ];
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_global.");
    let texts = keys
      .iter()
      .map(|key| format!("f%{}", i18n.get_runtime_text("key_bindings_global", key)))
      .collect::<Vec<_>>();
    let content_width = texts
      .iter()
      .map(|text| layout.get_text_width(text, Some(&params)))
      .max()
      .unwrap_or_default();
    let width = content_width.saturating_add(8).min(table.width);
    let height = 9;
    let x = table
      .x
      .saturating_add((table.width.saturating_sub(width)) / 2);
    let y = table
      .y
      .saturating_add((table.height.saturating_sub(height)) / 2);
    render.draw_host_border_rect(
      canvas,
      x,
      y,
      width,
      height,
      &crate::host_engine::services::BorderStyle::Line,
      Some(WHITE),
      None,
      Some(BLACK),
      None,
    );
    for (index, text) in texts.iter().enumerate() {
      let line_y = y.saturating_add(1 + index as u16 + u16::from(index == 5));
      match index {
        0 | 1 => {
          let color = YELLOW.clone();
          for fill_x in [
            x.saturating_add(1),
            x.saturating_add(2),
            x.saturating_add(width.saturating_sub(2)),
            x.saturating_add(width.saturating_sub(3)),
          ] {
            render.draw_host_filled_rect(
              canvas,
              fill_x,
              line_y,
              1,
              1,
              Some(" ".into()),
              None,
              Some(color.clone()),
            );
          }
        }
        2 => {
          render.draw_host_filled_rect(
            canvas,
            x.saturating_add(1),
            line_y,
            width.saturating_sub(2),
            1,
            Some(" ".into()),
            None,
            Some(GRAY.clone()),
          );
        }
        _ => {}
      }
      draw_centered(
        render,
        canvas,
        layout,
        x.saturating_add(3),
        width.saturating_sub(6),
        line_y,
        text,
        Some(if index == 2 {
          LIGHT_GRAY.clone()
        } else {
          WHITE.clone()
        }),
        (index == 2).then(|| GRAY.clone()),
        Some(&params),
        false,
      );
    }
  }
}

fn normalized_pattern(pattern: &[String]) -> Vec<String> {
  let mut normalized = pattern
    .iter()
    .map(|key| key.trim().to_ascii_lowercase())
    .collect::<Vec<_>>();
  normalized.sort();
  normalized
}

fn ceil_percent_width(content_width: u16, percent: u16) -> u16 {
  u32::from(content_width)
    .saturating_mul(100)
    .div_ceil(u32::from(percent))
    .min(u32::from(u16::MAX)) as u16
}

fn wrap_hint_items(
  items: impl IntoIterator<Item = String>,
  params: &RichTextParams,
  max_width: u16,
) -> Vec<String> {
  let rich = RichTextService::new();
  let limit = usize::from(max_width.max(1));
  let mut lines = vec![String::new()];
  let mut width = 0usize;
  for item in items {
    let item_width = UnicodeWidthStr::width(rich.visible_text(&item, Some(params)).as_str());
    let gap = usize::from(width > 0) * 2;
    if width > 0 && width.saturating_add(gap).saturating_add(item_width) > limit {
      lines.push(String::new());
      width = 0;
    }
    if width > 0 {
      lines.last_mut().unwrap().push_str("  ");
      width += 2;
    }
    lines.last_mut().unwrap().push_str(&item);
    width = width.saturating_add(item_width);
  }
  lines
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
    priority: 0,
  }
}

fn draw_left(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  column: (u16, u16),
  y: u16,
  text: &str,
  _layout: &LayoutService,
) {
  let width = column.1.saturating_sub(6);
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: column.0.saturating_add(3),
      y,
      text: text.to_string(),
      max_width: Some(width),
      max_height: Some(1),
      overflow_marker: Some("...".to_string()),
      ..Default::default()
    },
  );
}

#[allow(clippy::too_many_arguments)]
fn draw_centered(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  x: u16,
  width: u16,
  y: u16,
  text: &str,
  fg: Option<TextColor>,
  bg: Option<TextColor>,
  params: Option<&RichTextParams>,
  bold: bool,
) {
  let text_width = layout.get_text_width(text, params).min(width);
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: x.saturating_add(width.saturating_sub(text_width) / 2),
      y,
      text: text.to_string(),
      params: params.cloned(),
      fg,
      bg,
      max_width: Some(width),
      max_height: Some(1),
      bold,
      ..Default::default()
    },
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn conflict_detection_normalizes_key_order() {
    let mut ui = GlobalKeyBindingsUi {
      selected: 0,
      mode: EditMode::Edit,
      show_color_doc: false,
      rows: vec![
        BindingRow {
          action: "a".into(),
          description: String::new(),
          keys: vec![vec!["ctrl".into(), "z".into()]],
        },
        BindingRow {
          action: "b".into(),
          description: String::new(),
          keys: vec![vec!["z".into(), "ctrl".into()]],
        },
      ],
      profile: KeyBindingsProfile::default(),
      capture: None,
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
      back_area: HitAreaId(0),
      cell_areas: [[HitAreaId(0); 3]; ROW_COUNT],
    };
    let conflicts = ui.global_conflict_actions();
    assert!(conflicts.contains("a"));
    assert!(conflicts.contains("b"));
    assert!(matches!(
      ui.try_back(),
      Some(GlobalKeyBindingsCommand::Back(_))
    ));
    ui.rows[1].keys.clear();
    assert!(ui.global_conflict_actions().is_empty());
  }

  #[test]
  fn deleting_first_binding_promotes_second() {
    let mut profile = KeyBindingsProfile::default();
    profile
      .user
      .global
      .insert("a".into(), vec![vec!["a".into()], vec!["b".into()]]);
    let hit_area = HitAreaService::new();
    let mut ui = GlobalKeyBindingsUi::init(&hit_area);
    ui.load(
      vec![ActionMapEntry {
        action: "a".into(),
        description: "A".into(),
        keys: vec![vec!["a".into()], vec!["b".into()]],
        priority: 0,
      }],
      profile,
    );
    ui.mode = EditMode::Delete;
    ui.activate_slot(0);
    assert_eq!(ui.rows[0].keys, vec![vec!["b".to_string()]]);
  }

  #[test]
  fn reset_actions_restore_default_bindings() {
    let mut profile = KeyBindingsProfile::default();
    profile
      .default
      .global
      .insert("a".into(), vec![vec!["f1".into()]]);
    profile
      .default
      .global
      .insert("b".into(), vec![vec!["f2".into()]]);
    profile
      .user
      .global
      .insert("a".into(), vec![vec!["a".into()]]);
    profile
      .user
      .global
      .insert("b".into(), vec![vec!["b".into()]]);
    let hit_area = HitAreaService::new();
    let mut ui = GlobalKeyBindingsUi::init(&hit_area);
    ui.load(
      vec![
        ActionMapEntry {
          action: "a".into(),
          description: "A".into(),
          keys: vec![vec!["a".into()]],
          priority: 0,
        },
        ActionMapEntry {
          action: "b".into(),
          description: "B".into(),
          keys: vec![vec!["b".into()]],
          priority: 0,
        },
      ],
      profile,
    );

    ui.reset_selected();
    assert_eq!(ui.rows[0].keys, vec![vec!["f1".to_string()]]);
    assert_eq!(ui.rows[1].keys, vec![vec!["b".to_string()]]);
    ui.reset_all();
    assert_eq!(ui.rows[1].keys, vec![vec!["f2".to_string()]]);
    assert_eq!(ui.profile.user.global, ui.profile.default.global);
  }

  #[test]
  fn raw_binding_waits_eighty_milliseconds_before_committing() {
    let hit_area = HitAreaService::new();
    let mut ui = GlobalKeyBindingsUi::init(&hit_area);
    ui.load(
      vec![ActionMapEntry {
        action: "a".into(),
        description: "A".into(),
        keys: Vec::new(),
        priority: 0,
      }],
      KeyBindingsProfile::default(),
    );
    assert!(matches!(
      ui.activate_slot(1),
      Some(GlobalKeyBindingsCommand::CaptureStarted)
    ));

    let mut input = InputService::new();
    let mut log = crate::host_engine::services::LogService::new();
    input.enable_raw_key_capture();
    input.queue_key_event(
      crate::host_engine::services::KeyEvent {
        key: Key::A,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.poll();
    assert!(!ui.handle_raw_key_events(&mut input, Duration::from_millis(79)));
    assert!(ui.rows[0].keys.is_empty());
    assert!(ui.handle_raw_key_events(&mut input, Duration::from_millis(1)));
    assert_eq!(ui.rows[0].keys, vec![vec!["a".to_string()]]);
  }

  #[test]
  fn modifier_display_fits_global_binding_columns_at_eighty_cells() {
    let hit_area = HitAreaService::new();
    let mut ui = GlobalKeyBindingsUi::init(&hit_area);
    let pattern = vec!["right_shift".to_string(), "q".to_string()];
    ui.load(
      vec![ActionMapEntry {
        action: "host.quit".into(),
        description: "Quit".into(),
        keys: vec![pattern.clone()],
        priority: 0,
      }],
      KeyBindingsProfile::default(),
    );

    let mut layout = LayoutService::new();
    layout.resize_physical(80, 30);
    let table_width = ui.table_width(&layout, 80);
    let inner_width = table_width.saturating_sub(2);
    let key_column_width = inner_width.saturating_mul(30) / 100;
    let available_text_width = key_column_width.saturating_sub(6);
    let display = format_key_display(&[pattern]);

    assert!(available_text_width >= layout.get_text_width(&display, None));
  }
}
