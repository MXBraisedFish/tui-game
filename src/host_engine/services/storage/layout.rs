pub const DATA_DIR: &str = "data";

pub const DATA_CACHE_DIR: &str = "data/cache";

pub const IMAGE_CACHE_DIR: &str = "data/cache/images";

pub const SCREENSHOT_CACHE_DIR: &str = "data/cache/screenshot";

pub const RECORDING_CACHE_DIR: &str = "data/cache/recording";

pub const DATA_PROFILES_DIR: &str = "data/profiles";

pub const DATA_LOG_DIR: &str = "data/log";

pub const DATA_SCREENSHOT_DIR: &str = "data/screenshot";

pub const DATA_RECORDING_DIR: &str = "data/recording";

pub const DATA_MOD_DIR: &str = "data/mod";

pub const DATA_MOD_GAME_DIR: &str = "data/mod/game";

pub const DATA_MOD_SCREENSAVER_DIR: &str = "data/mod/screensaver";

pub const SCRIPTS_DIR: &str = "scripts";

pub const SCRIPTS_GAME_DIR: &str = "scripts/game";

pub const SCRIPTS_SCREENSAVER_DIR: &str = "scripts/screensaver";

pub const ASSETS_DIR: &str = "assets";

pub const ASSETS_LANGUAGE_DIR: &str = "assets/language";

pub const PROFILE_LANGUAGE_FILE: &str = "data/profiles/language.txt";

pub const PROFILE_TERMINAL_FILE: &str = "data/profiles/terminal_profile.json";

pub const PROFILE_PACKAGE_STATE_FILE: &str = "data/profiles/package_state.json";

pub const PROFILE_SCREENSHOT_FILE: &str = "data/profiles/screenshot_profile.json";

pub const PROFILE_RECORDING_FILE: &str = "data/profiles/recording_profile.json";

pub const PROFILE_DISPLAY_SETTINGS_FILE: &str = "data/profiles/display_settings.json";

pub const TUI_LOG_FILE: &str = "data/log/tui_log.txt";

pub const DEFAULT_LANGUAGE_CODE: &str = "en_us";

pub const LANGUAGE_REGISTRY_FILE: &str = "assets/language/language_registry.json";

pub const REQUIRED_DIRECTORIES: &[&str] = &[
  DATA_DIR,
  DATA_CACHE_DIR,
  IMAGE_CACHE_DIR,
  SCREENSHOT_CACHE_DIR,
  RECORDING_CACHE_DIR,
  DATA_PROFILES_DIR,
  DATA_LOG_DIR,
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
    r#"{"popup":"all","auto_recording":"off","auto_split":"minutes10","capture_frame_rate":"fps60","export_frame_rate":"recorded","legacy_frame_rate":30,"quality":"balanced","keyframe_interval_seconds":2,"pixel_scale":"original"}"#,
  ),
  (
    PROFILE_DISPLAY_SETTINGS_FILE,
    r#"{"logo_mode":"order","logo_sequence_cursor":0,"top_toolbar":true,"top_toolbar_custom_text":"","screensaver_source":"all","screensaver_order":"random","screensaver_sequence_cursor":0,"game_list_source":"all","game_list_warnings":true,"game_list_fps":"fps60"}"#,
  ),
  (TUI_LOG_FILE, ""),
  (LANGUAGE_REGISTRY_FILE, "{}"),
];
