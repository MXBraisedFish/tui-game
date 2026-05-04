//! 启动前最终准备入口

mod readiness;
mod validator;

pub use readiness::LaunchReadiness;

use crate::host_engine::boot::preload::cache_data::CacheData;
use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;
use crate::host_engine::boot::preload::official_ui::OfficialUiRegistry;
use crate::host_engine::boot::preload::persistent_data::PersistentData;
use crate::host_engine::boot::preload::state_machine::HostStateMachine;

/// 执行启动前最终准备。
///
/// 当前阶段只做基础一致性确认和待办项记录。
/// TODO: 后续在这里把游戏模块、持久化数据、缓存数据拼合成运行时只读上下文。
/// TODO: 后续在这里检查 UI Lua 入口脚本和宿主 API 暴露状态。
/// TODO: 后续在这里确认所有图片缓存、语言缓存、按键映射缓存已可直接供运行时使用。
pub fn load(
    game_module_registry: &GameModuleRegistry,
    official_ui_registry: &OfficialUiRegistry,
    persistent_data: &PersistentData,
    cache_data: &CacheData,
    host_state_machine: &HostStateMachine,
) -> Result<LaunchReadiness, Box<dyn std::error::Error>> {
    validator::validate_launch_readiness(
        game_module_registry,
        official_ui_registry,
        persistent_data,
        cache_data,
        host_state_machine,
    )
}
