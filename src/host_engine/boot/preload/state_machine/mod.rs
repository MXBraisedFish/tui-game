//! 宿主状态机预加载入口

mod app_state;
mod dialog_level;
mod mid_level;
mod top_level;

pub use app_state::HostStateMachine;
pub use dialog_level::{DialogContext, DialogState};
pub use mid_level::{GameListState, SettingState};
pub use top_level::TopLevelState;

/// 构建宿主初始状态机。
///
/// 本阶段只建立三层状态结构，不实现状态切换；后续运行时事件循环负责根据输入驱动状态转换。
pub fn load() -> HostStateMachine {
    HostStateMachine::new()
}
