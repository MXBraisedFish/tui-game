//! 宿主状态机根结构

use super::dialog_level::{DialogContext, DialogState};
use super::mid_level::{GameListState, SettingState};
use super::top_level::TopLevelState;

/// 宿主三层状态机。
///
/// 结构分为：
/// - 顶层状态机：主页面互斥切换。
/// - 中层状态机：特定页面内部的子页面切换。
/// - 弹窗状态机：模态弹窗覆盖当前页面。
#[derive(Clone, Debug)]
pub struct HostStateMachine {
    pub top_level_state: TopLevelState,
    pub game_list_state: GameListState,
    pub setting_state: SettingState,
    pub dialog_state: Option<DialogState>,
    pub dialog_context: DialogContext,
}

impl HostStateMachine {
    /// 构建启动后的初始状态。
    pub fn new() -> Self {
        Self {
            top_level_state: TopLevelState::Home,
            game_list_state: GameListState::List,
            setting_state: SettingState::Hub,
            dialog_state: None,
            dialog_context: DialogContext::None,
        }
    }

    /// 当前是否存在模态弹窗。
    pub fn has_dialog(&self) -> bool {
        self.dialog_state.is_some()
    }

    /// TODO: 后续在运行时事件循环中实现顶层页面切换。
    pub fn handle_top_level_transition(&mut self) {}

    /// TODO: 后续在运行时事件循环中实现中层页面切换。
    pub fn handle_mid_level_transition(&mut self) {}

    /// TODO: 后续在运行时事件循环中实现弹窗确认/取消逻辑。
    pub fn handle_dialog_transition(&mut self) {}
}
