//! Layout support for the storage service.

/// The data dir used by this module.
pub const DATA_DIR: &str = "data";

/// The data cache dir used by this module.
pub const DATA_CACHE_DIR: &str = "data/cache";

/// The image cache dir used by this module.
pub const IMAGE_CACHE_DIR: &str = "data/cache/images";

/// The screenshot cache dir used by this module.
pub const SCREENSHOT_CACHE_DIR: &str = "data/cache/screenshot";

/// The recording cache dir used by this module.
pub const RECORDING_CACHE_DIR: &str = "data/cache/recording";

/// The data profiles dir used by this module.
pub const DATA_PROFILES_DIR: &str = "data/profiles";

/// The data log dir used by this module.
pub const DATA_LOG_DIR: &str = "data/log";

/// The data game log dir used by this module.
pub const DATA_GAME_LOG_DIR: &str = "data/log/game";

/// The data screensaver log dir used by this module.
pub const DATA_SCREENSAVER_LOG_DIR: &str = "data/log/screensaver";

/// The data screenshot dir used by this module.
pub const DATA_SCREENSHOT_DIR: &str = "data/screenshot";

/// The data recording dir used by this module.
pub const DATA_RECORDING_DIR: &str = "data/recording";

/// The data mod dir used by this module.
pub const DATA_MOD_DIR: &str = "data/mod";

/// The data mod game dir used by this module.
pub const DATA_MOD_GAME_DIR: &str = "data/mod/game";

/// The data mod screensaver dir used by this module.
pub const DATA_MOD_SCREENSAVER_DIR: &str = "data/mod/screensaver";

/// The scripts dir used by this module.
pub const SCRIPTS_DIR: &str = "scripts";

/// The scripts game dir used by this module.
pub const SCRIPTS_GAME_DIR: &str = "scripts/game";

/// The scripts screensaver dir used by this module.
pub const SCRIPTS_SCREENSAVER_DIR: &str = "scripts/screensaver";

/// The assets dir used by this module.
pub const ASSETS_DIR: &str = "assets";

/// The assets language dir used by this module.
pub const ASSETS_LANGUAGE_DIR: &str = "assets/language";

/// The profile language file used by this module.
pub const PROFILE_LANGUAGE_FILE: &str = "data/profiles/language.txt";

/// The profile terminal file used by this module.
pub const PROFILE_TERMINAL_FILE: &str = "data/profiles/terminal_profile.json";

/// The profile package state file used by this module.
pub const PROFILE_PACKAGE_STATE_FILE: &str = "data/profiles/package_state.json";

/// The profile screenshot file used by this module.
pub const PROFILE_SCREENSHOT_FILE: &str = "data/profiles/screenshot_profile.json";

/// The profile recording file used by this module.
pub const PROFILE_RECORDING_FILE: &str = "data/profiles/recording_profile.json";

/// The profile display settings file used by this module.
pub const PROFILE_DISPLAY_SETTINGS_FILE: &str = "data/profiles/display_settings.json";

/// The profile key bindings file used by this module.
pub const PROFILE_KEY_BINDINGS_FILE: &str = "data/profiles/key_bindings.json";

/// The profile game save file used by this module.
pub const PROFILE_GAME_SAVE_FILE: &str = "data/profiles/game_save.json";

/// The tUI log file used by this module.
pub const TUI_LOG_FILE: &str = "data/log/tui_log.log";

/// The package log file used by this module.
pub const PACKAGE_LOG_FILE: &str = "data/log/package.log";

/// The default language code used by this module.
pub const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// The language registry file used by this module.
pub const LANGUAGE_REGISTRY_FILE: &str = "assets/language/language_registry.json";

/// The required directories used by this module.
pub const REQUIRED_DIRECTORIES: &[&str] = &[
  DATA_DIR,
  DATA_CACHE_DIR,
  IMAGE_CACHE_DIR,
  SCREENSHOT_CACHE_DIR,
  RECORDING_CACHE_DIR,
  DATA_PROFILES_DIR,
  DATA_LOG_DIR,
  DATA_GAME_LOG_DIR,
  DATA_SCREENSAVER_LOG_DIR,
  DATA_SCREENSHOT_DIR,
  DATA_RECORDING_DIR,
  DATA_MOD_DIR,
  DATA_MOD_GAME_DIR,
  DATA_MOD_SCREENSAVER_DIR,
  SCRIPTS_DIR,
  SCRIPTS_GAME_DIR,
  SCRIPTS_SCREENSAVER_DIR,
  ASSETS_DIR,
  ASSETS_LANGUAGE_DIR,
];

/// The default files used by this module.
pub const DEFAULT_FILES: &[(&str, &str)] = &[
  (
    PROFILE_TERMINAL_FILE,
    r#"{"unicode":null,"color":null,"mouse":null}"#,
  ),
  (
    PROFILE_PACKAGE_STATE_FILE,
    r#"{"games":{},"screensavers":{}}"#,
  ),
  (
    PROFILE_SCREENSHOT_FILE,
    r#"{"guide_seen":false,"double_action":"save_png","auto_exit":false,"fonts":[]}"#,
  ),
  (
    PROFILE_RECORDING_FILE,
    r#"{"popup":"all","auto_recording":"off","auto_split":"minutes3","capture_frame_rate":"fps60","export_frame_rate":"recorded","quality":"balanced","keyframe_interval_seconds":2,"pixel_scale":"original","gpu_acceleration":"auto"}"#,
  ),
  (
    PROFILE_DISPLAY_SETTINGS_FILE,
    r#"{"logo_mode":"order","logo_sequence_cursor":0,"top_toolbar":true,"top_toolbar_custom_text":"","screensaver_source":"all","screensaver_order":"random","screensaver_sequence_cursor":0,"game_list_source":"all","game_list_warnings":true,"game_list_fps":"fps60"}"#,
  ),
  (
    PROFILE_KEY_BINDINGS_FILE,
    r#"{"default":{"global":{},"games":{}},"user":{"global":{},"games":{}}}"#,
  ),
  (
    PROFILE_GAME_SAVE_FILE,
    r#"{"continue_slot":null,"best":{}}"#,
  ),
  (TUI_LOG_FILE, ""),
  (PACKAGE_LOG_FILE, ""),
  (LANGUAGE_REGISTRY_FILE, "{}"),
];
