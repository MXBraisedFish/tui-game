mod about;
mod game_list;
mod settings;

pub(crate) use settings::SettingsLayout;
pub(crate) use settings::language::LanguageSelectLayout;
pub use settings::language::{LanguageSelectCommand, LanguageSelectUi};
pub(crate) use settings::mods::ModsLayout;
pub use settings::mods::{ModsCommand, ModsUi};
pub use settings::{SettingsUi, SettingsUiCommand};

use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, InputActionEvent, KeyState, LayoutService,
  MouseButton, MouseEvent, MouseEventKind, Rect, RenderService, RichTextParams, TextColor,
};

use crate::host_engine::services::I18nService;

const LOGO_LINES: &[&str] = &[
  "████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗",
  "╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝",
  "   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  ",
  "   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ",
  "   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗",
  "   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝",
];

const HOME_MENU_LEN: usize = 5;

/// 将纯文本 Logo 转换为富文本：
/// - `█` 块使用默认前景色（红）
/// - 制表符（╗╔╝╚║═ 等）自动包裹 `<fg:white>...</fg>`
fn style_logo(lines: &[&str]) -> String {
  let plain = lines.join("\n");
  let mut result = String::from("f%");
  let mut in_box = false;

  for ch in plain.chars() {
    let is_box = ch != '█' && ch != ' ' && ch != '\n';

    if is_box && !in_box {
      result.push_str("<fg:white>");
      in_box = true;
    } else if !is_box && in_box {
      result.push_str("</fg>");
      in_box = false;
    }

    result.push(ch);
  }

  if in_box {
    result.push_str("</fg>");
  }

  result
}

