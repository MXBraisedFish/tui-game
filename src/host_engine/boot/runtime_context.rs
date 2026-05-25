//! Boot 阶段输出给 Runtime 阶段的统一上下文。

use std::sync::Arc;

use crate::host_engine::boot::i18n::I18nText;
use crate::host_engine::boot::preload::cache_data::CacheData;
use crate::host_engine::boot::preload::finalize_launch::LaunchReadiness;
use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;
use crate::host_engine::boot::preload::init_environment::{InitializedEnvironment, TerminalSize};
use crate::host_engine::boot::preload::overlay_modules::OverlayRegistry;
use crate::host_engine::boot::preload::persistent_data::PersistentData;
use crate::host_engine::boot::preload::state_machine::HostStateMachine;
use crate::host_engine::keybind::keybind_manager::KeybindManager;
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::storage::cache_store::CacheStore;
use crate::host_engine::storage::profile_store::ProfileStore;
pub use crate::host_engine::theme::ThemeManager;

/// Boot 阶段记录的终端环境概要。
#[derive(Clone, Debug)]
pub struct TerminalManager {
    pub terminal_size: TerminalSize,
    pub input_listener_running: bool,
}

impl TerminalManager {
    pub fn from_environment(environment: &InitializedEnvironment) -> Self {
        Self {
            terminal_size: environment.terminal_size,
            input_listener_running: environment.is_input_listener_running(),
        }
    }
}

/// Runtime 启动上下文。
///
/// 前半部分是新架构字段；后半部分保留旧 Runtime 仍在读取的数据边界。
/// 后续 Runtime 迁移完成后，可删除旧兼容字段。
pub struct RuntimeContext {
    pub terminal: TerminalManager,
    pub host_i18n: I18nText,
    pub profiles: Arc<ProfileStore>,
    pub cache: Arc<CacheStore>,
    pub packages: PackageManager,
    pub keybinds: KeybindManager,
    pub themes: ThemeManager,
    pub state_machine: HostStateMachine,
    pub launch_readiness: LaunchReadiness,

    pub initialized_environment: InitializedEnvironment,
    pub game_module_registry: GameModuleRegistry,
    pub overlay_registry: OverlayRegistry,
    pub persistent_data: PersistentData,
    pub cache_data: CacheData,
}
