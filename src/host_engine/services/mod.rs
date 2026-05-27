mod game;
mod input;
mod lua;
mod overlay;
mod package;
mod render;
mod storage;
mod ui;
mod terminal;

pub use game::{GameService, GameSessionState};
pub use input::{InputService, KeyInput};
pub use lua::LuaService;
pub use overlay::{OverlayKind, OverlayService, OverlaySessionState};
pub use package::PackageService;
pub use render::RenderService;
pub use storage::StorageService;
pub use ui::UiService;
pub use terminal::TerminalService;

pub struct EngineServices {
  pub package: PackageService,
  pub input: InputService,
  pub ui: UiService,
  pub game: GameService,
  pub overlay: OverlayService,
  pub storage: StorageService,
  pub lua: LuaService,
  pub render: RenderService,
  pub terminal: TerminalService
}

impl EngineServices {
  pub fn new() -> Self {
    Self {
      terminal: TerminalService::new(),
      package: PackageService::new(),
      input: InputService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      overlay: OverlayService::new(),
      storage: StorageService::new(),
      lua: LuaService::new(),
      render: RenderService::new(),
    }
  }
}