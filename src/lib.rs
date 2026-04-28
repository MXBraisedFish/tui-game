// 库 crate 的入口，声明所有顶层模块，自身无业务逻辑。属于"项目骨架"文件

// 应用层：UI、状态管理、国际化、加载画面等
pub mod app;

// 核心抽象：运行时命令、事件、键盘、存档、画布
pub mod core;

// 游戏注册：包发现、清单解析、描述符、资源访问
pub mod game;

// Lua 引擎：沙箱、生命周期、游戏 API
pub mod lua;

// Mod 系统：扫描、状态管理、图像处理
pub mod mods;

// 启动流程：环境准备、CLI 处理、panic 钩子
pub mod startup;

// 终端交互：会话管理、画布渲染、尺寸监控
pub mod terminal;

// 工具：路径管理、日志
pub mod utils;
