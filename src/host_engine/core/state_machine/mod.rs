// 第一层
// Boot -> 启动服务/CLI指令处理
// Init -> 初始化服务
// Runtime -> 运行时逻辑
// Shutdown -> 关闭服务/清理资源
// Stopped -> 停止服务/退出程序
// 要同步更新Panic的枚举

// 第二层(Runtime)
// MainHost -> 主宿主环境
// Overlay -> 叠加层环境

// 第三层(MainHost)
// Host -> 宿主环境
// Game -> 游戏环境

// 第四层1(Host)
// UiTree -> UI树（这部分是逻辑树形结构图，整体是一个很大的状态机结构）

// 第四层2(Game)
// GameLoop -> 游戏循环逻辑

// 额外服务
// context -> 上下文管理

// 状态机要求
// 1. Overlay，UiTree每个状态机元素必须分为logic（逻辑）和render（渲染）两个部分
// 2. Game（GameLoop）将主导游戏（lua）环境
// 3. 只有Host允许发出Shutdown信号（除非强制退出）

// 状态机传递动作映射表：
// 1. 所有状态机一视同仁：先传递动作表，然后切换运行
// 2. 所有映射表必包含全局事件（强制退出、切换覆盖层等），且为所有动作中的优先级最高

// 事件处理：
// 1. 优先路由全局事件
// 2. 其次根据当前状态机路由事件

// 渲染处理：
// 根据状态机路由即可

mod game;
mod host;
mod host_machine;
mod main_host;
mod overlay;
mod runtime;
mod ui_tree;

pub use game::GameState;

pub use host::HostState;

pub use host_machine::HostMachineState;

pub use main_host::MainHostState;

pub use overlay::{OverlayKind, OverlayLogicState, OverlayRenderState, OverlayStackState, OverlayState};

pub use runtime::RuntimeState;

pub use ui_tree::{UiNodeKind, UiNodeState, UiTreeState};
