use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const MENU_LEN: usize = 5;
const NS: &str = "storage_management_clear";

const MENU_KEYS: &[&str] = &[
  "storage_management.clear.cache",
  "storage_management.clear.log",
  "storage_management.clear.mod",
  "storage_management.clear.profile",
  "storage_management.clear.data",
];

pub struct StorageManagementClearUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
}

pub(crate) struct StorageManagementClearLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; MENU_LEN],
  hint_x: u16,
  hint_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageManagementClearCommand {
  Back,
  ClearCache,
  ClearLog,
  ClearMod,
  ClearProfile,
  ClearData,
}

impl UiObjectPoolOwner for StorageManagementClearUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for StorageManagementClearUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl StorageManagementClearUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "storage_management_clear.focus_up".to_string(),
        description: "Focus previous clear option".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_down".to_string(),
        description: "Focus next clear option".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.confirm".to_string(),
        description: "Confirm selected clear option".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.back".to_string(),
        description: "Back to storage management".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_cache".to_string(),
        description: "Focus clear cache".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_log".to_string(),
        description: "Focus clear log".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_mod".to_string(),
        description: "Focus clear mod".to_string(),
        keys: vec![vec!["3".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_profile".to_string(),
        description: "Focus clear profile".to_string(),
        keys: vec![vec!["4".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_clear.focus_data".to_string(),
        description: "Focus clear data".to_string(),
        keys: vec![vec!["5".to_string()]],
      },
    ]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<StorageManagementClearCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        Some(self.confirm_selected())
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(StorageManagementClearCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "storage_management_clear.focus_up" => {
          self.focus_previous();
          None
        }
        "storage_management_clear.focus_down" => {
          self.focus_next();
          None
        }
        "storage_management_clear.confirm" => Some(self.confirm_selected()),
        "storage_management_clear.back" => Some(StorageManagementClearCommand::Back),
        "storage_management_clear.focus_cache" => {
          self.selected_index = 0;
          None
        }
        "storage_management_clear.focus_log" => {
          self.selected_index = 1;
          None
        }
        "storage_management_clear.focus_mod" => {
          self.selected_index = 2;
          None
        }
        "storage_management_clear.focus_profile" => {
          self.selected_index = 3;
          None
        }
        "storage_management_clear.focus_data" => {
          self.selected_index = 4;
          None
        }
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<StorageManagementClearCommand> {
    let _ = dt;
    None
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, &positions, i18n);
    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    for (id, rect) in self.menu_areas.into_iter().zip(positions.menu_item_rects) {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  fn compute_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> StorageManagementClearLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "storage_management_clear.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let title_y = viewport.y.saturating_add(1);

    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; MENU_LEN] = std::array::from_fn(|i| {
      viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        menu_item_widths[i],
        0,
      ))
    });

    let hint = self.hint(i18n);
    let hint_w = layout.get_text_width(&hint, Some(&params));
    let hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0));
    let hint_y = viewport
      .y
      .saturating_add(layout.developer_height().saturating_sub(1));
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > MENU_LEN as u16 {
      title_y
        .saturating_add(1)
        .saturating_add((available - MENU_LEN as u16) / 2)
    } else {
      title_y.saturating_add(1)
    };

    StorageManagementClearLayout {
      title_x,
      title_y,
      menu_item_rects: std::array::from_fn(|i| Rect {
        x: menu_item_xs[i],
        y: menu_y.saturating_add(i as u16),
        width: menu_item_widths[i],
        height: 1,
      }),
      hint_x,
      hint_y,
    }
  }

  fn confirm_selected(&self) -> StorageManagementClearCommand {
    match self.selected_index {
      0 => StorageManagementClearCommand::ClearCache,
      1 => StorageManagementClearCommand::ClearLog,
      2 => StorageManagementClearCommand::ClearMod,
      3 => StorageManagementClearCommand::ClearProfile,
      _ => StorageManagementClearCommand::ClearData,
    }
  }

  fn focus_previous(&mut self) {
    self.selected_index = if self.selected_index == 0 {
      MENU_LEN - 1
    } else {
      self.selected_index - 1
    };
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % MENU_LEN;
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; MENU_LEN] {
    std::array::from_fn(|i| {
      let label = i18n.get_runtime_text(NS, MENU_KEYS[i]);
      if i == self.selected_index {
        format!("f%<fg:bright_cyan>❯ {} ❮</fg>", label)
      } else {
        label
      }
    })
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "storage_management_clear.action.focus"),
      i18n.get_runtime_text(NS, "storage_management_clear.action.select"),
      i18n.get_runtime_text(NS, "storage_management_clear.action.back"),
    )
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "storage_management_clear.")
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &StorageManagementClearLayout,
    i18n: &I18nService,
  ) {
    let title = i18n.get_runtime_text(NS, "storage_management_clear.title");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta>{}</fg>", title),
        bold: true,
        ..Default::default()
      },
    );

    for (i, item) in self.menu_items(i18n).iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_rects[i].x,
          y: positions.menu_item_rects[i].y,
          text: item.clone(),
          ..Default::default()
        },
      );
    }

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: self.hint(i18n),
        params: Some(self.build_key_params()),
        ..Default::default()
      },
    );
  }
}
