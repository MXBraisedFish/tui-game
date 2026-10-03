//! Key bindings page state, user commands, and terminal-cell presentation.

mod game;
mod global;

pub use game::{GameKeyBindingsCommand, GameKeyBindingsUi};
pub use global::{GlobalKeyBindingsCommand, GlobalKeyBindingsUi};

use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RichTextService, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService,
  TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const MENU_LEN: usize = 2;
const MENU_KEYS: [&str; MENU_LEN] = ["key_bindings.global", "key_bindings.game"];

/// An application request produced by key bindings interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyBindingsCommand {
  /// A request to back.
  Back,
  /// A request to open global.
  OpenGlobal,
  /// A request to open game.
  OpenGame,
}

/// The state and owned widgets of the key bindings view.
pub struct KeyBindingsUi {
  selected: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  global: GlobalKeyBindingsUi,
  game: GameKeyBindingsUi,
}

impl UiObjectPoolOwner for KeyBindingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for KeyBindingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl KeyBindingsUi {
  /// Create the key bindings view and allocate its owned UI objects.
  ///
  /// # Arguments
  ///
  /// * `hit_area` - The hit area.
  /// * `text_input` - The text input.
  /// * `scroll_box` - The scroll box.
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      global: GlobalKeyBindingsUi::init(hit_area),
      game: GameKeyBindingsUi::init(hit_area, text_input, scroll_box),
    }
  }

  /// Return mutable access to the owned global.
  pub fn global_mut(&mut self) -> &mut GlobalKeyBindingsUi {
    &mut self.global
  }

  /// Return mutable access to the owned game.
  pub fn game_mut(&mut self) -> &mut GameKeyBindingsUi {
    &mut self.game
  }

  /// Return the shortcuts currently enabled by the key bindings view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action("key_bindings.focus_up", "up", "Focus previous option"),
      action("key_bindings.focus_down", "down", "Focus next option"),
      action("key_bindings.confirm", "enter", "Confirm selected option"),
      action("key_bindings.list.back", "esc", "Back to settings"),
      action("key_bindings.focus_global", "1", "Focus global bindings"),
      action("key_bindings.focus_game", "2", "Focus game bindings"),
    ]
  }

  /// Interpret a key bindings UI event and return the requested application command.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<KeyBindingsCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.selected = self.menu_areas.iter().position(|area| area == id)?;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.selected = self.menu_areas.iter().position(|area| area == id)?;
        self.activate()
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(KeyBindingsCommand::Back),
      UiEvent::Action(action) if action.state == KeyState::Pressed => {
        match action.action.as_str() {
          "key_bindings.focus_up" => {
            self.selected = (self.selected + MENU_LEN - 1) % MENU_LEN;
            None
          }
          "key_bindings.focus_down" => {
            self.selected = (self.selected + 1) % MENU_LEN;
            None
          }
          "key_bindings.focus_global" => {
            self.selected = 0;
            None
          }
          "key_bindings.focus_game" => {
            self.selected = 1;
            None
          }
          "key_bindings.confirm" => self.activate(),
          "key_bindings.list.back" => Some(KeyBindingsCommand::Back),
          _ => None,
        }
      }
      _ => None,
    }
  }

  fn activate(&self) -> Option<KeyBindingsCommand> {
    Some(if self.selected == 0 {
      KeyBindingsCommand::OpenGlobal
    } else {
      KeyBindingsCommand::OpenGame
    })
  }

  /// Advance the key bindings view's transient state for this host frame.
  pub fn update(&mut self, _dt: Duration) {}

  /// Draw the key bindings view and register interaction regions in its assigned surfaces.
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
    let title = i18n.get_runtime_text("key_bindings", "key_bindings.title");
    let title_width = layout.get_text_width(&title, None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_width, 0));
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y: viewport.y.saturating_add(1),
        text: format!("f%<fg:bright_magenta>{title}</fg>"),
        bold: true,
        ..Default::default()
      },
    );

    let params = RichTextParams::from_action_map(&Self::action_map(), "key_bindings.");
    let hint_lines = wrap_hint_items(
      [
        "key_bindings.action.focus",
        "key_bindings.action.select",
        "key_bindings.action.confirm",
        "key_bindings.action.list.back",
      ]
      .into_iter()
      .map(|key| i18n.get_runtime_text("key_bindings", key)),
      &params,
      viewport.width,
    );
    let hint_height = hint_lines.len().max(1) as u16;
    let content_top = viewport.y.saturating_add(3);
    let content_bottom = viewport
      .y
      .saturating_add(viewport.height.saturating_sub(hint_height));
    let menu_y = content_top.saturating_add(
      content_bottom
        .saturating_sub(content_top)
        .saturating_sub(MENU_LEN as u16)
        / 2,
    );
    for (index, key) in MENU_KEYS.iter().enumerate() {
      let label = i18n.get_runtime_text("key_bindings", key);
      let text = if index == self.selected {
        format!("f%<fg:bright_cyan>❯ {label} ❮</fg>")
      } else {
        label
      };
      let width = layout.get_text_width(&text, None);
      let x = viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0));
      let rect = Rect {
        x,
        y: menu_y.saturating_add(index as u16),
        width,
        height: 1,
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x,
          y: rect.y,
          text,
          ..Default::default()
        },
      );
      hit_area.render_host(&mut self.objects, self.menu_areas[index], rect, canvas);
    }

    for (index, line) in hint_lines.iter().enumerate() {
      let text = format!("f%<fg:rgb(85,87,83)>{line}</fg>");
      let width = layout.get_text_width(&text, Some(&params));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: viewport
            .x
            .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0)),
          y: viewport
            .y
            .saturating_add(viewport.height.saturating_sub(hint_height))
            .saturating_add(index as u16),
          text,
          params: Some(params.clone()),
          max_width: Some(viewport.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
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

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
    priority: 0,
  }
}
