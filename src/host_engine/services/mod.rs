pub(crate) mod animation;
mod async_runtime;
mod audio;
mod canvas;
mod event;
pub(crate) mod export;
mod file;
mod i18n;
mod image;
mod input;
mod layout;
mod log;
mod lua;
mod network;
mod package;
mod random;
mod recording;
mod render;
mod render_pipeline;
mod rich_text;
mod screenshot;
mod storage;
pub mod text_layout;
mod time;
mod ui;
mod unicode;
use tg_core_version as version;
mod video;
pub(crate) mod widget;
mod write_barrier;

pub use animation::{
  AnimationBinding, AnimationClock, AnimationEasing, AnimationEvent, AnimationEventKind,
  AnimationHandle, AnimationId, AnimationInterpolation, AnimationOwner, AnimationPlaybackOptions,
  AnimationProperty, AnimationRepeatCount, AnimationRepeatMode, AnimationRepeatOptions,
  AnimationService, AnimationSource, AnimationTarget, AnimationValue, CharacterEffectService,
  TweenDefinition,
};
pub use async_runtime::{
  AsyncRuntime, EngineEvent, EngineTask, FileEvent, FileTask, ImageEvent, SleepTask, TaskId,
  TaskState, TimeAsyncEvent,
};
pub use audio::{
  AudioAsyncEvent, AudioCaptureId, AudioError, AudioErrorCode, AudioId, AudioObjectPool,
  AudioPoolId, AudioService, AudioSource, AudioState, ResolvedAudioFile,
};
pub use canvas::{CanvasCell, CanvasService};
pub use tg_service_clipboard::ClipboardService;
pub use tg_service_code_highlight::{CodeHighlightService, CodeHighlightTheme};
pub use event::EngineEventQueue;
pub use export::{ExportAsyncEvent, ExportService, ExportTask};
pub use tg_service_ffmpeg::{FfmpegInstallation, FfmpegService};
pub use file::FileService;
pub use tg_service_host_object::{HostAreaKind, HostObjectPool};
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{ImageConvertParams, ImageService};
pub use input::{
  ActionMapEntry, InputActionEvent, InputEventType, InputService, Key, KeyEvent, KeyEventKind,
  KeyState, MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, ScrollDirection, SystemEvent,
  TerminalKeyCode, TerminalKeyEvent, format_key_display, key_token, translate_action_map,
};
pub use tg_service_input_method::{ImPolicy, InputMethodService};
pub use layout::{LayoutService, Rect, Size};
pub use log::{
  HostLogMessage, LogLevel, LogPrintOptions, LogService, LogSessionId, LogSessionKind, LogSource,
};
pub use lua::{
  GameService, LuaActionState, LuaApiConfig, LuaDrawCommand, LuaDrawTarget, LuaEnqueueError,
  LuaErrorStage, LuaEventBroker, LuaEventData, LuaEventRoute, LuaExecutionStats, LuaFileOperation,
  LuaHostCommand, LuaI18nEventKind, LuaObjectPool, LuaPolicy, LuaService, LuaSessionDiagnostics,
  LuaSessionError, LuaSessionKind, LuaSessionSpec, LuaSessionToken, LuaTaskOperation,
  ScreensaverService,
};
pub use network::{
  NetworkError, NetworkErrorCode, NetworkEvent, NetworkMethod, NetworkResponse,
  NetworkResponseBody, NetworkResponseMode, NetworkService,
};
pub use package::{
  PackageAsset, PackageEvent, PackageId, PackageInfo, PackageListEntry, PackageService,
  PackageSource, PackageType,
};
pub use tg_service_popup::{PopupDismissEvent, PopupRequest, PopupService};
pub use random::RandomService;
pub use recording::{
  RecordingAsyncEvent, RecordingPlayback, RecordingService, RecordingState,
  load_recording_playback, load_recording_playback_metadata,
};
pub use render::{BorderCharacter, BorderStyle, CustomBorder, RenderService};
pub use render_pipeline::{ComposedCell, ComposedFrame, FrameCompositor, FramePresenter};
pub use rich_text::{
  RichTextParams, RichTextSegment, RichTextService, TerminalColor, TextColor, TextStyle,
  parse_text_color,
};
pub use screenshot::{ScreenshotAsyncEvent, ScreenshotRect, ScreenshotService, ScreenshotTask};
pub use storage::{
  ActionKeyMap, AutoRecordingMode, AutoSplitDuration, BestGameSave, DisplayFpsLimit,
  DisplayLogoMode, DisplayOrderMode, DisplaySettingsProfile, DisplaySourceMode, GamePackageState,
  GameSaveCapabilities, KeyBindingsProfile, PackageDefaultState, PackageStateProfile,
  RecordingExportFrameRate, RecordingExportQuality, RecordingFrameRate, RecordingGpuAcceleration,
  RecordingPixelScale, RecordingPopupMode, RecordingProfile, SafeModeDefault,
  ScreensaverPackageState, ScreenshotDoubleAction, ScreenshotProfile, StorageService,
};
pub use tg_service_terminal::TerminalService;
pub use text_layout::{DrawTextParams, TextAlign, TextMode, TextWrapMode};
pub use time::TimeService;
pub use ui::{UiEvent, UiObjectPool, UiObjectPoolOwner, UiService};
pub use unicode::UnicodeService;
pub use version::{
  HOST_API_VERSION, HOST_VERSION, IMAGE_CACHE_FORMAT_VERSION, MEDIA_MANIFEST_VERSION,
};
pub use video::{VideoAsyncEvent, VideoExportStage, VideoService};
pub use widget::{
  DelayTimerEvent, HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, HyperlinkEvent,
  HyperlinkService, MarkdownEvent, MarkdownRenderParams, MarkdownService, MarkdownViewId,
  MarkdownViewOptions, Overflow, ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions,
  ProgressBarSegmentStyle, ProgressBarService, RandomConfiguration, RandomConfiguredRange,
  RandomGeneratedValue, RandomGeneratorId, RandomSeed, RepeatTimerEvent, RepeatTimerId,
  RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions,
  ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarStyle, ScrollbarVisibility, SliceId,
  SliceLength, SliceOptions, SliceRect, SliceService, SurfaceId, TableBorderMode, TableColumn,
  TableDrawParams, TableId, TableOptions, TableOverflow, TableRow, TableService, TableStyle,
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService, TimerEvent, TimerId, TimerState,
};
pub use write_barrier::WriteBarrier;

