//! 运行阶段宿主 UI 状态容器。
//!
//! 宿主 UI 已迁移到 Rust `UiManager`；本模块只保留运行时状态、游戏会话和
//! Rust UI 所需的数据刷新能力。

use crate::host_engine::boot::environment::data_dirs;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value as JsonValue;

use crate::LoadedResources;
use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::lua_runtime::{LuaRuntimeConsumer, LuaRuntimeContext};
use crate::host_engine::runtime::game_engine::{GameSession, script_loader};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::runtime::ui_state::action_map::UiActionMap;
use crate::host_engine::runtime::ui_state::display_state::{DisplayPanelKind, DisplayUiState};
use crate::host_engine::runtime::ui_state::game_list_state::GameListUiState;
use crate::host_engine::runtime::ui_state::home_state::HomeUiState;
use crate::host_engine::runtime::ui_state::keybind_state::KeybindUiState;
use crate::host_engine::runtime::ui_state::keybind_system_state::KeybindSystemUiState;
use crate::host_engine::runtime::ui_state::language_state::LanguageUiState;
use crate::host_engine::runtime::ui_state::memory_state::MemoryUiState;
use crate::host_engine::runtime::ui_state::mod_hub_state::ModHubUiState;
use crate::host_engine::runtime::ui_state::mod_list_state::ModListUiState;
use crate::host_engine::runtime::ui_state::needed_size_state::NeededSizeMode;
use crate::host_engine::runtime::ui_state::overlay_list_state::{
    OverlayListKind, OverlayListUiState,
};
use crate::host_engine::runtime::ui_state::security_state::SecurityUiState;
use crate::host_engine::runtime::ui_state::setting_state::SettingUiState;

type UiRuntimeResult<T> = Result<T, Box<dyn std::error::Error>>;
const MOD_SCAN_CHECK_INTERVAL: Duration = Duration::from_secs(2);

/// 当前激活的宿主 UI 页面状态。
pub struct ActiveUiPage {
    keybinds: JsonValue,
    game_modules: Vec<GameModule>,
    overlay_registry: crate::host_engine::boot::preload::overlay_modules::OverlayRegistry,
    page_key: UiPageKey,
    page_needs_reload: bool,
    home_state: HomeUiState,
    game_list_state: GameListUiState,
    setting_state: SettingUiState,
    display_state: DisplayUiState,
    keybind_state: KeybindUiState,
    keybind_system_state: KeybindSystemUiState,
    language_state: LanguageUiState,
    memory_state: MemoryUiState,
    security_state: SecurityUiState,
    mod_hub_state: ModHubUiState,
    mod_list_state: ModListUiState,
    screensaver_list_state: OverlayListUiState,
    boss_list_state: OverlayListUiState,
    needed_size_mode: NeededSizeMode,
    action_map: UiActionMap,
    game_session: Option<GameSession>,
    mod_directory_signature: String,
    last_mod_scan_check: Instant,
}

