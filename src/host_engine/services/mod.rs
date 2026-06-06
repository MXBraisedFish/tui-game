// TODO(render):
// Remove transitional RenderService after Canvas fully replaces old renderer.

mod canvas;
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
mod unicode;

pub use canvas::{CanvasBuffer, CanvasCell, CanvasCellContent, CanvasService, CanvasStyle};
pub use game::{GameService, GameSessionState};
pub use i18n::{I18nService, LanguageInfo, LanguageRegistryEntry};
pub use input::{
  InputActionEvent,
  InputEventType,
  InputService,
  Key,
  KeyBinding,
  KeyEvent,
  KeyEventKind,
  KeyPattern,
  KeyState,
};
pub use log::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};
pub use lua::LuaService;
pub use overlay::{OverlayKind, OverlayService};
pub use package::PackageService;
pub use render::RenderService;
pub use rich_text::{RichText, RichTextService, TerminalColor, TextColor, TextStyle};
pub use storage::StorageService;
pub use terminal::TerminalService;
pub use terminal_capabilities::{ImageProtocol, TerminalCapabilities};
pub use ui::UiService;
pub use unicode::{UnicodeService, char_width, display_width, rich_text_width};

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
  pub unicode: UnicodeService,
  pub canvas: CanvasService,
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
      unicode: UnicodeService::new(),
      canvas: CanvasService::new(),
    }
  }
}