/// 引擎核心服务集合，持有所有子服务的实例
pub struct EngineServices {
  pub async_runtime: AsyncRuntime,
  pub engine_events: EngineEventQueue,
  pub audio: AudioService,
  pub file: FileService,
  pub network: NetworkService,
  pub random: RandomService,
  pub animation: AnimationService,
  pub character_effect: CharacterEffectService,
  pub screenshot: ScreenshotService,
  pub recording: RecordingService,
  pub ffmpeg: FfmpegService,
  pub video: VideoService,
  pub package: PackageService,
  pub popup: PopupService,
  pub clipboard: ClipboardService,
  pub runtime_objects: RuntimeObjectPool,
  pub time: TimeService,
  pub host_objects: HostObjectPool,
  pub hit_area: HitAreaService,
  pub hyperlink: HyperlinkService,
  pub scroll_box: ScrollBoxService,
  pub markdown: MarkdownService,
  pub code_highlight: CodeHighlightService,
  pub progress_bar: ProgressBarService,
  pub table: TableService,
  pub slice: SliceService,
  pub input: InputService,
  pub input_method: InputMethodService,
  pub ui: UiService,
  pub game: GameService,
  pub image: ImageService,
  pub screensaver: ScreensaverService,
  pub storage: StorageService,
  pub export: ExportService,
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
    let _ = log.set_output_path(storage.tui_log_path());
    let _ = log.set_package_scan_output_path(storage.package_log_path());
    let image_cache_dir = storage.path("data/cache/images");
    let ffmpeg = FfmpegService::new(
      storage.root_dir().to_path_buf(),
      storage.cache_dir_path().join("ffmpeg"),
    );
    let async_runtime = AsyncRuntime::new();
    let audio = AudioService::new(async_runtime.event_sender());

    Self {
      async_runtime,
      engine_events: EngineEventQueue::new(),
      audio,
      file: FileService::new(),
      network: NetworkService::new(),
      random: RandomService::new(),
      animation: AnimationService::new(),
      character_effect: CharacterEffectService::new(),
      screenshot: ScreenshotService::new(),
      recording: RecordingService::new(),
      ffmpeg,
      video: VideoService::new(),
      terminal: TerminalService::new(),
      clipboard: ClipboardService::new(),
      runtime_objects: RuntimeObjectPool::new(),
      time: TimeService::new(),
      host_objects: HostObjectPool::new(),
      hit_area: HitAreaService::new(),
      hyperlink: HyperlinkService::new(),
      scroll_box: ScrollBoxService::new(),
      markdown: MarkdownService::new(),
      code_highlight: CodeHighlightService::new(),
      progress_bar: ProgressBarService::new(),
      table: TableService::new(),
      slice: SliceService::new(),
      text_input: TextInputService::new(),
      package: PackageService::new(),
      popup: PopupService::new(),
      input: InputService::new(),
      input_method: InputMethodService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      image: ImageService::new(Some(image_cache_dir)),
      screensaver: ScreensaverService::new(),
      storage,
      export: ExportService::new(),
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
