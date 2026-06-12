mod canvas;
mod game;
mod i18n;
mod image;
mod input;
mod layout;
mod log;
mod lua;
mod overlay;
mod package;
mod render;
mod rich_text;
mod storage;
mod terminal;
mod terminal_capabilities;
mod terminal_detector;
pub mod text_layout;
mod ui;
mod unicode;

pub use canvas::CanvasService;
pub use game::GameService;
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{DrawImageParams, ImageCellRect, ImageError, ImageFit, ImageService};
pub use input::{
  translate_action_map, ActionMapEntry, InputActionEvent, InputService, KeyState, MouseButton,
  MouseEvent, MouseEventKind, SystemEvent,
};
pub use layout::{LayoutService, Rect};
pub use log::{LogService, LogSource};
pub use lua::LuaService;
pub use overlay::OverlayService;
pub use package::PackageService;
pub use render::RenderService;
pub use rich_text::{RichTextParams, RichTextService, TerminalColor, TextColor, TextStyle};
pub use storage::StorageService;
pub use terminal::TerminalService;
pub use terminal_capabilities::ImageProtocol;
pub use terminal_detector::{DetectionResult, TerminalDetector};
pub use text_layout::DrawTextParams;
pub use ui::UiService;
pub use unicode::UnicodeService;

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
  pub layout: LayoutService,
  pub image: ImageService,
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
      layout: LayoutService::new(),
      image: ImageService::new(ImageProtocol::None),
    }
  }
}
