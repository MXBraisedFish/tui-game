use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub mod language;
pub mod mods;

const SETTINGS_MENU_LEN: usize = 6;

const MENU_KEYS: &[&str] = &[
  "settings.language",
  "settings.key_bindings",
  "settings.mod",
  "settings.storage_management",
  "settings.security_settings",
  "settings.display_settings",
];

/// 设置页面布局信息。
pub(crate) struct SettingsLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; SETTINGS_MENU_LEN],
  action_hint_x: u16,
  action_hint_y: u16,
}

/// 设置页面 UI：包含菜单导航和操作提示。
pub struct SettingsUi {
  selected_index: usize,
  objects: UiObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; SETTINGS_MENU_LEN],
}

impl UiObjectPoolOwner for SettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

/// 设置页面的命令。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsUiCommand {
  Back,
  OpenLanguageSelect,
  OpenMods,
}

impl SettingsUi {
  /// 初始化设置页面 UI。
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
    }
  }

  /// 返回设置页面的按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "settings.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "settings.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "settings.back".to_string(),
        description: "Go back to home".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_language".to_string(),
        description: "Focus language option".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_key_bindings".to_string(),
        description: "Focus key bindings option".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_mod".to_string(),
        description: "Focus mod option".to_string(),
        keys: vec![vec!["3".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_storage_management".to_string(),
        description: "Focus storage management option".to_string(),
        keys: vec![vec!["4".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_security_settings".to_string(),
        description: "Focus security settings option".to_string(),
        keys: vec![vec!["5".to_string()]],
      },
      ActionMapEntry {
        action: "settings.focus_display_settings".to_string(),
        description: "Focus display settings option".to_string(),
        keys: vec![vec!["6".to_string()]],
      },
    ]
  }

  /// 处理 UI 事件，返回导航或确认命令。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<SettingsUiCommand> {
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
        match self.selected_index {
          0 => Some(SettingsUiCommand::OpenLanguageSelect),
          2 => Some(SettingsUiCommand::OpenMods),
          _ => None,
        }
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(SettingsUiCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "settings.focus_up" => {
          self.focus_previous();
          None
        }
        "settings.focus_down" => {
          self.focus_next();
          None
        }
        "settings.confirm" => match self.selected_index {
          0 => Some(SettingsUiCommand::OpenLanguageSelect),
          2 => Some(SettingsUiCommand::OpenMods),
          _ => None,
        },
        "settings.back" => Some(SettingsUiCommand::Back),
        "settings.focus_language" => {
          self.selected_index = 0;
          None
        }
        "settings.focus_key_bindings" => {
          self.selected_index = 1;
          None
        }
        "settings.focus_mod" => {
          self.selected_index = 2;
          None
        }
        "settings.focus_storage_management" => {
          self.selected_index = 3;
          None
        }
        "settings.focus_security_settings" => {
          self.selected_index = 4;
          None
        }
        "settings.focus_display_settings" => {
          self.selected_index = 5;
          None
        }

        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<SettingsUiCommand> {
    let _ = dt;
    None
  }

  /// 渲染设置页面到宿主层。
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

  /// 根据布局服务计算设置页面各元素的宿主坐标。
  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> SettingsLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text("settings", "settings.title");
    let title_w = layout.get_text_width(&format!("f%<b>{}<b>", title), None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let title_y = viewport.y.saturating_add(1);
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; SETTINGS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; SETTINGS_MENU_LEN] = std::array::from_fn(|i| {
      viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        menu_item_widths[i],
        0,
      ))
    });
    let menu_height = SETTINGS_MENU_LEN as u16;
    let action_hint = format!(
      "f%<fg:bright_black>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    let action_hint_w = layout.get_text_width(&action_hint, Some(&params));
    let action_hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, action_hint_w, 0));
    let action_hint_y = viewport
      .y
      .saturating_add(layout.developer_height().saturating_sub(1));
    let available = action_hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > menu_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - menu_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let menu_item_rects: [Rect; SETTINGS_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    SettingsLayout {
      title_x,
      title_y,
      menu_item_rects,
      action_hint_x,
      action_hint_y,
    }
  }

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = SETTINGS_MENU_LEN - 1;
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % SETTINGS_MENU_LEN;
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; SETTINGS_MENU_LEN] {
    std::array::from_fn(|i| {
      let label = i18n.get_runtime_text("settings", MENU_KEYS[i]);
      if i == self.selected_index {
        format!("f%<fg:bright_cyan>❯ {} ❮</fg>", label)
      } else {
        label
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "settings.")
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &SettingsLayout,
    i18n: &I18nService,
  ) {
    let title = i18n.get_runtime_text("settings", "settings.title");
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
    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
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
    let params = self.build_key_params();
    let action_hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.action_hint_x,
        y: positions.action_hint_y,
        text: action_hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }
}
