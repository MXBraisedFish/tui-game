use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, I18nService, InputActionEvent, KeyState,
  LayoutService, MouseButton, MouseEvent, MouseEventKind, Rect, RenderService, RichTextParams,
};

pub mod language;
pub(crate) use language::LanguageSelectLayout;
pub use language::{LanguageSelectCommand, LanguageSelectUi};

const SETTINGS_MENU_LEN: usize = 6;

const MENU_KEYS: &[&str] = &[
  "settings.language",
  "settings.key_bindings",
  "settings.mod",
  "settings.storage_management",
  "settings.security_settings",
  "settings.display_settings",
];

/// 布局计算结果
pub(crate) struct SettingsLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; SETTINGS_MENU_LEN],
  action_hint_x: u16,
  action_hint_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsUi {
  selected_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsUiCommand {
  Back,
  OpenLanguageSelect,
}

impl SettingsUi {
  pub fn init() -> Self {
    Self { selected_index: 0 }
  }

  // ── 输入绑定 ──

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

  // ── 输入处理 ──

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<SettingsUiCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "settings.focus_up" => {
        self.focus_previous();
        None
      }
      "settings.focus_down" => {
        self.focus_next();
        None
      }
      "settings.confirm" => {
        if self.selected_index == 0 {
          return Some(SettingsUiCommand::OpenLanguageSelect);
        }
        None
      }
      "settings.back" => Some(SettingsUiCommand::Back),

      // 数字键快速聚焦
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
    }
  }

  /// 鼠标事件：hover 聚焦、左键确认、右键返回。
  pub fn handle_mouse_event(
    &mut self,
    event: &MouseEvent,
    positions: &SettingsLayout,
  ) -> Option<SettingsUiCommand> {
    match event.kind {
      MouseEventKind::Move | MouseEventKind::Hold => {
        if let Some(index) = Self::hit_test_menu(positions, event.x, event.y) {
          self.selected_index = index;
        }
        None
      }
      MouseEventKind::Press => match event.button {
        Some(MouseButton::Left) => {
          if let Some(index) = Self::hit_test_menu(positions, event.x, event.y) {
            self.selected_index = index;
            if index == 0 {
              return Some(SettingsUiCommand::OpenLanguageSelect);
            }
          }
          None
        }
        Some(MouseButton::Right) => Some(SettingsUiCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<SettingsUiCommand> {
    let _ = dt;
    None
  }

  // ── 渲染 ──

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, &positions, i18n);
  }

  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> SettingsLayout {
    let params = self.build_key_params();

    // title —— 距离顶部 1 行，水平居中
    let title = i18n.get_runtime_text("settings", "settings.title");
    let title_w = layout.get_text_width(&format!("f%<b>{}<b>", title), None);
    let title_x = layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0);
    let title_y: u16 = 1;

    // 菜单项
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; SETTINGS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; SETTINGS_MENU_LEN] = std::array::from_fn(|i| {
      layout.resolve_x(LayoutService::ALIGN_CENTER, menu_item_widths[i], 0)
    });
    let menu_height = SETTINGS_MENU_LEN as u16;

    // 操作提示（底部）
    let action_hint = format!(
      "f%<fg:bright_black>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    let action_hint_w = layout.get_text_width(&action_hint, Some(&params));
    let action_hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, action_hint_w, 0);
    let terminal_height = layout.get_terminal_size().height;
    let action_hint_y = terminal_height.saturating_sub(1);

    // 菜单垂直居中（在 title 和 action_hint 之间）
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

  // ── 内部辅助 ──

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

  fn hit_test_menu(positions: &SettingsLayout, x: u16, y: u16) -> Option<usize> {
    positions
      .menu_item_rects
      .iter()
      .position(|rect| rect.contains(x, y))
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
    // title
    let title = i18n.get_runtime_text("settings", "settings.title");
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta>{}</fg>", title),
        bold: true,
        ..Default::default()
      },
    );

    // 菜单项
    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_rects[i].x,
          y: positions.menu_item_rects[i].y,
          text: item.clone(),
          ..Default::default()
        },
      );
    }

    // 操作提示
    let params = self.build_key_params();
    let action_hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    render.draw_text(
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
