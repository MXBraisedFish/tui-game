mod game;
mod input;
mod lua;
mod overlay;
mod package;
mod render;
mod storage;
mod ui;
mod terminal;
mod terminal_capabilities;
mod log;

pub use game::{GameService, GameSessionState};
pub use input::{InputEvent, InputService, KeyEventKind, KeyInput};
pub use lua::LuaService;
pub use overlay::{OverlayKind, OverlayService};
pub use package::PackageService;
pub use render::RenderService;
pub use storage::StorageService;
pub use ui::UiService;
pub use terminal::TerminalService;
pub use terminal_capabilities::{ImageProtocol, TerminalCapabilities};
pub use log::{LogEntry, LogLevel, LogService};

pub struct EngineServices {
  pub package: PackageService,
  pub input: InputService,
  pub ui: UiService,
  pub game: GameService,
  pub overlay: OverlayService,
  pub storage: StorageService,
  pub lua: LuaService,
  pub render: RenderService,
  pub terminal: TerminalService,
  pub log: LogService
}

impl EngineServices {
  pub fn new() -> Self {
    let mut log = LogService::new();



    Self {
      terminal: TerminalService::new(),
      package: PackageService::new(),
      input: InputService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      overlay: OverlayService::new(),
      storage: StorageService::new(&mut log),
      lua: LuaService::new(),
      render: RenderService::new(),
      log
    }
  }
}