/// 加载初始 Home UI 页面。
pub(crate) fn load_home_page(
    lua_runtime: &LuaRuntimeState,
    loaded_resources: &LoadedResources,
) -> UiRuntimeResult<ActiveUiPage> {
    let page_key = UiPageKey::Home;
    let action_map = UiActionMap::from_page(
        page_key.as_str(),
        &loaded_resources.persistent_data.keybinds,
    );

    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    let terminal_size = loaded_resources.initialized_environment.terminal_size;
    host_bridge.set_runtime_context(LuaRuntimeContext {
        consumer: LuaRuntimeConsumer::GamePackage,
        current_game: None,
        current_overlay: None,
        current_ui_actions: action_map.actions_value(),
        current_script_root: None,
        language_code: loaded_resources.persistent_data.language_code.clone(),
        keybinds: loaded_resources.persistent_data.keybinds.clone(),
        best_scores: loaded_resources.persistent_data.best_scores.clone(),
        game_state: loaded_resources.persistent_data.game_state.clone(),
        screensaver_state: loaded_resources.persistent_data.screensaver_state.clone(),
        boss_state: loaded_resources.persistent_data.boss_state.clone(),
        launch_mode: Default::default(),
        terminal_size,
        is_focused: true,
    });
    host_bridge.resize_canvas(terminal_size)?;

    let mut home_state = HomeUiState::new();
    home_state.reset_lua_state();
    let mut game_list_state = GameListUiState::new(
        loaded_resources.game_module_registry.clone(),
        loaded_resources.persistent_data.best_scores.clone(),
        loaded_resources.persistent_data.language_code.clone(),
        loaded_resources.persistent_data.game_state.clone(),
    );
    game_list_state.reset_lua_state();
    let mut setting_state = SettingUiState::new();
    setting_state.reset_lua_state();
    let mut display_state = DisplayUiState::new(
        loaded_resources.persistent_data.display_state.clone(),
        loaded_resources.overlay_registry.clone(),
        loaded_resources.persistent_data.screensaver_state.clone(),
        loaded_resources.persistent_data.boss_state.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    display_state.reset_lua_state();
    game_list_state.set_show_mod_badge(display_state.show_mod_badge());
    let mut keybind_state = KeybindUiState::new();
    keybind_state.reset_lua_state();
    let mut keybind_system_state = KeybindSystemUiState::new(
        crate::host_engine::runtime::ui_page::action_defaults::all_page_actions().clone(),
        loaded_resources.persistent_data.keybinds.clone(),
    );
    keybind_system_state.reset_lua_state();
    let mut language_state = LanguageUiState::new(
        loaded_resources.persistent_data.language_code.clone(),
        loaded_resources.cache_data.language_ui_texts.clone(),
    );
    language_state.reset_lua_state();
    let mut memory_state = MemoryUiState::new();
    memory_state.reset_lua_state();
    let mut security_state = SecurityUiState::new(
        crate::host_engine::boot::preload::persistent_data::security_profile::SecurityProfile::from_value(
            &loaded_resources.persistent_data.security_state,
        ),
    );
    security_state.reset_lua_state();
    let mut mod_hub_state = ModHubUiState::new();
    mod_hub_state.reset_lua_state();
    let mut mod_list_state = ModListUiState::new(
        loaded_resources.game_module_registry.clone(),
        loaded_resources.persistent_data.game_state.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    mod_list_state.reset_lua_state();
    let mut screensaver_list_state = OverlayListUiState::new(
        OverlayListKind::Screensaver,
        loaded_resources.overlay_registry.clone(),
        loaded_resources.persistent_data.screensaver_state.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    screensaver_list_state.reset_lua_state();
    let mut boss_list_state = OverlayListUiState::new(
        OverlayListKind::Boss,
        loaded_resources.overlay_registry.clone(),
        loaded_resources.persistent_data.boss_state.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    boss_list_state.reset_lua_state();
    Ok(ActiveUiPage {
        keybinds: loaded_resources.persistent_data.keybinds.clone(),
        game_modules: loaded_resources.game_module_registry.games.clone(),
        overlay_registry: loaded_resources.overlay_registry.clone(),
        page_key,
        page_needs_reload: false,
        home_state,
        game_list_state,
        setting_state,
        display_state,
        keybind_state,
        keybind_system_state,
        language_state,
        memory_state,
        security_state,
        mod_hub_state,
        mod_list_state,
        screensaver_list_state,
        boss_list_state,
        needed_size_mode: NeededSizeMode::Root,
        action_map,
        game_session: None,
        mod_directory_signature: mod_directory_signature(),
        last_mod_scan_check: Instant::now(),
    })
}

impl ActiveUiPage {
    /// 根据物理按键查找当前 UI 页面动作。
    pub fn action_for_key(&self, key: &str) -> Option<String> {
        self.action_map.action_for_key(key)
    }

    /// 当前页面内置动作 JSON。
    pub fn action_value(&self) -> JsonValue {
        self.action_map.actions_value()
    }

    /// 当前页面键。
    pub fn page_key(&self) -> UiPageKey {
        self.page_key
    }

    /// 设置尺寸提示模式。
    pub fn set_needed_size_mode(&mut self, needed_size_mode: NeededSizeMode) {
        self.needed_size_mode = needed_size_mode;
    }

    /// 当前是否正在运行游戏。
    pub(crate) fn has_game_session(&self) -> bool {
        self.game_session.is_some()
    }

    /// 当前游戏会话。
    pub(crate) fn game_session(&self) -> Option<&GameSession> {
        self.game_session.as_ref()
    }

    /// 当前游戏会话，可变。
    pub(crate) fn game_session_mut(&mut self) -> Option<&mut GameSession> {
        self.game_session.as_mut()
    }

    /// 清除当前游戏会话。
    pub(crate) fn clear_game_session(&mut self) {
        self.game_session = None;
        self.page_needs_reload = true;
    }

    /// 从宿主 Rust UI 启动指定游戏。
    pub(crate) fn start_game(
        &mut self,
        lua_runtime: &LuaRuntimeState,
        game_uid: &str,
    ) -> UiRuntimeResult<bool> {
        let Some(game_module) = self
            .game_modules
            .iter()
            .find(|game_module| game_module.uid == game_uid)
            .cloned()
        else {
            return Ok(false);
        };
        self.game_session = Some(script_loader::load_new_game(lua_runtime, game_module)?);
        Ok(true)
    }

    /// 刷新游戏列表使用的最佳记录快照。
    pub(crate) fn refresh_best_scores(&mut self, best_scores: JsonValue) {
        self.game_list_state.refresh_best_scores(best_scores);
    }

    /// 当前覆盖层包注册表。
    pub(crate) fn overlay_registry(
        &self,
    ) -> &crate::host_engine::boot::preload::overlay_modules::OverlayRegistry {
        &self.overlay_registry
    }

    pub(crate) fn root_idle_threshold(&self) -> u64 {
        self.display_state.idle_threshold()
    }

    pub(crate) fn should_auto_enter_screensaver(&self) -> bool {
        self.display_state.should_auto_enter_screensaver()
    }

    pub(crate) fn next_screensaver_overlay_uid(&mut self) -> Option<String> {
        self.display_state
            .selected_overlay_uid(DisplayPanelKind::Screensaver)
    }

    pub(crate) fn next_boss_overlay_uid(&mut self) -> Option<String> {
        self.display_state
            .selected_overlay_uid(DisplayPanelKind::Boss)
    }
}

/// 确保当前已加载指定 UI 页面脚本。
pub(crate) fn ensure_page(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    page_key: UiPageKey,
) -> UiRuntimeResult<()> {
    if matches!(
        page_key,
        UiPageKey::ModGameList | UiPageKey::ModScreensaverList | UiPageKey::ModBossList
    ) {
        refresh_mod_modules_if_needed(lua_runtime, active_ui_page)?;
    }

    if active_ui_page.page_key == page_key && !active_ui_page.page_needs_reload {
        return Ok(());
    }

    let action_map = UiActionMap::from_page(page_key.as_str(), &active_ui_page.keybinds);
    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    host_bridge.set_current_ui_actions(action_map.actions_value());
    active_ui_page.page_key = page_key;
    active_ui_page.page_needs_reload = false;
    active_ui_page.action_map = action_map;

    if page_key == UiPageKey::Home {
        active_ui_page.home_state.reset_lua_state();
    }
    if page_key == UiPageKey::GameList {
        active_ui_page.game_list_state.reset_lua_state();
    }
    if page_key == UiPageKey::Setting {
        active_ui_page.setting_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingDisplay {
        active_ui_page.display_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingKeybind {
        active_ui_page.keybind_state.reset_lua_state();
    }
    if page_key == UiPageKey::KeybindSystem {
        active_ui_page.keybind_system_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingLanguage {
        active_ui_page.language_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingMemory {
        active_ui_page.memory_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingSecurity {
        active_ui_page.security_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingMods {
        active_ui_page.mod_hub_state.reset_lua_state();
    }
    if page_key == UiPageKey::ModGameList {
        active_ui_page.mod_list_state.reset_lua_state();
    }
    if page_key == UiPageKey::ModScreensaverList {
        active_ui_page.screensaver_list_state.reset_lua_state();
    }
    if page_key == UiPageKey::ModBossList {
        active_ui_page.boss_list_state.reset_lua_state();
    }
    if page_key == UiPageKey::StorageDetails {
        active_ui_page.memory_state.root_state =
            crate::host_engine::runtime::ui_state::memory_state::MemoryRootState::new(
                active_ui_page.memory_state.root_state.select,
            );
    }

    Ok(())
}

fn refresh_mod_modules_if_needed(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
) -> UiRuntimeResult<()> {
    if active_ui_page.last_mod_scan_check.elapsed() < MOD_SCAN_CHECK_INTERVAL {
        return Ok(());
    }
    active_ui_page.last_mod_scan_check = Instant::now();

    let current_signature = mod_directory_signature();
    if current_signature == active_ui_page.mod_directory_signature {
        return Ok(());
    }

    let registry = crate::host_engine::boot::preload::game_modules::load()?;
    let overlay_registry = crate::host_engine::boot::preload::overlay_modules::load()?;
    let persistent_data = crate::host_engine::boot::preload::persistent_data::load()?;
    let _ = crate::host_engine::boot::preload::cache_data::load(&registry)?;

    active_ui_page.game_modules = registry.games.clone();
    active_ui_page.keybinds = persistent_data.keybinds.clone();
    active_ui_page.overlay_registry = overlay_registry.clone();
    active_ui_page.game_list_state = GameListUiState::new(
        registry.clone(),
        persistent_data.best_scores.clone(),
        persistent_data.language_code.clone(),
        persistent_data.game_state.clone(),
    );
    active_ui_page.game_list_state.reset_lua_state();
    active_ui_page.mod_list_state = ModListUiState::new(
        registry,
        persistent_data.game_state.clone(),
        persistent_data.language_code.clone(),
    );
    active_ui_page.mod_list_state.reset_lua_state();
    active_ui_page
        .screensaver_list_state
        .replace_registry_and_state(
            overlay_registry.clone(),
            persistent_data.screensaver_state.clone(),
            persistent_data.language_code.clone(),
        );
    active_ui_page.boss_list_state.replace_registry_and_state(
        overlay_registry.clone(),
        persistent_data.boss_state.clone(),
        persistent_data.language_code.clone(),
    );
    active_ui_page.display_state.replace_overlay_data(
        overlay_registry,
        persistent_data.screensaver_state.clone(),
        persistent_data.boss_state.clone(),
    );
    active_ui_page
        .keybind_system_state
        .refresh_keybinds(persistent_data.keybinds.clone());

    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    let mut current_context = host_bridge.runtime_context();
    current_context.language_code = persistent_data.language_code;
    current_context.keybinds = persistent_data.keybinds;
    current_context.best_scores = persistent_data.best_scores;
    current_context.game_state = persistent_data.game_state;
    current_context.screensaver_state = persistent_data.screensaver_state;
    current_context.boss_state = persistent_data.boss_state;
    host_bridge.set_runtime_context(current_context);

    active_ui_page.mod_directory_signature = current_signature;
    active_ui_page.page_needs_reload = true;
    Ok(())
}

fn mod_directory_signature() -> String {
    let root = data_dirs::root_dir().join("data/mod");
    let mut parts = Vec::new();
    collect_directory_signature(root.as_path(), root.as_path(), &mut parts);
    parts.sort();
    parts.join("|")
}

fn collect_directory_signature(root: &Path, current: &Path, parts: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let modified_secs = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        let relative_path = path
            .strip_prefix(root)
            .ok()
            .and_then(|path| path.to_str())
            .unwrap_or_default()
            .replace('\\', "/");
        parts.push(format!("{relative_path}:{modified_secs}"));
        if metadata.is_dir() {
            collect_directory_signature(root, path.as_path(), parts);
        }
    }
}
