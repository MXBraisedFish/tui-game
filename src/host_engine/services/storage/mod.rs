use tg_core_atomic_fs as atomic;
mod bootstrap;
mod game_save;
mod layout;
mod profile;
mod service;

pub(crate) use atomic::{atomic_replace_with, atomic_write};
pub use game_save::{BestGameSave, ContinueGameSave, GameSaveCapabilities, GameSaveProfile};
pub use profile::{
  ActionKeyMap, AutoRecordingMode, AutoSplitDuration, DisplayFpsLimit, DisplayLogoMode,
  DisplayOrderMode, DisplaySettingsProfile, DisplaySourceMode, GamePackageState,
  KeyBindingsProfile, PackageDefaultState, PackageStateProfile, RecordingExportFrameRate,
  RecordingExportQuality, RecordingFrameRate, RecordingGpuAcceleration, RecordingPixelScale,
  RecordingPopupMode, RecordingProfile, SafeModeDefault, ScreensaverPackageState,
  ScreenshotDoubleAction, ScreenshotProfile,
};
pub use service::StorageService;
