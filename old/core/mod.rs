// core 模块的入口文件，负责声明所有子模块，使外部可以通过 crate::core::xxx 访问各个核心功能

// 游戏→宿主的命令枚举
pub mod command;

// 宿主→游戏的事件枚举
pub mod event;

// 全局键盘输入管理
pub mod key;

// 游戏运行时管理（帧生命周期、画布、命令收集）
pub mod runtime;

// JSON 存档管理（数据槽、继续游戏、键位持久化）
pub mod save;

// 虚拟画布（Canvas），像素/文本绘制
pub mod screen;

// 最佳成绩存储管理
pub mod stats;