use std::{
  collections::{BTreeMap, HashMap, HashSet},
  time::Duration,
};

use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, InputService, Key, KeyBindingsProfile, KeyEventKind,
  KeyState, LayoutService, MouseButton, Overflow, PackageInfo, PopupRequest, Rect, RenderService,
  RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxId,
  ScrollBoxOptions, ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility,
  TerminalColor, TextColor, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
  format_key_display, key_token,
};

const CAPTURE_DELAY: Duration = Duration::from_millis(80);
const ACTIVE_BORDER: TextColor = TextColor::Rgb {
  r: 95,
  g: 215,
  b: 105,
};
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
const RED: TextColor = TextColor::Terminal(TerminalColor::BrightRed);
const BLUE: TextColor = TextColor::Terminal(TerminalColor::Blue);
const WHITE: TextColor = TextColor::Terminal(TerminalColor::BrightWhite);
const BLACK: TextColor = TextColor::Terminal(TerminalColor::Black);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActivePanel {
  Games,
  Keys,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditMode {
  Edit,
  Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameSort {
  Title,
  Conflict,
}

impl GameSort {
  fn next(self) -> Self {
    match self {
      Self::Title => Self::Conflict,
      Self::Conflict => Self::Title,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Title => "key_bindings_game.list.sort.title",
      Self::Conflict => "key_bindings_game.list.sort.conflict",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySort {
  Priority,
  Name,
  Editable,
  Conflict,
}

impl KeySort {
  fn next(self) -> Self {
    match self {
      Self::Priority => Self::Name,
      Self::Name => Self::Editable,
      Self::Editable => Self::Conflict,
      Self::Conflict => Self::Priority,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Priority => "key_bindings_game.key.sort.priority",
      Self::Name => "key_bindings_game.key.sort.name",
      Self::Editable => "key_bindings_game.key.sort.edit",
      Self::Conflict => "key_bindings_game.key.sort.conflict",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ConflictLevel {
  None,
  Global,
  Internal,
}

#[derive(Clone, Debug)]
struct GameBindingRow {
  action: String,
  description: String,
  keys: Vec<Vec<String>>,
  locked: bool,
  priority: usize,
}

#[derive(Clone, Debug)]
struct GameBindingEntry {
  id: String,
  title: String,
  rows: Vec<GameBindingRow>,
}

#[derive(Clone, Debug)]
struct CaptureState {
  slot: usize,
  keys: Vec<Key>,
  elapsed: Option<Duration>,
}

#[derive(Clone, Debug)]
pub enum GameKeyBindingsCommand {
  Back(KeyBindingsProfile),
  Conflict(PopupRequest),
  FocusSearch,
  BlurSearch,
  CaptureStarted,
  Scroll(i32),
}

struct GameKeyBindingsLayout {
  left: Rect,
  right: Rect,
  search: Rect,
  left_sort_y: u16,
  left_rows: Rect,
  right_header_y: u16,
  right_sort_y: u16,
  right_rows: Rect,
  hint_y: u16,
  hint_lines: Vec<String>,
}

pub struct GameKeyBindingsUi {
  active: ActivePanel,
  mode: EditMode,
  show_color_doc: bool,
  search_text: String,
  games: Vec<GameBindingEntry>,
  selected_game_id: Option<String>,
  selected_action: Option<String>,
  game_ascending: bool,
  game_sort: GameSort,
  key_ascending: bool,
  key_sort: KeySort,
  profile: KeyBindingsProfile,
  capture: Option<CaptureState>,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  search_input: TextInputId,
  left_scroll: ScrollBoxId,
  right_scroll: ScrollBoxId,
  back_area: HitAreaId,
  left_panel_area: HitAreaId,
  right_panel_area: HitAreaId,
  search_area: HitAreaId,
  left_order_area: HitAreaId,
  left_sort_area: HitAreaId,
  right_order_area: HitAreaId,
  right_sort_area: HitAreaId,
  game_areas: Vec<HitAreaId>,
  row_areas: Vec<[HitAreaId; 3]>,
}

impl UiObjectPoolOwner for GameKeyBindingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for GameKeyBindingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl GameKeyBindingsUi {
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
        .expect("failed to create game key bindings scroll box")
    };
    let left_scroll = make_scroll(&mut objects);
    let right_scroll = make_scroll(&mut objects);
    Self {
      active: ActivePanel::Games,
      mode: EditMode::Edit,
      show_color_doc: false,
      search_text: String::new(),
      games: Vec::new(),
      selected_game_id: None,
      selected_action: None,
      game_ascending: true,
      game_sort: GameSort::Title,
      key_ascending: true,
      key_sort: KeySort::Priority,
      profile: KeyBindingsProfile::default(),
      capture: None,
      search_input,
      left_scroll,
      right_scroll,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      left_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      right_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      search_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      left_order_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      left_sort_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      right_order_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      right_sort_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      game_areas: Vec::new(),
      row_areas: Vec::new(),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn load(&mut self, packages: Vec<PackageInfo>, profile: KeyBindingsProfile) {
    self.active = ActivePanel::Games;
    self.mode = EditMode::Edit;
    self.show_color_doc = false;
    self.search_text.clear();
    self.capture = None;
    self.profile = profile;
    self.games = packages
      .into_iter()
      .filter_map(|package| {
        let game = package.game?;
        let title = if game.name.is_empty() {
          package.display.title
        } else {
          game.name
        };
        let mut order = game.action_order;
        let mut remaining = game
          .actions
          .keys()
          .filter(|action| !order.contains(action))
          .cloned()
          .collect::<Vec<_>>();
        remaining.sort();
        order.extend(remaining);
        let rows = order
          .into_iter()
          .filter_map(|action| {
            let config = game.actions.get(&action)?;
            let keys = self
              .profile
              .user
              .games
              .get(&package.id.storage_key())
              .and_then(|actions| actions.get(&action))
              .cloned()
              .unwrap_or_else(|| config.keys.clone());
            Some(GameBindingRow {
              action,
              description: config.description.clone(),
              keys,
              locked: config.lock,
              priority: 0,
            })
          })
          .enumerate()
          .map(|(priority, mut row)| {
            row.priority = priority;
            row
          })
          .collect();
        Some(GameBindingEntry {
          id: package.id.storage_key(),
          title,
          rows,
        })
      })
      .collect();
    self.selected_game_id = self.filtered_game_ids().first().cloned();
    self.select_first_editable_action();
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    [
      ("key_bindings_game.focus_up", "up"),
      ("key_bindings_game.focus_down", "down"),
      ("key_bindings_game.scroll_up", "w"),
      ("key_bindings_game.scroll_down", "s"),
      ("key_bindings_game.back", "esc"),
      ("key_bindings_game.color_doc", "f"),
      ("key_bindings_game.switch", "tab"),
      ("key_bindings_game.search", "c"),
      ("key_bindings_game.order", "z"),
      ("key_bindings_game.sort", "x"),
      ("key_bindings_game.key_switch", "b"),
      ("key_bindings_game.key_edit_del_1", "1"),
      ("key_bindings_game.key_edit_del_2", "2"),
      ("key_bindings_game.reset.only", "r"),
      ("key_bindings_game.reset.all", "t"),
    ]
    .into_iter()
    .map(|(name, key)| action(name, key))
    .collect()
  }

  pub fn is_capturing(&self) -> bool {
    self.capture.is_some()
  }

  pub fn search_input(&self) -> TextInputId {
    self.search_input
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<GameKeyBindingsCommand> {
    if self.capture.is_some() {
      return None;
    }
    if self.show_color_doc {
      if let UiEvent::Action(action) = event
        && action.state == KeyState::Pressed
        && action.action == "key_bindings_game.color_doc"
      {
        self.show_color_doc = false;
      }
      return None;
    }
    match event {
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        Some(GameKeyBindingsCommand::FocusSearch)
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search_text = value.clone();
        self.restore_game_selection(None);
        None
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        Some(GameKeyBindingsCommand::BlurSearch)
      }
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        if *id == self.left_panel_area
          || *id == self.search_area
          || *id == self.left_order_area
          || *id == self.left_sort_area
        {
          self.active = ActivePanel::Games;
        } else if *id == self.right_panel_area
          || *id == self.right_order_area
          || *id == self.right_sort_area
        {
          self.active = ActivePanel::Keys;
        } else if let Some(index) = self.game_areas.iter().position(|area| area == id) {
          self.active = ActivePanel::Games;
          self.select_game_by_visible_index(index);
        } else if let Some((index, _)) = self.row_area_position(*id)
          && self
            .visible_rows()
            .get(index)
            .is_some_and(|row| !row.locked)
        {
          self.active = ActivePanel::Keys;
          self.selected_action = self.visible_rows().get(index).map(|row| row.action.clone());
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.search_area => Some(GameKeyBindingsCommand::FocusSearch),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.left_order_area || *id == self.right_order_area => {
        self.active = if *id == self.left_order_area {
          ActivePanel::Games
        } else {
          ActivePanel::Keys
        };
        self.toggle_order();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.left_sort_area || *id == self.right_sort_area => {
        self.active = if *id == self.left_sort_area {
          ActivePanel::Games
        } else {
          ActivePanel::Keys
        };
        self.toggle_sort();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        if let Some(index) = self.game_areas.iter().position(|area| area == id) {
          self.active = ActivePanel::Games;
          self.select_game_by_visible_index(index);
          return None;
        }
        if let Some((index, column)) = self.row_area_position(*id) {
          let (action, locked) = self
            .visible_rows()
            .get(index)
            .map(|row| (row.action.clone(), row.locked))?;
          if locked {
            return None;
          }
          self.active = ActivePanel::Keys;
          self.selected_action = Some(action);
          return (column > 0)
            .then(|| self.activate_slot(column - 1))
            .flatten();
        }
        if *id == self.left_panel_area {
          self.active = ActivePanel::Games;
        } else if *id == self.right_panel_area {
          self.active = ActivePanel::Keys;
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => self.try_back(),
      UiEvent::Action(action) if action.state == KeyState::Pressed => {
        match action.action.as_str() {
          "key_bindings_game.focus_up" => {
            self.move_selection(-1);
            None
          }
          "key_bindings_game.focus_down" => {
            self.move_selection(1);
            None
          }
          "key_bindings_game.scroll_up" => Some(GameKeyBindingsCommand::Scroll(-3)),
          "key_bindings_game.scroll_down" => Some(GameKeyBindingsCommand::Scroll(3)),
          "key_bindings_game.back" => self.try_back(),
          "key_bindings_game.color_doc" => {
            self.show_color_doc = true;
            None
          }
          "key_bindings_game.switch" => {
            self.active = match self.active {
              ActivePanel::Games => ActivePanel::Keys,
              ActivePanel::Keys => ActivePanel::Games,
            };
            None
          }
          "key_bindings_game.search" if self.active == ActivePanel::Games => {
            Some(GameKeyBindingsCommand::FocusSearch)
          }
          "key_bindings_game.order" => {
            self.toggle_order();
            None
          }
          "key_bindings_game.sort" => {
            self.toggle_sort();
            None
          }
          "key_bindings_game.key_switch" if self.active == ActivePanel::Keys => {
            self.mode = match self.mode {
              EditMode::Edit => EditMode::Delete,
              EditMode::Delete => EditMode::Edit,
            };
            None
          }
          "key_bindings_game.key_edit_del_1" if self.active == ActivePanel::Keys => {
            self.activate_slot(0)
          }
          "key_bindings_game.key_edit_del_2" if self.active == ActivePanel::Keys => {
            self.activate_slot(1)
          }
          "key_bindings_game.reset.only" if self.active == ActivePanel::Keys => {
            self.reset_selected();
            None
          }
          "key_bindings_game.reset.all" if self.active == ActivePanel::Keys => {
            self.reset_current_game();
            None
          }
          _ => None,
        }
      }
      _ => None,
    }
  }

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
      && let Some(row) = self.selected_row_mut()
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

  pub fn scroll_active(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    let id = match self.active {
      ActivePanel::Games => self.left_scroll,
      ActivePanel::Keys => self.right_scroll,
    };
    let _ = scroll_box.scroll_by(&mut self.objects, id, 0, dy, layout);
    let top = scroll_box.scroll_y(&self.objects, id).unwrap_or(0) as usize;
    let height = scroll_box
      .visible_content_height(&self.objects, id, layout)
      .unwrap_or(0) as usize;
    if height == 0 {
      return;
    }
    let bottom = top.saturating_add(height);
    match self.active {
      ActivePanel::Games => {
        let ids = self.filtered_game_ids();
        let current = self
          .selected_game_id
          .as_ref()
          .and_then(|id| ids.iter().position(|candidate| candidate == id))
          .unwrap_or(0);
        if !ids.is_empty() {
          let visible = current.clamp(top, bottom.saturating_sub(1).min(ids.len() - 1));
          self.select_game_by_visible_index(visible);
        }
      }
      ActivePanel::Keys => {
        let rows = self.visible_rows();
        let current = self
          .selected_action
          .as_ref()
          .and_then(|action| rows.iter().position(|row| &row.action == action));
        if current.is_none_or(|index| index < top || index >= bottom) {
          let visible = rows
            .iter()
            .enumerate()
            .skip(top)
            .take(height)
            .filter(|(_, row)| !row.locked)
            .map(|(index, row)| (index, row.action.clone()))
            .collect::<Vec<_>>();
          self.selected_action = if dy < 0 {
            visible.first()
          } else {
            visible.last()
          }
          .map(|(_, action)| action.clone());
        }
      }
    }
  }

  pub fn prepare_surfaces(
    &mut self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) {
    let pos = self.compute_layout(layout, i18n, text_input);
    let viewport = layout.developer_viewport_rect();
    let local = |rect: Rect| Rect {
      x: rect.x.saturating_sub(viewport.x),
      y: rect.y.saturating_sub(viewport.y),
      width: rect.width,
      height: rect.height,
    };
    let left = local(pos.left_rows);
    let right = local(pos.right_rows);
    let left_len = self.filtered_game_ids().len() as u16;
    let right_len = self.visible_rows().len() as u16;
    let _ = scroll_box.set_rect(&mut self.objects, self.left_scroll, left, layout);
    let _ = scroll_box.set_rect(&mut self.objects, self.right_scroll, right, layout);
    let _ = scroll_box.set_content_size(
      &mut self.objects,
      self.left_scroll,
      left.width.saturating_sub(1).max(1),
      left_len.max(left.height).max(1),
      layout,
    );
    let _ = scroll_box.set_content_size(
      &mut self.objects,
      self.right_scroll,
      right.width.saturating_sub(1).max(1),
      right_len.max(right.height).max(1),
      layout,
    );
    self.ensure_selection_visible(scroll_box, layout);
  }

  #[allow(clippy::too_many_arguments)]
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
    self.draw_frames(render, canvas, layout, i18n, &pos);
    self.draw_search(canvas, i18n, text_input, &pos);
    self.draw_games(render, canvas, layout, i18n, &pos);
    self.draw_key_rows(render, canvas, layout, i18n, &pos);
    self.draw_hints(render, canvas, layout, &pos);
    self.register_hit_areas(hit_area, scroll_box, canvas, &pos);
    if self.show_color_doc {
      self.draw_color_doc(render, canvas, layout, i18n, &pos);
    }
  }

  fn activate_slot(&mut self, requested_slot: usize) -> Option<GameKeyBindingsCommand> {
    let row = self.selected_row()?;
    if row.locked {
      return None;
    }
    match self.mode {
      EditMode::Delete => {
        let row = self.selected_row_mut()?;
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
        Some(GameKeyBindingsCommand::CaptureStarted)
      }
    }
  }

  fn try_back(&self) -> Option<GameKeyBindingsCommand> {
    if self.has_internal_conflicts() {
      Some(GameKeyBindingsCommand::Conflict(PopupRequest {
        text: String::new(),
        color: RED,
        duration: Duration::from_secs(2),
        dismiss_on: Vec::new(),
        replaceable: true,
        persistent: false,
      }))
    } else {
      Some(GameKeyBindingsCommand::Back(self.profile.clone()))
    }
  }

  fn selected_game(&self) -> Option<&GameBindingEntry> {
    let id = self.selected_game_id.as_ref()?;
    self.games.iter().find(|game| &game.id == id)
  }

  fn selected_game_mut(&mut self) -> Option<&mut GameBindingEntry> {
    let id = self.selected_game_id.clone()?;
    self.games.iter_mut().find(|game| game.id == id)
  }

  fn selected_row(&self) -> Option<&GameBindingRow> {
    let action = self.selected_action.as_ref()?;
    self
      .selected_game()?
      .rows
      .iter()
      .find(|row| &row.action == action)
  }

  fn selected_row_mut(&mut self) -> Option<&mut GameBindingRow> {
    let action = self.selected_action.clone()?;
    self
      .selected_game_mut()?
      .rows
      .iter_mut()
      .find(|row| row.action == action)
  }

  fn filtered_game_ids(&self) -> Vec<String> {
    let query = self.search_text.to_lowercase();
    let mut games = self
      .games
      .iter()
      .filter(|game| {
        query.is_empty()
          || RichTextService::new()
            .visible_text(&game.title, Some(&self.game_params(&game.id)))
            .to_lowercase()
            .contains(&query)
      })
      .collect::<Vec<_>>();
    games.sort_by(|a, b| {
      let order = match self.game_sort {
        GameSort::Title => self.visible_game_title(a).cmp(&self.visible_game_title(b)),
        GameSort::Conflict => self
          .game_conflict_level(a)
          .cmp(&self.game_conflict_level(b))
          .reverse()
          .then_with(|| self.visible_game_title(a).cmp(&self.visible_game_title(b))),
      };
      if self.game_ascending {
        order
      } else {
        order.reverse()
      }
    });
    games.into_iter().map(|game| game.id.clone()).collect()
  }

  fn visible_rows(&self) -> Vec<&GameBindingRow> {
    let Some(game) = self.selected_game() else {
      return Vec::new();
    };
    let mut rows = game.rows.iter().collect::<Vec<_>>();
    rows.sort_by(|a, b| {
      let order = match self.key_sort {
        KeySort::Priority => a.priority.cmp(&b.priority),
        KeySort::Name => self
          .visible_description(game, a)
          .cmp(&self.visible_description(game, b)),
        KeySort::Editable => a.locked.cmp(&b.locked).then_with(|| {
          self
            .visible_description(game, a)
            .cmp(&self.visible_description(game, b))
        }),
        KeySort::Conflict => self
          .row_conflict_level(game, a)
          .cmp(&self.row_conflict_level(game, b))
          .reverse()
          .then_with(|| {
            self
              .visible_description(game, a)
              .cmp(&self.visible_description(game, b))
          }),
      };
      if self.key_ascending {
        order
      } else {
        order.reverse()
      }
    });
    rows
  }

  fn game_params(&self, game_id: &str) -> RichTextParams {
    let user = self
      .profile
      .user
      .games
      .get(game_id)
      .cloned()
      .unwrap_or_default();
    let default = self
      .profile
      .default
      .games
      .get(game_id)
      .cloned()
      .unwrap_or_default();
    RichTextParams::from_key_action_maps(&map_to_hash(&user), &map_to_hash(&default))
  }

  fn visible_game_title(&self, game: &GameBindingEntry) -> String {
    RichTextService::new().visible_text(&game.title, Some(&self.game_params(&game.id)))
  }

  fn visible_description(&self, game: &GameBindingEntry, row: &GameBindingRow) -> String {
    RichTextService::new().visible_text(&row.description, Some(&self.game_params(&game.id)))
  }

  fn internal_conflict_actions(&self, game: &GameBindingEntry) -> HashSet<String> {
    let mut owners: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    for row in &game.rows {
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

  fn global_patterns(&self) -> HashSet<Vec<String>> {
    self
      .profile
      .user
      .global
      .values()
      .flatten()
      .map(|pattern| normalized_pattern(pattern))
      .collect()
  }

  fn row_conflict_level(&self, game: &GameBindingEntry, row: &GameBindingRow) -> ConflictLevel {
    if self.internal_conflict_actions(game).contains(&row.action) {
      ConflictLevel::Internal
    } else {
      let global = self.global_patterns();
      if row
        .keys
        .iter()
        .any(|pattern| global.contains(&normalized_pattern(pattern)))
      {
        ConflictLevel::Global
      } else {
        ConflictLevel::None
      }
    }
  }

  fn game_conflict_level(&self, game: &GameBindingEntry) -> ConflictLevel {
    game
      .rows
      .iter()
      .map(|row| self.row_conflict_level(game, row))
      .max()
      .unwrap_or(ConflictLevel::None)
  }

  fn has_internal_conflicts(&self) -> bool {
    self
      .games
      .iter()
      .any(|game| !self.internal_conflict_actions(game).is_empty())
  }

  fn select_game_by_visible_index(&mut self, index: usize) {
    if let Some(id) = self.filtered_game_ids().get(index).cloned()
      && self.selected_game_id.as_ref() != Some(&id)
    {
      self.selected_game_id = Some(id);
      self.select_first_editable_action();
    }
  }

  fn select_first_editable_action(&mut self) {
    self.selected_action = self
      .visible_rows()
      .into_iter()
      .find(|row| !row.locked)
      .map(|row| row.action.clone());
  }

  fn move_selection(&mut self, delta: i32) {
    match self.active {
      ActivePanel::Games => {
        let ids = self.filtered_game_ids();
        let current = self
          .selected_game_id
          .as_ref()
          .and_then(|id| ids.iter().position(|candidate| candidate == id))
          .unwrap_or(0);
        let next = move_index(current, ids.len(), delta);
        self.select_game_by_visible_index(next);
      }
      ActivePanel::Keys => {
        let editable = self
          .visible_rows()
          .into_iter()
          .filter(|row| !row.locked)
          .map(|row| row.action.clone())
          .collect::<Vec<_>>();
        let current = self
          .selected_action
          .as_ref()
          .and_then(|action| editable.iter().position(|candidate| candidate == action))
          .unwrap_or(0);
        self.selected_action = editable
          .get(move_index(current, editable.len(), delta))
          .cloned();
      }
    }
  }

  fn restore_game_selection(&mut self, preferred: Option<String>) {
    let ids = self.filtered_game_ids();
    let selected = preferred
      .or_else(|| self.selected_game_id.clone())
      .filter(|id| ids.contains(id))
      .or_else(|| ids.first().cloned());
    if selected != self.selected_game_id {
      self.selected_game_id = selected;
      self.select_first_editable_action();
    }
  }

  fn toggle_order(&mut self) {
    match self.active {
      ActivePanel::Games => {
        let selected = self.selected_game_id.clone();
        self.game_ascending = !self.game_ascending;
        self.restore_game_selection(selected);
      }
      ActivePanel::Keys => {
        let selected = self.selected_action.clone();
        self.key_ascending = !self.key_ascending;
        self.restore_action_selection(selected);
      }
    }
  }

  fn toggle_sort(&mut self) {
    match self.active {
      ActivePanel::Games => {
        let selected = self.selected_game_id.clone();
        self.game_sort = self.game_sort.next();
        self.restore_game_selection(selected);
      }
      ActivePanel::Keys => {
        let selected = self.selected_action.clone();
        self.key_sort = self.key_sort.next();
        self.restore_action_selection(selected);
      }
    }
  }

  fn restore_action_selection(&mut self, preferred: Option<String>) {
    let editable = self
      .visible_rows()
      .into_iter()
      .filter(|row| !row.locked)
      .map(|row| row.action.clone())
      .collect::<Vec<_>>();
    self.selected_action = preferred
      .filter(|action| editable.contains(action))
      .or_else(|| editable.first().cloned());
  }

  fn sync_selected_to_profile(&mut self) {
    let Some(game_id) = self.selected_game_id.clone() else {
      return;
    };
    let Some(row) = self.selected_row() else {
      return;
    };
    let action = row.action.clone();
    let keys = row.keys.clone();
    self
      .profile
      .user
      .games
      .entry(game_id)
      .or_default()
      .insert(action, keys);
  }

  fn reset_selected(&mut self) {
    let Some(game_id) = self.selected_game_id.clone() else {
      return;
    };
    let Some(action) = self.selected_action.clone() else {
      return;
    };
    let defaults = self
      .profile
      .default
      .games
      .get(&game_id)
      .and_then(|actions| actions.get(&action))
      .cloned()
      .unwrap_or_default();
    if let Some(row) = self.selected_row_mut()
      && !row.locked
    {
      row.keys = defaults;
      self.sync_selected_to_profile();
    }
  }

  fn reset_current_game(&mut self) {
    let Some(game_id) = self.selected_game_id.clone() else {
      return;
    };
    let defaults = self
      .profile
      .default
      .games
      .get(&game_id)
      .cloned()
      .unwrap_or_default();
    let updated = {
      let Some(game) = self.selected_game_mut() else {
        return;
      };
      for row in &mut game.rows {
        if !row.locked {
          row.keys = defaults.get(&row.action).cloned().unwrap_or_default();
        }
      }
      game
        .rows
        .iter()
        .map(|row| (row.action.clone(), row.keys.clone()))
        .collect::<Vec<_>>()
    };
    let user = self.profile.user.games.entry(game_id).or_default();
    for (action, keys) in updated {
      user.insert(action, keys);
    }
  }

  fn compute_layout(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> GameKeyBindingsLayout {
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
    GameKeyBindingsLayout {
      left,
      right,
      search: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(1),
        width: left.width.saturating_sub(2),
        height: 1,
      },
      left_sort_y: left.y.saturating_add(2),
      left_rows: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(3),
        width: left.width.saturating_sub(2),
        height: left.height.saturating_sub(4),
      },
      right_header_y: right.y.saturating_add(1),
      right_sort_y: right.y.saturating_add(2),
      right_rows: Rect {
        x: right.x.saturating_add(1),
        y: right.y.saturating_add(3),
        width: right.width.saturating_sub(2),
        height: right.height.saturating_sub(4),
      },
      hint_y: viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(hint_h),
      hint_lines,
    }
  }

  fn draw_frames(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameKeyBindingsLayout,
  ) {
    for (rect, active) in [
      (pos.left, self.active == ActivePanel::Games),
      (pos.right, self.active == ActivePanel::Keys),
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
          WHITE.clone()
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
      i18n.get_runtime_text("key_bindings_game", "key_bindings_game.list"),
    );
    self.draw_panel_title(
      render,
      canvas,
      pos.right,
      i18n.get_runtime_text("key_bindings_game", "key_bindings_game.key"),
    );
    self.draw_sort_line(
      render,
      canvas,
      i18n,
      pos.left,
      pos.left_sort_y,
      self.game_ascending,
      self.game_sort.key(),
      self.active == ActivePanel::Games,
      true,
    );
    self.draw_key_headers(render, canvas, layout, i18n, pos);
    self.draw_sort_line(
      render,
      canvas,
      i18n,
      pos.right,
      pos.right_sort_y,
      self.key_ascending,
      self.key_sort.key(),
      self.active == ActivePanel::Keys,
      false,
    );
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
        text: format!("f%<fg:bright_magenta><b>{title}</b></fg>"),
        max_width: Some(rect.width.saturating_sub(2)),
        ..Default::default()
      },
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_sort_line(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    rect: Rect,
    y: u16,
    ascending: bool,
    sort_key: &str,
    active: bool,
    list: bool,
  ) {
    let order_key = if list {
      if ascending {
        "key_bindings_game.list.order.ascending"
      } else {
        "key_bindings_game.list.order.descending"
      }
    } else if ascending {
      "key_bindings_game.key.order.ascending"
    } else {
      "key_bindings_game.key.order.descending"
    };
    let order = i18n.get_runtime_text("key_bindings_game", order_key);
    let sort = i18n.get_runtime_text("key_bindings_game", sort_key);
    let labels = format!("[{order}]{sort}");
    let label_w = UnicodeWidthStr::width(labels.as_str()) as u16;
    let line_w = rect.width.saturating_sub(label_w + 2);
    let border_color = if active {
      "bright_green"
    } else {
      "bright_white"
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x,
        y,
        text: format!(
          "f%<fg:{border_color}>├[</fg><fg:bright_yellow>{order}</fg><fg:{border_color}>]</fg><fg:bright_green>{sort}</fg><fg:{border_color}>{}┤</fg>",
          "─".repeat(line_w as usize)
        ),
        max_width: Some(rect.width),
        ..Default::default()
      },
    );
  }

  fn draw_key_headers(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameKeyBindingsLayout,
  ) {
    let columns = key_columns(pos.right_rows.width.saturating_sub(1));
    let headers = [
      i18n.get_runtime_text("key_bindings_game", "key_bindings_game.action"),
      i18n.get_runtime_text("key_bindings_game", "key_bindings_game.key1"),
      i18n.get_runtime_text("key_bindings_game", "key_bindings_game.key2"),
    ];
    for ((x, width), header) in columns.iter().copied().zip(headers) {
      draw_centered(
        render,
        canvas,
        layout,
        pos.right_rows.x.saturating_add(x),
        width,
        pos.right_header_y,
        &header,
        None,
        None,
        None,
        true,
      );
    }
  }

  fn draw_search(
    &mut self,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    text_input: &TextInputService,
    pos: &GameKeyBindingsLayout,
  ) {
    text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search,
        placeholder: i18n.get_runtime_text(
          "key_bindings_game",
          "key_bindings_game.list.search.placeholder",
        ),
        fg: Some(WHITE.clone()),
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

  fn draw_games(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameKeyBindingsLayout,
  ) {
    let ids = self.filtered_game_ids();
    if ids.is_empty() {
      draw_empty(
        render,
        canvas,
        layout,
        pos.left_rows,
        &i18n.get_runtime_text("key_bindings_game", "key_bindings_game.no.game"),
      );
      return;
    }
    for (index, id) in ids.iter().enumerate() {
      let Some(game) = self.games.iter().find(|game| &game.id == id) else {
        continue;
      };
      let width = pos.left_rows.width.saturating_sub(1);
      if self.selected_game_id.as_ref() == Some(id) {
        render.draw_text_in_scroll_box(
          canvas,
          self.left_scroll,
          &DrawTextParams {
            x: 0,
            y: index as u16,
            text: "f%<fg:bright_cyan>▌</fg>".into(),
            ..Default::default()
          },
        );
      }
      let marker = match self.game_conflict_level(game) {
        ConflictLevel::Internal => Some(RED.clone()),
        ConflictLevel::Global => Some(YELLOW.clone()),
        ConflictLevel::None => None,
      };
      let marker_w = u16::from(marker.is_some());
      render.draw_text_in_scroll_box(
        canvas,
        self.left_scroll,
        &DrawTextParams {
          x: 2,
          y: index as u16,
          text: game.title.clone(),
          params: Some(self.game_params(&game.id)),
          max_width: Some(width.saturating_sub(marker_w + 2)),
          max_height: Some(1),
          overflow_marker: Some("...".into()),
          ..Default::default()
        },
      );
      if let Some(color) = marker {
        render.draw_text_in_scroll_box(
          canvas,
          self.left_scroll,
          &DrawTextParams {
            x: width.saturating_sub(marker_w),
            y: index as u16,
            text: "▌".into(),
            fg: Some(color),
            ..Default::default()
          },
        );
      }
    }
  }

  fn draw_key_rows(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameKeyBindingsLayout,
  ) {
    let rows = self.visible_rows();
    if rows.is_empty() {
      draw_empty(
        render,
        canvas,
        layout,
        pos.right_rows,
        &i18n.get_runtime_text("key_bindings_game", "key_bindings_game.no.key"),
      );
      return;
    }
    let Some(game) = self.selected_game() else {
      return;
    };
    let columns = key_columns(pos.right_rows.width.saturating_sub(1));
    let params = self.game_params(&game.id);
    for (index, row) in rows.into_iter().enumerate() {
      let y = index as u16;
      let width = pos.right_rows.width.saturating_sub(1);
      if row.locked {
        render.draw_filled_rect_in_scroll_box(
          canvas,
          self.right_scroll,
          0,
          y,
          width,
          1,
          Some(" ".into()),
          None,
          Some(GRAY.clone()),
        );
      }
      let conflict = self.row_conflict_level(game, row);
      let conflict_color = match conflict {
        ConflictLevel::Internal => Some(RED.clone()),
        ConflictLevel::Global => Some(YELLOW.clone()),
        ConflictLevel::None => None,
      };
      if let Some(color) = conflict_color {
        for x in [1, 2, width.saturating_sub(2), width.saturating_sub(3)] {
          render.draw_filled_rect_in_scroll_box(
            canvas,
            self.right_scroll,
            x,
            y,
            1,
            1,
            Some(" ".into()),
            None,
            Some(color.clone()),
          );
        }
      }
      if !row.locked && self.selected_action.as_ref() == Some(&row.action) {
        let color = match self.mode {
          EditMode::Edit => CYAN.clone(),
          EditMode::Delete => MAGENTA.clone(),
        };
        for x in [0, width.saturating_sub(1)] {
          render.draw_filled_rect_in_scroll_box(
            canvas,
            self.right_scroll,
            x,
            y,
            1,
            1,
            Some(" ".into()),
            None,
            Some(color.clone()),
          );
        }
      }
      draw_scroll_left(
        render,
        canvas,
        self.right_scroll,
        columns[0],
        y,
        &row.description,
        Some(&params),
        row.locked,
      );
      for slot in 0..2 {
        let text = row.keys.get(slot).map_or_else(String::new, |pattern| {
          format_key_display(std::slice::from_ref(pattern))
        });
        let capturing = self.selected_action.as_ref() == Some(&row.action)
          && self
            .capture
            .as_ref()
            .is_some_and(|capture| capture.slot == slot);
        if capturing {
          render.draw_filled_rect_in_scroll_box(
            canvas,
            self.right_scroll,
            columns[slot + 1].0.saturating_add(3),
            y,
            columns[slot + 1].1.saturating_sub(6),
            1,
            Some(" ".into()),
            None,
            Some(BLUE.clone()),
          );
        }
        draw_scroll_centered(
          render,
          canvas,
          layout,
          self.right_scroll,
          columns[slot + 1],
          y,
          &text,
          if row.locked {
            LIGHT_GRAY.clone()
          } else {
            WHITE.clone()
          },
          if capturing {
            Some(BLUE.clone())
          } else if row.locked {
            Some(GRAY.clone())
          } else {
            None
          },
        );
      }
    }
  }

  fn hint_lines(
    &self,
    i18n: &I18nService,
    _text_input: &TextInputService,
    width: u16,
  ) -> Vec<String> {
    let keys = if self.capture.is_some() {
      vec!["key_bindings_game.action.any"]
    } else if self.active == ActivePanel::Games {
      vec![
        "key_bindings_game.action.select",
        "key_bindings_game.action.back",
        if self.show_color_doc {
          "key_bindings_game.action.color_doc.out"
        } else {
          "key_bindings_game.action.color_doc.in"
        },
        "key_bindings_game.action.scroll.list",
        "key_bindings_game.action.switch_list",
        "key_bindings_game.action.list.search",
        "key_bindings_game.action.list.order",
        "key_bindings_game.action.list.sort",
      ]
    } else {
      vec![
        "key_bindings_game.action.select",
        "key_bindings_game.action.back",
        if self.show_color_doc {
          "key_bindings_game.action.color_doc.out"
        } else {
          "key_bindings_game.action.color_doc.in"
        },
        "key_bindings_game.action.scroll.list",
        "key_bindings_game.action.switch_list",
        "key_bindings_game.action.list.order",
        "key_bindings_game.action.list.sort",
        "key_bindings_game.action.switch",
        match self.mode {
          EditMode::Edit => "key_bindings_game.action.key.edit",
          EditMode::Delete => "key_bindings_game.action.key.del",
        },
        "key_bindings_game.action.reset.only",
        "key_bindings_game.action.reset.all",
      ]
    };
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_game.");
    wrap_hint_items(
      keys
        .into_iter()
        .map(|key| i18n.get_runtime_text("key_bindings_game", key)),
      &params,
      width,
    )
  }

  fn draw_hints(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    pos: &GameKeyBindingsLayout,
  ) {
    let viewport = layout.developer_viewport_rect();
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_game.");
    for (index, line) in pos.hint_lines.iter().enumerate() {
      let visible = RichTextService::new().visible_text(line, Some(&params));
      let width = UnicodeWidthStr::width(visible.as_str()) as u16;
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: viewport
            .x
            .saturating_add(viewport.width.saturating_sub(width) / 2),
          y: pos.hint_y.saturating_add(index as u16),
          text: format!("f%<fg:rgb(85,87,83)>{line}</fg>"),
          params: Some(params.clone()),
          max_width: Some(viewport.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
  }

  fn draw_color_doc(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    pos: &GameKeyBindingsLayout,
  ) {
    let keys = [
      "key_bindings_game.color_doc.yellow",
      "key_bindings_game.color_doc.red",
      "key_bindings_game.color_doc.gray",
      "key_bindings_game.color_doc.composite",
      "key_bindings_game.color_doc.priority",
      "key_bindings_game.action.color_doc.out",
    ];
    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings_game.");
    let texts = keys
      .iter()
      .map(|key| format!("f%{}", i18n.get_runtime_text("key_bindings_game", key)))
      .collect::<Vec<_>>();
    let content_w = texts
      .iter()
      .map(|text| layout.get_text_width(text, Some(&params)))
      .max()
      .unwrap_or_default();
    let page = Rect {
      x: pos.left.x,
      y: pos.left.y,
      width: pos.left.width.saturating_add(pos.right.width),
      height: pos.left.height.max(pos.right.height),
    };
    let width = content_w.saturating_add(8).min(page.width);
    let height = 9.min(page.height);
    let x = page.x.saturating_add(page.width.saturating_sub(width) / 2);
    let y = page
      .y
      .saturating_add(page.height.saturating_sub(height) / 2);
    render.draw_host_border_rect(
      canvas,
      x,
      y,
      width,
      height,
      &BorderStyle::Line,
      Some(WHITE.clone()),
      None,
      Some(BLACK.clone()),
      None,
    );
    for (index, text) in texts.iter().enumerate() {
      let line_y = y.saturating_add(1 + index as u16 + u16::from(index == 5));
      match index {
        0 | 1 => {
          let color = if index == 0 {
            YELLOW.clone()
          } else {
            RED.clone()
          };
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
        2 => render.draw_host_filled_rect(
          canvas,
          x.saturating_add(1),
          line_y,
          width.saturating_sub(2),
          1,
          Some(" ".into()),
          None,
          Some(GRAY.clone()),
        ),
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

  fn register_hit_areas(
    &mut self,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    canvas: &mut CanvasService,
    pos: &GameKeyBindingsLayout,
  ) {
    let game_len = self.filtered_game_ids().len();
    let row_len = self.visible_rows().len();
    resize_hit_areas(&mut self.objects, hit_area, &mut self.game_areas, game_len);
    resize_row_areas(&mut self.objects, hit_area, &mut self.row_areas, row_len);
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
    hit_area.render_host(&mut self.objects, self.search_area, pos.search, canvas);
    self.render_sort_hit_areas(hit_area, canvas, pos);
    let left_top = scroll_box
      .scroll_y(&self.objects, self.left_scroll)
      .unwrap_or(0) as usize;
    for (index, id) in self
      .game_areas
      .iter()
      .enumerate()
      .skip(left_top)
      .take(pos.left_rows.height as usize)
    {
      hit_area.render_host(
        &mut self.objects,
        *id,
        Rect {
          x: pos.left_rows.x,
          y: pos.left_rows.y.saturating_add((index - left_top) as u16),
          width: pos.left_rows.width.saturating_sub(1),
          height: 1,
        },
        canvas,
      );
    }
    let right_top = scroll_box
      .scroll_y(&self.objects, self.right_scroll)
      .unwrap_or(0) as usize;
    let columns = key_columns(pos.right_rows.width.saturating_sub(1));
    let locked_rows = self
      .visible_rows()
      .into_iter()
      .map(|row| row.locked)
      .collect::<Vec<_>>();
    for (index, areas) in self
      .row_areas
      .iter()
      .enumerate()
      .skip(right_top)
      .take(pos.right_rows.height as usize)
    {
      if locked_rows.get(index).copied().unwrap_or(false) {
        continue;
      }
      let y = pos.right_rows.y.saturating_add((index - right_top) as u16);
      for (column, id) in areas.iter().enumerate() {
        hit_area.render_host(
          &mut self.objects,
          *id,
          Rect {
            x: pos.right_rows.x.saturating_add(columns[column].0),
            y,
            width: columns[column].1,
            height: 1,
          },
          canvas,
        );
      }
    }
  }

  fn render_sort_hit_areas(
    &mut self,
    hit_area: &HitAreaService,
    canvas: &mut CanvasService,
    pos: &GameKeyBindingsLayout,
  ) {
    let left_order_w = 12.min(pos.left.width.saturating_sub(2));
    let right_order_w = 12.min(pos.right.width.saturating_sub(2));
    for (id, rect) in [
      (
        self.left_order_area,
        Rect {
          x: pos.left.x.saturating_add(1),
          y: pos.left_sort_y,
          width: left_order_w,
          height: 1,
        },
      ),
      (
        self.left_sort_area,
        Rect {
          x: pos.left.x.saturating_add(1 + left_order_w),
          y: pos.left_sort_y,
          width: pos.left.width.saturating_sub(left_order_w + 2),
          height: 1,
        },
      ),
      (
        self.right_order_area,
        Rect {
          x: pos.right.x.saturating_add(1),
          y: pos.right_sort_y,
          width: right_order_w,
          height: 1,
        },
      ),
      (
        self.right_sort_area,
        Rect {
          x: pos.right.x.saturating_add(1 + right_order_w),
          y: pos.right_sort_y,
          width: pos.right.width.saturating_sub(right_order_w + 2),
          height: 1,
        },
      ),
    ] {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  fn row_area_position(&self, id: HitAreaId) -> Option<(usize, usize)> {
    self.row_areas.iter().enumerate().find_map(|(row, areas)| {
      areas
        .iter()
        .position(|area| *area == id)
        .map(|column| (row, column))
    })
  }

  fn ensure_selection_visible(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService) {
    let (id, selected) = match self.active {
      ActivePanel::Games => {
        let ids = self.filtered_game_ids();
        let selected = self
          .selected_game_id
          .as_ref()
          .and_then(|game| ids.iter().position(|candidate| candidate == game))
          .unwrap_or(0);
        (self.left_scroll, selected)
      }
      ActivePanel::Keys => {
        let rows = self.visible_rows();
        let selected = self
          .selected_action
          .as_ref()
          .and_then(|action| rows.iter().position(|row| &row.action == action))
          .unwrap_or(0);
        (self.right_scroll, selected)
      }
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
}

fn normalized_pattern(pattern: &[String]) -> Vec<String> {
  let mut normalized = pattern
    .iter()
    .map(|key| key.trim().to_ascii_lowercase())
    .collect::<Vec<_>>();
  normalized.sort();
  normalized
}

fn map_to_hash(map: &BTreeMap<String, Vec<Vec<String>>>) -> HashMap<String, Vec<Vec<String>>> {
  map
    .iter()
    .map(|(action, keys)| (action.clone(), keys.clone()))
    .collect()
}

fn move_index(current: usize, len: usize, delta: i32) -> usize {
  if len == 0 {
    return 0;
  }
  (current as i32 + delta).clamp(0, len.saturating_sub(1) as i32) as usize
}

fn key_columns(width: u16) -> [(u16, u16); 3] {
  let action = width.saturating_mul(40) / 100;
  let key1 = width.saturating_mul(30) / 100;
  let key2 = width.saturating_sub(action + key1);
  [(0, action), (action, key1), (action + key1, key2)]
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

fn resize_row_areas(
  pool: &mut UiObjectPool,
  service: &HitAreaService,
  areas: &mut Vec<[HitAreaId; 3]>,
  len: usize,
) {
  while areas.len() > len {
    if let Some(row) = areas.pop() {
      for id in row {
        service.remove(pool, id);
      }
    }
  }
  while areas.len() < len {
    areas.push(std::array::from_fn(|_| {
      service.create(pool, HitAreaOptions::default())
    }));
  }
}

fn action(name: &str, key: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.into(),
    description: name.into(),
    keys: vec![vec![key.into()]],
  }
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

fn draw_empty(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  rect: Rect,
  text: &str,
) {
  let width = layout.get_text_width(text, None).min(rect.width);
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: rect.x.saturating_add(rect.width.saturating_sub(width) / 2),
      y: rect.y.saturating_add(rect.height.saturating_sub(1) / 2),
      text: format!("f%<fg:rgb(85,87,83)>{text}</fg>"),
      max_width: Some(rect.width),
      max_height: Some(1),
      ..Default::default()
    },
  );
}

#[allow(clippy::too_many_arguments)]
fn draw_scroll_left(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  scroll: ScrollBoxId,
  column: (u16, u16),
  y: u16,
  text: &str,
  params: Option<&RichTextParams>,
  locked: bool,
) {
  render.draw_text_in_scroll_box(
    canvas,
    scroll,
    &DrawTextParams {
      x: column.0.saturating_add(3),
      y,
      text: text.to_string(),
      params: params.cloned(),
      fg: locked.then(|| LIGHT_GRAY.clone()),
      bg: locked.then(|| GRAY.clone()),
      max_width: Some(column.1.saturating_sub(6)),
      max_height: Some(1),
      overflow_marker: Some("...".into()),
      ..Default::default()
    },
  );
}

#[allow(clippy::too_many_arguments)]
fn draw_scroll_centered(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  scroll: ScrollBoxId,
  column: (u16, u16),
  y: u16,
  text: &str,
  fg: TextColor,
  bg: Option<TextColor>,
) {
  let inner_x = column.0.saturating_add(3);
  let inner_w = column.1.saturating_sub(6);
  let text_w = layout.get_text_width(text, None).min(inner_w);
  render.draw_text_in_scroll_box(
    canvas,
    scroll,
    &DrawTextParams {
      x: inner_x.saturating_add(inner_w.saturating_sub(text_w) / 2),
      y,
      text: text.to_string(),
      fg: Some(fg),
      bg,
      max_width: Some(inner_w),
      max_height: Some(1),
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

  fn test_ui() -> GameKeyBindingsUi {
    GameKeyBindingsUi::init(
      &HitAreaService::new(),
      &TextInputService::new(),
      &ScrollBoxService::new(),
    )
  }

  #[test]
  fn conflicts_are_scoped_to_one_game_and_global_bindings() {
    let mut profile = KeyBindingsProfile::default();
    profile
      .user
      .global
      .insert("host".into(), vec![vec!["f1".into()]]);
    let ui = GameKeyBindingsUi {
      active: ActivePanel::Games,
      mode: EditMode::Edit,
      show_color_doc: false,
      search_text: String::new(),
      games: Vec::new(),
      selected_game_id: None,
      selected_action: None,
      game_ascending: true,
      game_sort: GameSort::Title,
      key_ascending: true,
      key_sort: KeySort::Priority,
      profile,
      capture: None,
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
      search_input: TextInputId(0),
      left_scroll: ScrollBoxId(0),
      right_scroll: ScrollBoxId(0),
      back_area: HitAreaId(0),
      left_panel_area: HitAreaId(0),
      right_panel_area: HitAreaId(0),
      search_area: HitAreaId(0),
      left_order_area: HitAreaId(0),
      left_sort_area: HitAreaId(0),
      right_order_area: HitAreaId(0),
      right_sort_area: HitAreaId(0),
      game_areas: Vec::new(),
      row_areas: Vec::new(),
    };
    let game = GameBindingEntry {
      id: "game".into(),
      title: "Game".into(),
      rows: vec![
        GameBindingRow {
          action: "a".into(),
          description: "A".into(),
          keys: vec![vec!["f1".into()]],
          locked: false,
          priority: 0,
        },
        GameBindingRow {
          action: "b".into(),
          description: "B".into(),
          keys: vec![vec!["z".into()]],
          locked: false,
          priority: 1,
        },
        GameBindingRow {
          action: "c".into(),
          description: "C".into(),
          keys: vec![vec!["z".into()]],
          locked: false,
          priority: 2,
        },
      ],
    };
    assert_eq!(
      ui.row_conflict_level(&game, &game.rows[0]),
      ConflictLevel::Global
    );
    assert_eq!(
      ui.row_conflict_level(&game, &game.rows[1]),
      ConflictLevel::Internal
    );
  }

  #[test]
  fn locked_actions_are_skipped_and_cannot_start_capture() {
    let mut ui = test_ui();
    ui.games = vec![GameBindingEntry {
      id: "game".into(),
      title: "Game".into(),
      rows: vec![
        GameBindingRow {
          action: "first".into(),
          description: "First".into(),
          keys: vec![vec!["a".into()]],
          locked: false,
          priority: 0,
        },
        GameBindingRow {
          action: "locked".into(),
          description: "Locked".into(),
          keys: vec![vec!["b".into()]],
          locked: true,
          priority: 1,
        },
        GameBindingRow {
          action: "last".into(),
          description: "Last".into(),
          keys: vec![vec!["c".into()]],
          locked: false,
          priority: 2,
        },
      ],
    }];
    ui.selected_game_id = Some("game".into());
    ui.selected_action = Some("first".into());
    ui.active = ActivePanel::Keys;

    ui.move_selection(1);
    assert_eq!(ui.selected_action.as_deref(), Some("last"));
    ui.selected_action = Some("locked".into());
    assert!(ui.activate_slot(0).is_none());
    assert!(!ui.is_capturing());
  }

  #[test]
  fn game_and_action_rows_are_drawn_into_their_scroll_boxes() {
    let hit_area = HitAreaService::new();
    let text_input = TextInputService::new();
    let scroll_box = ScrollBoxService::new();
    let mut ui = GameKeyBindingsUi::init(&hit_area, &text_input, &scroll_box);
    ui.games = vec![GameBindingEntry {
      id: "sample".into(),
      title: "Sample Game".into(),
      rows: vec![GameBindingRow {
        action: "jump".into(),
        description: "Jump".into(),
        keys: vec![vec!["space".into()]],
        locked: false,
        priority: 0,
      }],
    }];
    ui.selected_game_id = Some("sample".into());
    ui.selected_action = Some("jump".into());

    let mut layout = LayoutService::new();
    layout.resize_physical(120, 40);
    layout.set_developer_viewport(Rect {
      x: 0,
      y: 0,
      width: 120,
      height: 40,
    });
    let i18n = I18nService::new();
    ui.prepare_surfaces(&layout, &i18n, &text_input, &scroll_box);

    let mut canvas = CanvasService::new();
    ui.objects.begin_render();
    ui.objects.prepare_canvas(&mut canvas, &layout);
    ui.render(
      &mut RenderService::new(),
      &mut canvas,
      &layout,
      &i18n,
      &hit_area,
      &text_input,
      &scroll_box,
    );
    let frame = crate::host_engine::services::FrameCompositor::new().compose(&canvas);
    let rendered = (0..frame.height())
      .flat_map(|y| (0..frame.width()).map(move |x| (x, y)))
      .filter_map(|(x, y)| match frame.get(x, y) {
        Some(crate::host_engine::services::ComposedCell::Text(cell)) => Some(cell.text.as_str()),
        _ => None,
      })
      .collect::<String>();

    assert!(rendered.contains("Sample Game"));
    assert!(rendered.contains("Jump"));
  }

  #[test]
  fn game_description_resolves_user_and_package_default_keys_separately() {
    let hit_area = HitAreaService::new();
    let text_input = TextInputService::new();
    let scroll_box = ScrollBoxService::new();
    let mut ui = GameKeyBindingsUi::init(&hit_area, &text_input, &scroll_box);
    ui.profile.default.games.insert(
      "sample".into(),
      BTreeMap::from([("jump".into(), vec![vec!["space".into()]])]),
    );
    ui.profile.user.games.insert(
      "sample".into(),
      BTreeMap::from([("jump".into(), vec![vec!["j".into()]])]),
    );
    let game = GameBindingEntry {
      id: "sample".into(),
      title: "Sample".into(),
      rows: vec![],
    };
    let row = GameBindingRow {
      action: "jump".into(),
      description: "f%{key:jump}/{key_default:jump} Jump".into(),
      keys: vec![vec!["j".into()]],
      locked: false,
      priority: 0,
    };

    assert_eq!(ui.visible_description(&game, &row), "[J]/[Space] Jump");
  }
}
