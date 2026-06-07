// 第一层
// Boot -> 启动服务/CLI指令处理
// Init -> 初始化服务
// Runtime -> 运行时逻辑
// Shutdown -> 关闭服务/清理资源
// Stopped -> 停止服务/退出程序
// 要同步更新Panic的枚举
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostMachineState {
  Boot,
  Init,
  Runtime(RuntimeState),
  Shutdown,
  Stopped,
}

// 第二层(Runtime)
// MainHost -> 主宿主环境
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeState {
  pub main_host: MainHostState,
  pub overlays: OverlayStackState,
}
// Overlay -> 叠加层环境
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayStackState {
  pub stack: Vec<OverlayState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayState {
  pub kind: OverlayKind,
  pub logic: OverlayLogicState,
  pub render: OverlayRenderState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
  ConfirmExit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayLogicState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayRenderState;

// 第三层(MainHost)
// Host -> 宿主环境
// Game -> 游戏环境
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostState {
  Host(HostState),
  Game(GameState),
}

// 第四层1(Host)
// UiTree -> UI树（这部分是逻辑树形结构图，整体是一个很大的状态机结构）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostState {
  pub ui_tree: UiTreeState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTreeState {
  pub path: Vec<UiNodeState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeState {
  pub kind: UiNodeKind,
  pub logic: UiNodeLogicState,
  pub render: UiNodeRenderState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNodeKind {
  Root,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeLogicState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeRenderState;

// 第四层2(Game)
// GameLoop -> 游戏循环逻辑
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
  pub game_loop: GameLoopState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLoopState;

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
