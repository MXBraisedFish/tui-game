// game 模块的入口文件，声明所有子模块，使外部可以通过 crate::game::xxx 访问

// 动作绑定数据结构
pub mod action;

// 清单文件数据结构
pub mod manifest;

// 游戏包发现、加载、验证
pub mod package;

// 游戏注册表，管理所有已发现游戏的描述符集合
pub mod registry;

// 包资源访问与国际化文本解析
pub mod resources;
