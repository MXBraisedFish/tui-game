// app 模块的入口文件，声明所有 13 个子模块。自身无业务逻辑，仅作模块组织

// 启动时资源扫描与全局缓存
pub mod content_cache;

// 继续游戏状态同步与查询
pub mod continue_game;

// 游戏选择页面（列表+详情+排序+热重载）
pub mod game_selection;

// 国际化多语言系统
pub mod i18n;

// 主菜单页面布局计算
pub mod layout;

// 启动加载画面渲染
pub mod loading_screen;

// 应用主循环、状态机、事件分发
pub mod main_loop;

// 主菜单页面（LOGO+菜单项+继续游戏）
pub mod menu;

// 占位页面（About/Continue等）
pub mod placeholder_pages;

// 富文本解析器（f%格式）
pub mod rich_text;

// 设置系统（目录模块，含9个子模块）
pub mod settings;

// 游戏统计数据管理（空壳）
pub mod stats;

// 后台版本更新检查
pub mod version_check;