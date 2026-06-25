mod canvas;
mod clipboard;
mod game;
mod host_object;
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
pub mod text_layout;
mod ui;
mod unicode;
pub(crate) mod widget;

pub use canvas::{CanvasCell, CanvasService};
pub use clipboard::ClipboardService;
pub use game::GameService;
pub use host_object::{HostArea, HostAreaId, HostAreaKind, HostObjectPool};
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{ImageConvertParams, ImageService};
pub use input::{
  ActionMapEntry, InputActionEvent, InputEventType, InputService, Key, KeyEventKind, KeyState,
  MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, ScrollDirection, SystemEvent,
  TerminalKeyCode, TerminalKeyEvent, translate_action_map,
};
pub use layout::{LayoutService, Rect, Size};
pub use log::{LogService, LogSource};
pub use lua::LuaService;
pub use overlay::OverlayService;
pub use package::PackageService;
pub use render::{BorderStyle, RenderService};
pub use render_pipeline::{FrameCompositor, FramePresenter};
pub use rich_text::{RichTextParams, RichTextService, TerminalColor, TextColor, TextStyle};
pub use storage::StorageService;
pub use terminal::TerminalService;
pub use text_layout::DrawTextParams;
pub use ui::{UiEvent, UiObjectPool, UiObjectPoolOwner, UiService};
pub use unicode::UnicodeService;
pub use widget::{
  HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, Overflow, ScrollBoxEvent, ScrollBoxId,
  ScrollBoxOptions, ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarSide,
  ScrollbarStyle, ScrollbarVisibility, SliceId, SliceLength, SliceOptions, SliceRect, SliceService,
  SurfaceId, TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService, VerticalAlign,
};

/// 引擎核心服务集合，持有所有子服务的实例
pub struct EngineServices {
  pub package: PackageService,
  pub clipboard: ClipboardService,
  pub host_objects: HostObjectPool,
  pub hit_area: HitAreaService,
  pub scroll_box: ScrollBoxService,
  pub slice: SliceService,
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
    let storage = StorageService::new(&mut log);
    let image_cache_dir = storage.path("data/cache/images");

    Self {
      terminal: TerminalService::new(),
      clipboard: ClipboardService::new(),
      host_objects: HostObjectPool::new(),
      hit_area: HitAreaService::new(),
      scroll_box: ScrollBoxService::new(),
      slice: SliceService::new(),
      text_input: TextInputService::new(),
      package: PackageService::new(),
      input: InputService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      image: ImageService::new(Some(image_cache_dir)),
      overlay: OverlayService::new(),
      storage,
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
