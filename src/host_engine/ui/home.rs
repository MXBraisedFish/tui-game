mod settings;
mod game_list;
mod about;

pub use settings::SettingsUi;
pub use game_list::GameListUi;
pub use about::AboutUi;

pub struct MainUi;

impl MainUi {
    pub fn handle_event(&mut self) {
    }

    pub fn update(&mut self) {
    }

    pub fn render(&self) {
    }

    pub fn action_map(&self) {
    }
}
