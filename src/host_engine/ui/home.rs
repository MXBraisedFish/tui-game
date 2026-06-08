mod settings;
mod game_list;
mod about;

pub use settings::SettingsUi;
pub use game_list::GameListUi;
pub use about::AboutUi;

use std::time::Duration;

use crate::host_engine::services::{ActionMapEntry, InputActionEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeUi {
  selected_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeUiCommand {
  OpenGameList,
  OpenSettings,
  OpenAbout,
  Shutdown,
}

impl HomeUi {
  pub fn init() -> Self {
    Self { selected_index: 0 }
  }

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<HomeUiCommand> {
    let _ = event;
    None
  }

  pub fn update(&mut self, dt: Duration) -> Option<HomeUiCommand> {
    let _ = dt;
    None
  }

  pub fn render(&self) {}

  pub fn action_map() -> Vec<ActionMapEntry> {
    Vec::new()
  }
}
