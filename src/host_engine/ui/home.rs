mod settings;
mod game_list;
mod about;

pub use settings::SettingsUi;
pub use game_list::GameListUi;
pub use about::AboutUi;

use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, InputActionEvent, LayoutService, RenderService,
};

const LOGO_LINES: &[&str] = &[
  "████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗",
  "╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝",
  "   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  ",
  "   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ",
  "   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗",
  "   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝",
];

/// 布局计算结果 —— 把定位从绘制里拆出来
struct HomeLayout {
  logo_x: u16,
  logo_y: u16,
  menu_x: u16,
  menu_y: u16,
  version_x: u16,
  version_y: u16,
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
        action: "home.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
    ]
  }

  // ── 输入处理 ──

  pub fn handle_event(
    &mut self,
    event: &InputActionEvent,
  ) -> Option<HomeUiCommand> {
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
  ) {
    let positions = self.compute_positions(canvas.size(), layout);
    self.draw_content(render, canvas, &positions);
  }

  // ── 内部辅助 ──

  fn confirm_selected(&self) -> HomeUiCommand {
    match self.selected_index {
      0 => HomeUiCommand::StartGame,
      1 => HomeUiCommand::ContinueGame,
      2 => HomeUiCommand::OpenSettings,
      3 => HomeUiCommand::OpenAbout,
      _ => HomeUiCommand::Exit,
    }
  }

  fn menu_lines(&self) -> Vec<String> {
    let items = [
      ("[1]", "Start Game"),
      ("[2]", "Continue Game"),
      ("[3]", "Settings"),
      ("[4]", "About"),
      ("[ESC]", "Exit"),
    ];

    items
      .iter()
      .enumerate()
      .map(|(index, (key, label))| {
        if index == self.selected_index {
          format!("> {} {}", key, label)
        } else {
          format!("  {} {}", key, label)
        }
      })
      .collect()
  }

  /// 纯定位计算 —— 不碰画布，不碰绘制
  fn compute_positions(
    &self,
    canvas_size: (u16, u16),
    layout: &LayoutService,
  ) -> HomeLayout {
    let (canvas_width, canvas_height) = canvas_size;

    let logo = LOGO_LINES.join("\n");
    let (logo_width, logo_height) = layout.measure_size(&logo);

    let menu = self.menu_lines().join("\n");
    let (menu_width, menu_height) = layout.measure_size(&menu);

    let version = env!("CARGO_PKG_VERSION");
    let version_width = layout.measure_width(version);

    // 整体垂直居中
    let total_height = logo_height
      .saturating_add(2)
      .saturating_add(menu_height)
      .saturating_add(2)
      .saturating_add(1);

    let start_y = layout.center_y(canvas_height, total_height);

    let logo_x = layout.center_x(canvas_width, logo_width);
    let logo_y = start_y;

    let menu_x = layout.center_x(canvas_width, menu_width);
    let menu_y = logo_y
      .saturating_add(logo_height)
      .saturating_add(2);

    let version_x = layout.center_x(canvas_width, version_width);
    let version_y = menu_y
      .saturating_add(menu_height)
      .saturating_add(2);

    HomeLayout {
      logo_x,
      logo_y,
      menu_x,
      menu_y,
      version_x,
      version_y,
    }
  }

  /// 纯绘制 —— 不计算位置
  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &HomeLayout,
  ) {
    let logo = LOGO_LINES.join("\n");
    render.draw_text_block(canvas, positions.logo_x, positions.logo_y, &logo);

    let menu = self.menu_lines().join("\n");
    render.draw_text_block(canvas, positions.menu_x, positions.menu_y, &menu);

    let version = env!("CARGO_PKG_VERSION");
    render.draw_text(canvas, positions.version_x, positions.version_y, version);
  }
}
