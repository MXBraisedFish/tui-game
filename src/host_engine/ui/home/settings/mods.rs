use std::collections::HashMap;
use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, UiEvent, UiObjectPool, UiObjectPoolOwner,
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

pub struct ModsUi {
  selected_index: usize,
  objects: UiObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MODS_MENU_LEN],
}

impl UiObjectPoolOwner for ModsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModsCommand {
  OpenGame,
  OpenScreensaver,
  Back,
}

impl ModsUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
    }
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

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ModsCommand> {
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
      }) => Some(ModsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
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
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, &positions, i18n);
    let terminal = layout.physical_size();
    hit_area.render_host(
      &mut self.objects,
      self.back_area,
      Rect {
        x: 0,
        y: 0,
        width: terminal.width,
        height: terminal.height,
      },
      canvas,
    );
    for (id, rect) in self.menu_areas.into_iter().zip(positions.menu_item_rects) {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> ModsLayout {
    let params = self.build_key_params();

    // title —— 距离顶部 1 行，水平居中
    let title = i18n.get_runtime_text("mods", "mods.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, title_w, 0);
    let title_y: u16 = 1;

    // 菜单项
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; MODS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; MODS_MENU_LEN] = std::array::from_fn(|i| {
      layout.resolve_host_x(LayoutService::ALIGN_CENTER, menu_item_widths[i], 0)
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
    let hint_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, hint_w, 0);
    let terminal_height = layout.physical_size().height;
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

    // 菜单项
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

    // 操作提示
    let params = self.build_key_params();
    let hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("mods", "mods.action.focus"),
      i18n.get_runtime_text("mods", "mods.action.select"),
      i18n.get_runtime_text("mods", "mods.action.confirm"),
      i18n.get_runtime_text("mods", "mods.action.back"),
    );
    render.draw_host_text(
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
