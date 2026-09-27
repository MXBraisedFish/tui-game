use tg_core_version as version;
pub(crate) use tg_service_animation as animation;
use tg_service_audio as audio;
use tg_service_canvas as canvas;
pub(crate) use tg_service_export as export;
use tg_service_file as file;
use tg_service_i18n as i18n;
use tg_service_image as image;
use tg_service_input as input;
use tg_service_layout as layout;
use tg_service_log as log;
use tg_service_lua as lua;
use tg_service_network as network;
use tg_service_package as package;
use tg_service_random as random;
use tg_service_recording as recording;
use tg_service_render as render;
use tg_service_render_pipeline as render_pipeline;
use tg_service_screenshot as screenshot;
use tg_service_storage as storage;
pub use tg_service_text_layout as text_layout;
use tg_service_time as time;
use tg_service_video as video;
pub(crate) use tg_service_widget as widget;

pub use animation::{
  AnimationBinding, AnimationClock, AnimationEasing, AnimationHandle, AnimationInterpolation,
  AnimationOwner, AnimationPlaybackOptions, AnimationProperty, AnimationRepeatCount,
  AnimationRepeatMode, AnimationRepeatOptions, AnimationService, AnimationSource, AnimationTarget,
  AnimationValue, TweenDefinition,
};
pub use audio::{
  AudioAsyncEvent, AudioError, AudioId, AudioObjectPool, AudioService, AudioSource, AudioState,
};
pub use canvas::{CanvasCell, CanvasService};
pub use export::{ExportAsyncEvent, ExportService, ExportTask};
pub use file::FileEvent;
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{ImageConvertMode, ImageConvertParams, ImageEvent, ImageService};
pub use input::{
  ActionMapEntry, InputActionEvent, InputListenerError, InputService, Key, KeyEvent, KeyEventKind,
  KeyState, MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, SystemEvent, TerminalKeyCode,
  format_key_display, key_token, translate_action_map,
};
pub use layout::{LayoutService, Rect, Size};
pub use log::{
  HostLogMessage, LogLevel, LogPrintOptions, LogService, LogSessionId, LogSessionKind, LogSource,
};
pub use lua::{
  GameService, GameStartOptions, LuaActionState, LuaApiConfig, LuaDrawCommand, LuaDrawTarget,
  LuaEnqueueError, LuaErrorStage, LuaEventBroker, LuaEventData, LuaEventRoute, LuaHostCommand,
  LuaObjectPool, LuaRoutableEvent, LuaService, LuaSessionDiagnostics, LuaSessionError,
  LuaSessionKind, LuaSessionSpec, LuaSessionToken, LuaTaskOperation, ScreensaverService,
};
pub use network::{NetworkEvent, NetworkService};
pub use package::{
  PackageAsset, PackageAsyncEvent, PackageEvent, PackageId, PackageInfo, PackageListEntry,
  PackageService, PackageSource,
};
pub use random::{RandomGeneratorId, RandomSeed, RandomService};
pub use recording::{
  RecordingAsyncEvent, RecordingPlayback, RecordingService, RecordingState,
  load_recording_playback, load_recording_playback_metadata,
};
pub use render::{BorderStyle, RenderService};
pub use render_pipeline::{ComposedCell, ComposedFrame, FrameCompositor, FramePresenter};
pub use screenshot::{ScreenshotAsyncEvent, ScreenshotRect, ScreenshotService, ScreenshotTask};
pub use storage::{
  ActionKeyMap, AutoRecordingMode, AutoSplitDuration, BestGameSave, DisplayFpsLimit,
  DisplayLogoMode, DisplayOrderMode, DisplaySettingsProfile, DisplaySourceMode, GamePackageState,
  GameSaveCapabilities, KeyBindingsProfile, PackageDefaultState, PackageStateProfile,
  RecordingExportFrameRate, RecordingExportQuality, RecordingFrameRate, RecordingGpuAcceleration,
  RecordingPixelScale, RecordingPopupMode, RecordingProfile, ScreensaverPackageState,
  ScreenshotDoubleAction, ScreenshotProfile, StorageService,
};
pub use tg_service_async::TaskId;
pub use tg_service_clipboard::ClipboardService;
pub use tg_service_code_highlight::CodeHighlightService;
pub use tg_service_ffmpeg::FfmpegService;
pub use tg_service_host_object::{HostAreaKind, HostObjectPool};
pub use tg_service_input_method::{ImPolicy, InputMethodService};
pub use tg_service_popup::{PopupDismissEvent, PopupRequest, PopupService};
pub use tg_service_rich_text::{
  RichTextParams, RichTextService, TerminalColor, TextColor, TextMode, TextStyle,
};
pub use tg_service_terminal::TerminalService;
pub use tg_service_text_layout::{DrawTextParams, TextAlign};
pub use time::{TimeAsyncEvent, TimeService, TimerId};
pub use version::{HOST_VERSION, MEDIA_MANIFEST_VERSION};
pub use video::{VideoAsyncEvent, VideoExportStage, VideoService};
pub use widget::{
  HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, HyperlinkService, MarkdownRenderParams,
  MarkdownService, MarkdownViewId, MarkdownViewOptions, Overflow, ProgressBarFillOrigin,
  ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService,
  RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions,
  ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility, TableBorderMode,
  TableColumn, TableDrawParams, TableId, TableOptions, TableOverflow, TableRow, TableService,
  TableStyle, TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService,
};
pub use widget::{UiEvent, UiObjectPool, UiObjectPoolOwner};

#[cfg(test)]
pub use file::FileTask;
#[cfg(test)]
pub use input::InputEventType;
#[cfg(test)]
pub use lua::LuaExecutionStats;
#[cfg(test)]
pub use package::PackageType;
#[cfg(test)]
pub use time::SleepTask;
