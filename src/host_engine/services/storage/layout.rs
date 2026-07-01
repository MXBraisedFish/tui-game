pub const DATA_DIR: &str = "data";

pub const DATA_CACHE_DIR: &str = "data/cache";

pub const IMAGE_CACHE_DIR: &str = "data/cache/images";

pub const DATA_PROFILES_DIR: &str = "data/profiles";

pub const DATA_LOG_DIR: &str = "data/log";

pub const DATA_MOD_DIR: &str = "data/mod";

pub const DATA_MOD_GAME_DIR: &str = "data/mod/game";

pub const DATA_MOD_SCREENSAVER_DIR: &str = "data/mod/screensaver";

pub const SCRIPTS_DIR: &str = "scripts";

pub const SCRIPTS_GAME_DIR: &str = "scripts/game";

pub const SCRIPTS_SCREENSAVER_DIR: &str = "scripts/screensaver";

pub const ASSETS_DIR: &str = "assets";

pub const ASSETS_LANGUAGE_DIR: &str = "assets/language";

pub const GAME_SCAN_CACHE_FILE: &str = "data/cache/game_scan_cache.json";

pub const SCREENSAVER_SCAN_CACHE_FILE: &str = "data/cache/screensaver_scan_cache.json";

pub const LANGUAGE_UI_CACHE_FILE: &str = "data/cache/language_ui_cache.json";

pub const PROFILE_LANGUAGE_FILE: &str = "data/profiles/language.txt";

pub const PROFILE_TERMINAL_FILE: &str = "data/profiles/terminal_profile.json";

pub const TUI_LOG_FILE: &str = "data/log/tui_log.txt";

pub const DEFAULT_LANGUAGE_CODE: &str = "en_us";

pub const LANGUAGE_REGISTRY_FILE: &str = "assets/language/language_registry.json";

pub const REQUIRED_DIRECTORIES: &[&str] = &[
  DATA_DIR,
  DATA_CACHE_DIR,
  IMAGE_CACHE_DIR,
  DATA_PROFILES_DIR,
  DATA_LOG_DIR,
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
  (GAME_SCAN_CACHE_FILE, "{}"),
  (SCREENSAVER_SCAN_CACHE_FILE, "{}"),
  (LANGUAGE_UI_CACHE_FILE, "{}"),
  (
    PROFILE_TERMINAL_FILE,
    r#"{"unicode":null,"color":null,"mouse":null}"#,
  ),
  (TUI_LOG_FILE, ""),
  (LANGUAGE_REGISTRY_FILE, "{}"),
];
