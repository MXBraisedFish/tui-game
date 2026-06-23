mod canvas;
mod clipboard;
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
mod render_pipeline;
mod rich_text;
mod storage;
mod terminal;
mod terminal_capabilities;
mod text_input;
pub mod text_layout;
mod ui;
mod unicode;

pub use canvas::{CanvasCell, CanvasService};
pub use clipboard::ClipboardService;
pub use game::GameService;
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{ImageConvertParams, ImageService};
pub use input::{
  ActionMapEntry, InputActionEvent, InputEventType, InputService, Key, KeyEventKind, KeyState,
  MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, SystemEvent, TerminalKeyCode,
  TerminalKeyEvent, translate_action_map,
};
pub use layout::{LayoutService, Rect};
pub use log::{LogService, LogSource};
pub use lua::LuaService;
pub use overlay::OverlayService;
pub use package::PackageService;
pub use render::{BorderStyle, RenderService};
pub use render_pipeline::{FrameCompositor, FramePresenter};
pub use rich_text::{RichTextParams, RichTextService, TerminalColor, TextColor, TextStyle};
pub use storage::StorageService;
pub use terminal::TerminalService;
pub use text_input::{
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService, VerticalAlign,
};
pub use text_layout::DrawTextParams;
pub use ui::{UiObjectPool, UiObjectPoolOwner, UiService};
pub use unicode::UnicodeService;

pub struct EngineServices {
  pub package: PackageService,
  pub clipboard: ClipboardService,
  pub input: InputService,
  pub ui: UiService,
  pub game: GameService,
  pub image: ImageService,
  pub overlay: OverlayService,
  pub storage: StorageService,
  pub lua: LuaService,
  pub render: RenderService,
  pub terminal: TerminalService,
  pub text_input: TextInputService,
  pub log: LogService,
  pub i18n: I18nService,
  pub rich_text: RichTextService,
  pub unicode: UnicodeService,
  pub canvas: CanvasService,
  pub layout: LayoutService,
  pub compositor: FrameCompositor,
  pub presenter: FramePresenter,
}

impl EngineServices {
  pub fn new() -> Self {
    let mut log = LogService::new();

    Self {
      terminal: TerminalService::new(),
      clipboard: ClipboardService::new(),
      text_input: TextInputService::new(),
      package: PackageService::new(),
      input: InputService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      image: ImageService::new(),
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
      compositor: FrameCompositor::new(),
      presenter: FramePresenter::new(),
    }
  }
}
