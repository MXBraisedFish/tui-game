mod game;
mod i18n;
mod input;
mod log;
mod lua;
mod overlay;
mod package;
mod render;
mod rich_text;
mod storage;
mod terminal;
mod terminal_capabilities;
mod ui;

pub use game::{GameService, GameSessionState};
pub use i18n::{I18nService, LanguageInfo, LanguageRegistryEntry};
pub use input::{InputEvent, InputService, KeyEventKind, KeyInput};
pub use log::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};
pub use lua::LuaService;
pub use overlay::{OverlayKind, OverlayService};
pub use package::PackageService;
pub use render::RenderService;
pub use rich_text::RichTextService;
pub use storage::StorageService;
pub use terminal::TerminalService;
pub use terminal_capabilities::{ImageProtocol, TerminalCapabilities};
pub use ui::UiService;

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
  pub log: LogService,
  pub i18n: I18nService,
  pub rich_text: RichTextService,
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
      log,
      i18n: I18nService::new(),
      rich_text: RichTextService::new(),
    }
  }
}
