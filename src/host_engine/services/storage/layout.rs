// 数据根目录
pub const DATA_DIR: &str = "data";
// 缓存目录
pub const DATA_CACHE_DIR: &str = "data/cache";
// 缓存-图片目录
pub const DATA_CACHE_IMAGES_DIR: &str = "data/cache/images";
// 配置目录
pub const DATA_PROFILES_DIR: &str = "data/profiles";
// 日志目录
pub const DATA_LOG_DIR: &str = "data/log";

// 模组目录
pub const DATA_MOD_DIR: &str = "data/mod";
// 游戏包
pub const DATA_MOD_GAME_DIR: &str = "data/mod/game";
// 屏保包
pub const DATA_MOD_SCREENSAVER_DIR: &str = "data/mod/screensaver";
// 老板包
pub const DATA_MOD_BOSS_DIR: &str = "data/mod/boss";

// 默认脚本包
pub const SCRIPTS_DIR: &str = "scripts";
// 游戏包
pub const SCRIPTS_GAME_DIR: &str = "scripts/game";
// 屏保包
pub const SCRIPTS_SCREENSAVER_DIR: &str = "scripts/screensaver";
// 老板包
pub const SCRIPTS_BOSS_DIR: &str = "scripts/boss";

// 默认资源包
pub const ASSETS_DIR: &str = "assets";
// 默认语言包
pub const ASSETS_LANGUAGE_DIR: &str = "assets/language";
// 默认图片资源
pub const ASSETS_IMAGES_DIR: &str = "assets/images";

// 游戏包扫描缓存
pub const GAME_SCAN_CACHE_FILE: &str = "data/cache/game_scan_cache.json";
// 屏保包扫描缓存
pub const SCREENSAVER_SCAN_CACHE_FILE: &str = "data/cache/screensaver_scan_cache.json";
// 老板包扫描缓存
pub const BOSS_SCAN_CACHE_FILE: &str = "data/cache/boss_scan_cache.json";
// ui语言缓存
pub const LANGUAGE_UI_CACHE_FILE: &str = "data/cache/language_ui_cache.json";

// 配置-语言
pub const PROFILE_LANGUAGE_FILE: &str = "data/profiles/language.txt";

// 日志-宿主
pub const TUI_LOG_FILE: &str = "data/log/tui_log.txt";

// 默认语言代码（英文）
pub const DEFAULT_LANGUAGE_CODE: &str = "en_us";

// 语言注册表文件
pub const LANGUAGE_REGISTRY_FILE: &str = "assets/language/language_registry.json";

// 必须有的目录
pub const REQUIRED_DIRECTORIES: &[&str] = &[
  DATA_DIR,
  DATA_CACHE_DIR,
  DATA_CACHE_IMAGES_DIR,
  DATA_PROFILES_DIR,
  DATA_LOG_DIR,
  DATA_MOD_DIR,
  DATA_MOD_GAME_DIR,
  DATA_MOD_SCREENSAVER_DIR,
  DATA_MOD_BOSS_DIR,
  SCRIPTS_DIR,
  SCRIPTS_GAME_DIR,
  SCRIPTS_SCREENSAVER_DIR,
  SCRIPTS_BOSS_DIR,
  ASSETS_DIR,
  ASSETS_LANGUAGE_DIR,
  ASSETS_IMAGES_DIR,
];

// 默认文件以及内容
pub const DEFAULT_FILES: &[(&str, &str)] = &[
  (GAME_SCAN_CACHE_FILE, "{}"),
  (SCREENSAVER_SCAN_CACHE_FILE, "{}"),
  (BOSS_SCAN_CACHE_FILE, "{}"),
  (LANGUAGE_UI_CACHE_FILE, "{}"),
  (PROFILE_LANGUAGE_FILE, DEFAULT_LANGUAGE_CODE),
  (TUI_LOG_FILE, ""),
  (LANGUAGE_REGISTRY_FILE, "{}"),
];
