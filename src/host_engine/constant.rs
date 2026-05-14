// constant.rs — 宿主硬编码常量，公开到全局

/// API 版本号
pub const API_VERSION: u32 = 1;

/// 宿主版本号（编译时注入）
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 宿主 UI 最小宽度（终端字符列数）。
pub const ROOT_UI_MIN_WIDTH: u16 = 98;

/// 宿主 UI 最小高度（终端字符行数）。
pub const ROOT_UI_MIN_HEIGHT: u16 = 26;

/// 单个动作最多允许绑定的物理按键数量。
pub const MAX_ACTION_KEYS: usize = 4;

/// 宿主默认 icon，供游戏包、Saver 包、老板包共享。
pub const DEFAULT_PACKAGE_ICON: &[&str] = &["████████", "██ ██ ██", "   ██   ", "  ████  "];

/// 游戏包默认 banner。
pub const DEFAULT_GAME_BANNER: &[&str] = &[
    "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.   ",
    "  MMMb    dPMM .dP'    `YM. MM    `Yb. ",
    "  M YM   ,M MM dM'      `MM MM     `Mb ",
    "  M  Mb  M' MM MM        MM MM      MM ",
    "  M  YM.P'  MM MM.      ,MP MM     ,MP ",
    "  M  `YM'   MM `Mb.    ,dP' MM    ,dP' ",
    ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'   ",
];

/// 老板包默认 banner。
pub const DEFAULT_BOSS_BANNER: &[&str] = &[
    "__      __ ___     ___    _  __  ",
    "\\ \\    / // _ \\   | _ \\  | |/ /  ",
    " \\ \\/\\/ /| (_) |  |   /  | ' <   ",
    "  \\_/\\_/  \\___/   |_|_\\  |_|\\_\\  ",
    "_|\"\"\"\"\"|_|\"\"\"\"\"|_|\"\"\"\"\"|_|\"\"\"\"\"| ",
    "\"`-0-0-'\"`-0-0-'\"`-0-0-'\"`-0-0-' ",
];

/// Saver 包默认 banner。
pub const DEFAULT_SAVER_BANNER: &[&str] = &[
    ":::::::::  :::     ::: ::::::::: ",
    ":+:    :+: :+:     :+: :+:    :+:",
    "+:+    +:+ +:+     +:+ +:+    +:+",
    "+#+    +:+ +#+     +:+ +#+    +:+",
    "+#+    +#+  +#+   +#+  +#+    +#+",
    "#+#    #+#   #+#+#+#   #+#    #+#",
    "#########      ###     ######### ",
];

/// GitHub 最新版本检查 API
pub const UPDATE_API_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";

/// Gtihub 页面
pub const GITHUB_URL: &str = "https://github.com/MXBraisedFish/tui-game";
