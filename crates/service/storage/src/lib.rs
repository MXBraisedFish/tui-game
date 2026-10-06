//! Deployment-relative storage paths, persisted profiles, package state, and game saves.
//!
//! # Examples
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! use tg_service_log::LogService;
//! use tg_service_storage::{DisplaySettingsProfile, StorageService};
//!
//! fn main() {
//!   let root = create_temp_dir("tg-storage-smoke");
//!   std::fs::create_dir_all(root.join("assets")).expect("temp root");
//!
//!   let mut log = LogService::new();
//!   let storage = StorageService::new(root.clone(), &mut log).expect("initialize explicit root");
//!   assert!(
//!     storage.data_dir_path().is_dir(),
//!     "data directory is created"
//!   );
//!   let fps = storage.display_settings_profile().game_list_fps;
//!   assert_eq!(
//!     fps,
//!     DisplaySettingsProfile::default().game_list_fps,
//!     "fresh root uses defaults"
//!   );
//!   println!(
//!     "storage ok: root={} fps={fps:?}",
//!     storage.root_dir().display()
//!   );
//!
//!   std::fs::remove_dir_all(&root).expect("clean temp root");
//! }
//!
//! fn create_temp_dir(prefix: &str) -> PathBuf {
//!   let nonce = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap_or_default()
//!     .as_nanos();
//!   let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
//!   std::fs::create_dir(&root)
//!     .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
//!   root
//! }
//! ```

mod bootstrap;
mod game_save;
mod layout;
mod profile;
mod service;

pub use game_save::{BestGameSave, ContinueGameSave, GameSaveCapabilities, GameSaveProfile};
pub use profile::{
  ActionKeyMap, AutoRecordingMode, AutoSplitDuration, DisplayFpsLimit, DisplayLogoMode,
  DisplayOrderMode, DisplaySettingsProfile, DisplaySourceMode, GamePackageState,
  KeyBindingsProfile, PackageDefaultState, PackageStateProfile, RecordingExportFrameRate,
  RecordingExportQuality, RecordingFrameRate, RecordingGpuAcceleration, RecordingPixelScale,
  RecordingPopupMode, RecordingProfile, ScreensaverPackageState, ScreenshotDoubleAction,
  ScreenshotProfile,
};
pub use service::StorageService;
