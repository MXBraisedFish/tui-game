use std::collections::HashMap;
use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, I18nService, InputActionEvent, KeyState,
  LayoutService, MouseButton, MouseEvent, MouseEventKind, Rect, RenderService, RichTextParams,
};

const MODS_MENU_LEN: usize = 2;

const MENU_KEYS: &[&str] = &["mods.game", "mods.screensaver"];

/// 布局计算结果
pub(crate) struct ModsLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; MODS_MENU_LEN],
  hint_x: u16,
  hint_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModsUi {
  selected_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModsCommand {
  OpenGame,
  OpenScreensaver,
  Back,
}

impl ModsUi {
  pub fn init() -> Self {
    Self { selected_index: 0 }
  }

  // ── 输入绑定 ──

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "mods.focus_game".to_string(),
        description: "Focus game pack option".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "mods.focus_screensaver".to_string(),
        description: "Focus screensaver pack option".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "mods.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "mods.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "mods.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "mods.back".to_string(),
        description: "Go back to settings".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  // ── 输入处理 ──

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<ModsCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "mods.focus_game" => {
        self.selected_index = 0;
        None
      }
      "mods.focus_screensaver" => {
        self.selected_index = 1;
        None
      }
      "mods.focus_up" => {
        self.focus_previous();
        None
      }
      "mods.focus_down" => {
        self.focus_next();
        None
      }
      "mods.confirm" => Some(self.confirm_selected()),
      "mods.back" => Some(ModsCommand::Back),
      _ => None,
    }
  }

  pub fn handle_mouse_event(
    &mut self,
    event: &MouseEvent,
    positions: &ModsLayout,
  ) -> Option<ModsCommand> {
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
            return Some(self.confirm_selected());
          }
          None
        }
        Some(MouseButton::Right) => Some(ModsCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<ModsCommand> {
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

  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> ModsLayout {
    let params = self.build_key_params();

    // title —— 距离顶部 1 行，水平居中
    let title = i18n.get_runtime_text("mods", "mods.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x = layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0);
    let title_y: u16 = 1;

    // 菜单项
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; MODS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; MODS_MENU_LEN] = std::array::from_fn(|i| {
      layout.resolve_x(LayoutService::ALIGN_CENTER, menu_item_widths[i], 0)
    });
    let menu_height = MODS_MENU_LEN as u16;

    // 操作提示（底部）
    let hint = format!(
      "f%<fg:bright_black>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("mods", "mods.action.focus"),
      i18n.get_runtime_text("mods", "mods.action.select"),
      i18n.get_runtime_text("mods", "mods.action.confirm"),
      i18n.get_runtime_text("mods", "mods.action.back"),
    );
    let hint_w = layout.get_text_width(&hint, Some(&params));
    let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);
    let terminal_height = layout.get_terminal_size().height;
    let hint_y = terminal_height.saturating_sub(1);

    // 菜单垂直居中（在 title 和 hint 之间）
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > menu_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - menu_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let menu_item_rects: [Rect; MODS_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    ModsLayout {
      title_x,
      title_y,
      menu_item_rects,
      hint_x,
      hint_y,
    }
  }

  // ── 内部辅助 ──

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = MODS_MENU_LEN - 1;
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % MODS_MENU_LEN;
  }

  fn confirm_selected(&self) -> ModsCommand {
    match self.selected_index {
      0 => ModsCommand::OpenGame,
      _ => ModsCommand::OpenScreensaver,
    }
  }

  fn hit_test_menu(positions: &ModsLayout, x: u16, y: u16) -> Option<usize> {
    positions
      .menu_item_rects
      .iter()
      .position(|rect| rect.contains(x, y))
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; MODS_MENU_LEN] {
    std::array::from_fn(|i| {
      let label = i18n.get_runtime_text("mods", MENU_KEYS[i]);
      if i == self.selected_index {
        format!("f%<fg:bright_cyan>❯ {} ❮</fg>", label)
      } else {
        label
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    // 基础：from_action_map 生成 mods.xxx 和短别名 xxx
    let base = RichTextParams::from_action_map(&Self::action_map(), "mods.");
    // 语言文件里 {key:...} 用的是 settings.xxx 前缀，桥接过去
    let mut key_actions = base.key_actions;
    let aliases: &[(&str, &str)] = &[
      ("settings.focus_game", "mods.focus_game"),
      ("settings.screensaver", "mods.focus_screensaver"),
      ("settings.focus_up", "mods.focus_up"),
      ("settings.focus_down", "mods.focus_down"),
      ("settings.confirm", "mods.confirm"),
      ("settings.back", "mods.back"),
    ];
    for &(alias, action) in aliases {
      if let Some(keys) = key_actions.get(action) {
        key_actions.insert(alias.to_string(), keys.clone());
      }
    }
    RichTextParams {
      values: HashMap::new(),
      key_actions,
    }
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &ModsLayout,
    i18n: &I18nService,
  ) {
    // title
    let title = i18n.get_runtime_text("mods", "mods.title");
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
    let hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("mods", "mods.action.focus"),
      i18n.get_runtime_text("mods", "mods.action.select"),
      i18n.get_runtime_text("mods", "mods.action.confirm"),
      i18n.get_runtime_text("mods", "mods.action.back"),
    );
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }
}