/// 布局计算结果 —— 把定位从绘制里拆出来
pub(crate) struct HomeLayout {
  logo_x: u16,
  logo_y: u16,
  menu_item_xs: [u16; HOME_MENU_LEN],
  menu_item_rects: [Rect; HOME_MENU_LEN],
  menu_y: u16,
  version_x: u16,
  version_y: u16,
  action_hint_x: u16,
  action_hint_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeUi {
  selected_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeUiCommand {
  StartGame,
  ContinueGame,
  OpenSettings,
  OpenAbout,
  Exit,
}

impl HomeUi {
  pub fn init() -> Self {
    Self { selected_index: 0 }
  }

  // ── 输入绑定 ──

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "home.focus_exit".to_string(),
        description: "Focus exit option".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_start_game".to_string(),
        description: "Focus start game option".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_continue_game".to_string(),
        description: "Focus continue game option".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_settings".to_string(),
        description: "Focus settings option".to_string(),
        keys: vec![vec!["3".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_about".to_string(),
        description: "Focus about option".to_string(),
        keys: vec![vec!["4".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "home.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "home.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
    ]
  }

  // ── 输入处理 ──

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<HomeUiCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "home.focus_exit" => {
        self.selected_index = 4;
        None
      }

      "home.focus_start_game" => {
        self.selected_index = 0;
        None
      }

      "home.focus_continue_game" => {
        self.selected_index = 1;
        None
      }

      "home.focus_settings" => {
        self.selected_index = 2;
        None
      }

      "home.focus_about" => {
        self.selected_index = 3;
        None
      }

      "home.focus_up" => {
        self.focus_previous();
        None
      }

      "home.focus_down" => {
        self.focus_next();
        None
      }

      "home.confirm" => Some(self.confirm_selected()),

      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<HomeUiCommand> {
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

  // ── 内部辅助 ──

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = HOME_MENU_LEN - 1;
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % HOME_MENU_LEN;
  }

  fn confirm_selected(&self) -> HomeUiCommand {
    match self.selected_index {
      0 => HomeUiCommand::StartGame,
      1 => HomeUiCommand::ContinueGame,
      2 => HomeUiCommand::OpenSettings,
      3 => HomeUiCommand::OpenAbout,
      _ => HomeUiCommand::Exit,
    }
  }

  /// 处理鼠标事件：hover 自动聚焦，左键点击确认。
  pub fn handle_mouse_event(
    &mut self,
    event: &MouseEvent,
    positions: &HomeLayout,
  ) -> Option<HomeUiCommand> {
    match event.kind {
      MouseEventKind::Move | MouseEventKind::Hold => {
        if let Some(index) = Self::hit_test_menu(positions, event.x, event.y) {
          self.selected_index = index;
        }
        None
      }
      MouseEventKind::Press => {
        if event.button == Some(MouseButton::Left) {
          if let Some(index) = Self::hit_test_menu(positions, event.x, event.y) {
            self.selected_index = index;
            return Some(self.confirm_selected());
          }
        }
        None
      }
      _ => None,
    }
  }

  /// 命中测试：返回鼠标坐标命中的菜单项索引（None 表示未命中任何项）。
  fn hit_test_menu(positions: &HomeLayout, x: u16, y: u16) -> Option<usize> {
    positions
      .menu_item_rects
      .iter()
      .position(|rect| rect.contains(x, y))
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; HOME_MENU_LEN] {
    let labels = [
      i18n.get_runtime_text("home", "home.game_list"),
      i18n.get_runtime_text("home", "home.countinue"),
      i18n.get_runtime_text("home", "home.settings"),
      i18n.get_runtime_text("home", "home.about"),
      i18n.get_runtime_text("home", "home.exit"),
    ];

    std::array::from_fn(|i| {
      if i == self.selected_index {
        let fg = if i >= 4 { "bright_red" } else { "bright_cyan" };
        format!("f%<fg:{}>❯ {} ❮</fg>", fg, labels[i])
      } else {
        labels[i].to_string()
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "home.")
  }

  /// 纯定位计算 —— 不碰画布，不碰绘制
  pub(crate) fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> HomeLayout {
    let params = self.build_key_params();

    let logo = style_logo(LOGO_LINES);
    let logo_size = layout.get_text_size(&logo, None);

    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; HOME_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    // 每个菜单项单独居中
    let menu_item_xs: [u16; HOME_MENU_LEN] = std::array::from_fn(|i| {
      layout.resolve_x(LayoutService::ALIGN_CENTER, menu_item_widths[i], 0)
    });
    let menu_height = HOME_MENU_LEN as u16;

    let version = env!("CARGO_PKG_VERSION");
    let version_width = layout.get_text_width(
      &format!("f%<fg:rgb(85,87,83)>v{}</fg>", version).to_string(),
      None,
    );

    // 操作提示（单行，三个提示用两个空格分隔，灰色）
    let action_hint = format!(
      "f%<fg:bright_black>{}  {}  {}</fg>",
      i18n.get_runtime_text("home", "home.action.focus"),
      i18n.get_runtime_text("home", "home.action.select"),
      i18n.get_runtime_text("home", "home.action.confirm"),
    );
    let action_hint_w = layout.get_text_width(&action_hint, Some(&params));

    // 整体垂直居中
    let total_height = logo_size
      .height
      .saturating_add(2)
      .saturating_add(menu_height)
      .saturating_add(2) // version height = 1
      .saturating_add(1) // gap
      .saturating_add(1); // 1 action hint line

    let start_y = layout.resolve_y(LayoutService::ALIGN_MIDDLE, total_height, 0);

    let logo_x = layout.resolve_x(LayoutService::ALIGN_CENTER, logo_size.width, 0);
    let logo_y = start_y;

    let menu_y = logo_y.saturating_add(logo_size.height).saturating_add(2);

    let menu_item_rects: [Rect; HOME_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    let version_x = layout.resolve_x(LayoutService::ALIGN_CENTER, version_width, 0);
    let version_y = menu_y.saturating_add(menu_height).saturating_add(2);

    let action_hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, action_hint_w, 0);
    let action_hint_y = version_y.saturating_add(2);

    HomeLayout {
      logo_x,
      logo_y,
      menu_item_xs,
      menu_item_rects,
      menu_y,
      version_x,
      version_y,
      action_hint_x,
      action_hint_y,
    }
  }

  /// 纯绘制 —— 不计算位置
  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &HomeLayout,
    i18n: &I18nService,
  ) {
    let logo = style_logo(LOGO_LINES);
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.logo_x,
        y: positions.logo_y,
        text: logo,
        fg: Some(TextColor::Rgb {
          r: (255),
          g: (165),
          b: (0),
        }),
        ..Default::default()
      },
    );

    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_xs[i],
          y: positions.menu_y.saturating_add(i as u16),
          text: item.clone(),
          ..Default::default()
        },
      );
    }

    let version = env!("CARGO_PKG_VERSION").to_string();
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.version_x,
        y: positions.version_y,
        text: format!("f%<fg:rgb(85,87,83)>v{}</fg>", version),
        ..Default::default()
      },
    );

    // ── 操作提示（淡灰色，单行） ──
    let params = self.build_key_params();
    let action_hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}</fg>",
      i18n.get_runtime_text("home", "home.action.focus"),
      i18n.get_runtime_text("home", "home.action.select"),
      i18n.get_runtime_text("home", "home.action.confirm"),
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
