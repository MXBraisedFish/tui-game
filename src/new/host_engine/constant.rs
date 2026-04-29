// constant.rs — 宿主硬编码常量，公开到全局

/// API 版本号
pub const API_VERSION: u32 = 1;

/// 宿主版本号（编译时注入）
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub 最新版本检查 API
pub const UPDATE_API_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";

/// Gtihub 页面
pub const GITHUB_URL: &str = "https://github.com/MXBraisedFish/tui-game";
