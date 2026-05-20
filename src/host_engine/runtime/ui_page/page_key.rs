//! 状态机到 UI 包页面键的统一转换

use crate::host_engine::boot::preload::state_machine::{
    DialogState, GameListState, HostStateMachine, SettingState, TopLevelState,
};

/// UI 包页面键。
///
/// 该枚举只负责与 official_ui/package.json 的 entry/actions 键保持一致。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPageKey {
    Home,
    GameList,
    Setting,
    SettingSecurity,
    SettingMods,
    ModGameList,
    ModSaverList,
    ModBossList,
    SettingLanguage,
    SettingMemory,
    SettingKeybind,
    SettingDisplay,
    KeybindSystem,
    StorageDetails,
    WarningSecurity,
    WarningMod,
    WarningClearCache,
    WarningClearData,
    WarningNeededSize,
}

impl UiPageKey {
    /// 返回 official_ui/package.json 使用的页面键。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::GameList => "game_list",
            Self::Setting => "setting",
            Self::SettingSecurity => "setting_security",
            Self::SettingMods => "setting_mods",
            Self::ModGameList => "mod_game_list",
            Self::ModSaverList => "mod_saver_list",
            Self::ModBossList => "mod_boss_list",
            Self::SettingLanguage => "setting_language",
            Self::SettingMemory => "setting_memory",
            Self::SettingKeybind => "setting_keybind",
            Self::SettingDisplay => "setting_display",
            Self::KeybindSystem => "keybind_system",
            Self::StorageDetails => "storage_details",
            Self::WarningSecurity => "warning_security",
            Self::WarningMod => "warning_mod",
            Self::WarningClearCache => "warning_clear_cache",
            Self::WarningClearData => "warning_clear_data",
            Self::WarningNeededSize => "warning_needed_size",
        }
    }

    /// 根据宿主状态机解析当前应渲染的 UI 页面键。
    pub fn from_state_machine(host_state_machine: &HostStateMachine) -> Self {
        if let Some(dialog_state) = host_state_machine.dialog_state.as_ref() {
            return Self::from_dialog_state(dialog_state);
        }

        match host_state_machine.top_level_state {
            TopLevelState::Home => Self::Home,
            TopLevelState::GameList => {
                Self::from_game_list_state(&host_state_machine.game_list_state)
            }
            TopLevelState::Setting => Self::from_setting_state(&host_state_machine.setting_state),
            TopLevelState::About => Self::Setting,
        }
    }

    fn from_game_list_state(game_list_state: &GameListState) -> Self {
        match game_list_state {
            GameListState::List | GameListState::Game => Self::GameList,
        }
    }

    fn from_setting_state(setting_state: &SettingState) -> Self {
        match setting_state {
            SettingState::Hub => Self::Setting,
            SettingState::Language => Self::SettingLanguage,
            SettingState::ModList => Self::SettingMods,
            SettingState::ModGameList => Self::ModGameList,
            SettingState::ModSaverList => Self::ModSaverList,
            SettingState::ModBossList => Self::ModBossList,
            SettingState::Keybind => Self::SettingKeybind,
            SettingState::Display => Self::SettingDisplay,
            SettingState::Security => Self::SettingSecurity,
            SettingState::Memory => Self::SettingMemory,
            SettingState::KeybindSystem => Self::KeybindSystem,
            SettingState::StorageDetails => Self::StorageDetails,
        }
    }

    fn from_dialog_state(dialog_state: &DialogState) -> Self {
        match dialog_state {
            DialogState::ModSecurityWarning => Self::WarningMod,
            DialogState::SecurityWarning => Self::WarningSecurity,
            DialogState::ClearCacheWarning => Self::WarningClearCache,
            DialogState::ClearDataWarning => Self::WarningClearData,
            DialogState::NeededSizeWarning => Self::WarningNeededSize,
        }
    }
}
