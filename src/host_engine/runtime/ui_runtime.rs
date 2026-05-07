//! 运行阶段官方 UI Lua 页面执行

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use mlua::{Function, Table, Value};
use serde_json::Value as JsonValue;

use crate::LoadedResources;
use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::lua_runtime::api::LuaEvent;
use crate::host_engine::boot::preload::lua_runtime::api::callback_api;
use crate::host_engine::boot::preload::lua_runtime::api::{self, ApiScope};
use crate::host_engine::boot::preload::lua_runtime::{
    HostLuaBridge, HostLuaMessage, LuaRuntimeConsumer, LuaRuntimeContext,
};
use crate::host_engine::boot::preload::state_machine::{
    DialogState, GameListState, HostStateMachine, SettingState, TopLevelState,
};
use crate::host_engine::runtime::game_engine::{GameSession, script_loader};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::runtime::ui_state::action_map::UiActionMap;
use crate::host_engine::runtime::ui_state::game_list_state::{
    GameListLuaAction, GameListLuaState, GameListUiState,
};
use crate::host_engine::runtime::ui_state::home_state::HomeUiState;
use crate::host_engine::runtime::ui_state::language_state::{
    LanguageLuaAction, LanguageLuaState, LanguageUiState,
};
use crate::host_engine::runtime::ui_state::lua_state::HomeLuaState;
use crate::host_engine::runtime::ui_state::memory_state::{
    MemoryConfirmAction, MemoryLuaAction, MemoryLuaState, MemoryUiState,
};
use crate::host_engine::runtime::ui_state::mod_list_state::{
    ModListLuaAction, ModListLuaState, ModListUiState,
};
use crate::host_engine::runtime::ui_state::needed_size_state::{
    NeededSizeMode, NeededSizeRootState,
};
use crate::host_engine::runtime::ui_state::root_state::HomeConfirmAction;
use crate::host_engine::runtime::ui_state::setting_state::{
    SettingLuaAction, SettingLuaState, SettingUiState,
};

type UiRuntimeResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 当前激活的官方 UI 页面实例。
pub struct ActiveUiPage {
    package_root: PathBuf,
    manifest: JsonValue,
    game_modules: Vec<GameModule>,
    page_key: UiPageKey,
    page_needs_reload: bool,
    home_state: HomeUiState,
    game_list_state: GameListUiState,
    setting_state: SettingUiState,
    language_state: LanguageUiState,
    memory_state: MemoryUiState,
    mod_list_state: ModListUiState,
    needed_size_mode: NeededSizeMode,
    action_map: UiActionMap,
    game_session: Option<GameSession>,
}

/// 加载初始 Home UI 页面。
pub(crate) fn load_home_page(
    lua_runtime: &LuaRuntimeState,
    loaded_resources: &LoadedResources,
) -> UiRuntimeResult<ActiveUiPage> {
    let official_ui_package = loaded_resources
        .official_ui_registry
        .packages
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "official UI package not found"))?;
    let page_key = UiPageKey::Home;
    let entry_path = entry_path(&official_ui_package.manifest, page_key.as_str())?;
    let script_path = resolve_script_path(&official_ui_package.root_dir, entry_path.as_str())?;
    let action_map =
        UiActionMap::from_manifest_page(&official_ui_package.manifest, page_key.as_str());

    let lua = &lua_runtime.lua_runtime_environment.lua;
    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    let terminal_size = loaded_resources.initialized_environment.terminal_size;
    host_bridge.set_runtime_context(LuaRuntimeContext {
        consumer: LuaRuntimeConsumer::OfficialUiPackage,
        current_game: None,
        current_ui_actions: action_map.actions_value(),
        current_script_root: Some(official_ui_package.root_dir.join("scripts")),
        language_code: loaded_resources.persistent_data.language_code.clone(),
        keybinds: loaded_resources.persistent_data.keybinds.clone(),
        best_scores: loaded_resources.persistent_data.best_scores.clone(),
        mod_state: loaded_resources.persistent_data.mod_state.clone(),
        launch_mode: Default::default(),
        terminal_size,
    });
    host_bridge.resize_canvas(terminal_size)?;

    let source = fs::read_to_string(&script_path)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())?;
    lua.load(source.as_str())
        .set_name(script_path.to_string_lossy().as_ref())
        .exec()?;
    callback_api::validate_required_callbacks(lua, ApiScope::official_ui_package())?;

    let mut home_state = HomeUiState::new();
    home_state.reset_lua_state();
    let mut game_list_state = GameListUiState::new(
        loaded_resources.game_module_registry.clone(),
        loaded_resources.persistent_data.best_scores.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    game_list_state.reset_lua_state();
    let mut setting_state = SettingUiState::new();
    setting_state.reset_lua_state();
    let mut language_state = LanguageUiState::new(
        loaded_resources.persistent_data.language_code.clone(),
        loaded_resources.cache_data.language_ui_texts.clone(),
    );
    language_state.reset_lua_state();
    let mut memory_state = MemoryUiState::new();
    memory_state.reset_lua_state();
    let mut mod_list_state = ModListUiState::new(
        loaded_resources.game_module_registry.clone(),
        loaded_resources.persistent_data.mod_state.clone(),
        loaded_resources.persistent_data.language_code.clone(),
    );
    mod_list_state.reset_lua_state();
    Ok(ActiveUiPage {
        package_root: official_ui_package.root_dir.clone(),
        manifest: official_ui_package.manifest.clone(),
        game_modules: loaded_resources.game_module_registry.games.clone(),
        page_key,
        page_needs_reload: false,
        home_state,
        game_list_state,
        setting_state,
        language_state,
        memory_state,
        mod_list_state,
        needed_size_mode: NeededSizeMode::Root,
        action_map,
        game_session: None,
    })
}

impl ActiveUiPage {
    /// 根据物理按键查找当前 UI 页面动作。
    pub fn action_for_key(&self, key: &str) -> Option<String> {
        self.action_map.action_for_key(key)
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
}

/// 确保当前已加载指定 UI 页面脚本。
pub(crate) fn ensure_page(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    page_key: UiPageKey,
) -> UiRuntimeResult<()> {
    if active_ui_page.page_key == page_key && !active_ui_page.page_needs_reload {
        return Ok(());
    }

    let action_map = UiActionMap::from_manifest_page(&active_ui_page.manifest, page_key.as_str());
    switch_to_ui_context(lua_runtime, active_ui_page, action_map.actions_value())?;
    load_page_script(lua_runtime, active_ui_page, page_key)?;
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
    if page_key == UiPageKey::SettingLanguage {
        active_ui_page.language_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingMemory {
        active_ui_page.memory_state.reset_lua_state();
    }
    if page_key == UiPageKey::SettingMods {
        active_ui_page.mod_list_state.reset_lua_state();
    }
    if page_key == UiPageKey::StorageDetails {
        active_ui_page.memory_state.root_state =
            crate::host_engine::runtime::ui_state::memory_state::MemoryRootState::new(
                active_ui_page.memory_state.root_state.select,
            );
    }

    Ok(())
}

/// 将事件传递给当前 UI 页面。
pub(crate) fn handle_event(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    event: LuaEvent,
) -> UiRuntimeResult<()> {
    let lua = &lua_runtime.lua_runtime_environment.lua;
    sync_page_script_state(lua_runtime, active_ui_page)?;
    let handle_event: Function = lua.globals().get("handle_event")?;
    let lua_state = match active_ui_page.page_key {
        UiPageKey::Home => active_ui_page.home_state.lua_state.to_lua_table(lua)?,
        UiPageKey::GameList => active_ui_page
            .game_list_state
            .root_state
            .to_lua_table(lua)?,
        UiPageKey::SettingMods => active_ui_page.mod_list_state.root_state.to_lua_table(lua)?,
        UiPageKey::Setting => active_ui_page.setting_state.lua_state.to_lua_table(lua)?,
        UiPageKey::SettingLanguage => active_ui_page.language_state.lua_state.to_lua_table(lua)?,
        UiPageKey::SettingMemory => active_ui_page.memory_state.lua_state.to_lua_table(lua)?,
        UiPageKey::WarningNeededSize => {
            let table = lua.create_table()?;
            table.set("exit", false)?;
            table.set("mode", active_ui_page.needed_size_mode.as_str())?;
            table
        }
        _ => lua.create_table()?,
    };
    let event_table = event.into_lua_table(lua)?;
    let returned_state = handle_event.call::<Value>((lua_state, event_table))?;

    if active_ui_page.page_key != UiPageKey::Home {
        if active_ui_page.page_key == UiPageKey::GameList {
            let lua_state = GameListLuaState::from_lua_value(returned_state)?;
            handle_game_list_lua_state(lua_runtime, active_ui_page, host_state_machine, lua_state)?;
            return Ok(());
        }
        if active_ui_page.page_key == UiPageKey::Setting {
            let lua_state = SettingLuaState::from_lua_value(returned_state)?;
            handle_setting_lua_state(active_ui_page, host_state_machine, lua_state);
            return Ok(());
        }
        if active_ui_page.page_key == UiPageKey::SettingMods {
            let lua_state = ModListLuaState::from_lua_value(returned_state)?;
            handle_mod_list_lua_state(lua_runtime, active_ui_page, host_state_machine, lua_state)?;
            return Ok(());
        }
        if active_ui_page.page_key == UiPageKey::SettingLanguage {
            let lua_state = LanguageLuaState::from_lua_value(returned_state)?;
            handle_language_lua_state(
                &lua_runtime.lua_runtime_environment.host_bridge,
                active_ui_page,
                host_state_machine,
                lua_state,
            )?;
            return Ok(());
        }
        if active_ui_page.page_key == UiPageKey::SettingMemory {
            let lua_state = MemoryLuaState::from_lua_value(returned_state)?;
            handle_memory_lua_state(active_ui_page, host_state_machine, lua_state);
            return Ok(());
        }
        if active_ui_page.page_key == UiPageKey::StorageDetails {
            handle_storage_details_lua_state(host_state_machine, returned_state)?;
            return Ok(());
        }
        if matches!(
            active_ui_page.page_key,
            UiPageKey::WarningClearCache | UiPageKey::WarningClearData
        ) {
            handle_clear_warning_lua_state(active_ui_page, host_state_machine, returned_state)?;
            return Ok(());
        }
        handle_non_home_lua_state(
            &lua_runtime.lua_runtime_environment.host_bridge,
            active_ui_page.page_key,
            returned_state,
        )?;
        return Ok(());
    }

    let lua_state = HomeLuaState::from_lua_value(returned_state)?;
    if lua_state.exit {
        lua_runtime
            .lua_runtime_environment
            .host_bridge
            .push_message(HostLuaMessage::ExitGame);
        return Ok(());
    }

    if let Some(confirm_action) = active_ui_page.home_state.apply_lua_state(lua_state) {
        handle_home_confirm_action(host_state_machine, confirm_action);
    }
    Ok(())
}

/// 同步 Lua 脚本内部的页面缓存状态。
///
/// 部分官方 UI 脚本会在 render 阶段缓存 root_state 供 handle_event 使用。
/// 页面刚切换后，首个输入事件可能早于首次 render 到达，因此这里先用当前 root_state
/// 做一次脚本级同步，避免事件处理读取到空缓存。
fn sync_page_script_state(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
) -> UiRuntimeResult<()> {
    if active_ui_page.page_key != UiPageKey::GameList
        && active_ui_page.page_key != UiPageKey::SettingMods
    {
        return Ok(());
    }

    let lua = &lua_runtime.lua_runtime_environment.lua;
    let render: Function = lua.globals().get("render")?;
    let root_state = if active_ui_page.page_key == UiPageKey::GameList {
        active_ui_page
            .game_list_state
            .root_state
            .to_lua_table(lua)?
    } else {
        active_ui_page.mod_list_state.root_state.to_lua_table(lua)?
    };
    render.call::<()>(root_state)?;
    Ok(())
}

/// 渲染当前 UI 页面。
pub(crate) fn render(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
) -> UiRuntimeResult<()> {
    let lua = &lua_runtime.lua_runtime_environment.lua;
    let root_state = match active_ui_page.page_key {
        UiPageKey::Home => active_ui_page.home_state.root_state.to_lua_table(lua)?,
        UiPageKey::GameList => active_ui_page
            .game_list_state
            .root_state
            .to_lua_table(lua)?,
        UiPageKey::SettingMods => active_ui_page.mod_list_state.root_state.to_lua_table(lua)?,
        UiPageKey::Setting => active_ui_page.setting_state.root_state.to_lua_table(lua)?,
        UiPageKey::SettingLanguage => active_ui_page.language_state.root_state.to_lua_table(lua)?,
        UiPageKey::SettingMemory => active_ui_page.memory_state.root_state.to_lua_table(lua)?,
        UiPageKey::StorageDetails => active_ui_page.memory_state.root_state.to_lua_table(lua)?,
        UiPageKey::WarningClearCache => clear_cache_root_state(lua)?,
        UiPageKey::WarningClearData => clear_data_root_state(lua)?,
        _ => lua.create_table()?,
    };
    let render: Function = lua.globals().get("render")?;
    render.call::<()>(root_state)?;
    Ok(())
}

/// 渲染尺寸警告页面。
pub(crate) fn render_needed_size(
    lua_runtime: &LuaRuntimeState,
    needed_size_state: NeededSizeRootState,
) -> UiRuntimeResult<()> {
    let lua = &lua_runtime.lua_runtime_environment.lua;
    let root_state = needed_size_state.to_lua_table(lua)?;
    let render: Function = lua.globals().get("render")?;
    render.call::<()>(root_state)?;
    Ok(())
}

fn entry_path(manifest: &JsonValue, entry_name: &str) -> UiRuntimeResult<String> {
    manifest
        .get("entry")
        .and_then(JsonValue::as_object)
        .and_then(|entry| entry.get(entry_name))
        .and_then(JsonValue::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("official UI entry `{entry_name}` is missing"),
            )
            .into()
        })
}

fn resolve_script_path(package_root: &Path, logical_path: &str) -> UiRuntimeResult<PathBuf> {
    let trimmed_path = logical_path.trim();
    if trimmed_path.is_empty() || Path::new(trimmed_path).is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid UI script path: {trimmed_path}"),
        )
        .into());
    }

    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(trimmed_path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid UI script path: {trimmed_path}"),
                )
                .into());
            }
        }
    }

    Ok(package_root.join("scripts").join(clean_path))
}

fn load_page_script(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
    page_key: UiPageKey,
) -> UiRuntimeResult<()> {
    let entry_path = entry_path(&active_ui_page.manifest, page_key.as_str())?;
    let script_path = resolve_script_path(&active_ui_page.package_root, entry_path.as_str())?;
    let source = fs::read_to_string(&script_path)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())?;
    lua_runtime
        .lua_runtime_environment
        .lua
        .load(source.as_str())
        .set_name(script_path.to_string_lossy().as_ref())
        .exec()?;
    callback_api::validate_required_callbacks(
        &lua_runtime.lua_runtime_environment.lua,
        ApiScope::official_ui_package(),
    )?;
    Ok(())
}

fn switch_to_ui_context(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
    current_ui_actions: JsonValue,
) -> UiRuntimeResult<()> {
    let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
    let current_context = host_bridge.runtime_context();
    host_bridge.set_runtime_context(LuaRuntimeContext {
        consumer: LuaRuntimeConsumer::OfficialUiPackage,
        current_game: None,
        current_ui_actions,
        current_script_root: Some(active_ui_page.package_root.join("scripts")),
        language_code: current_context.language_code,
        keybinds: current_context.keybinds,
        best_scores: current_context.best_scores,
        mod_state: current_context.mod_state,
        launch_mode: current_context.launch_mode,
        terminal_size: current_context.terminal_size,
    });
    api::install_runtime_apis(
        &lua_runtime.lua_runtime_environment.lua,
        ApiScope::official_ui_package(),
        host_bridge.clone(),
    )?;
    Ok(())
}

fn handle_home_confirm_action(
    host_state_machine: &mut HostStateMachine,
    confirm_action: HomeConfirmAction,
) {
    match confirm_action {
        HomeConfirmAction::GameList => {
            host_state_machine.top_level_state = TopLevelState::GameList;
        }
        HomeConfirmAction::Setting => {
            host_state_machine.top_level_state = TopLevelState::Setting;
            host_state_machine.setting_state = SettingState::Hub;
        }
        HomeConfirmAction::About => {
            host_state_machine.top_level_state = TopLevelState::About;
        }
        HomeConfirmAction::ContinueGame => {
            // TODO: 接入状态机后在这里切换到对应顶层页面。
        }
    }
}

fn handle_setting_lua_state(
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    lua_state: SettingLuaState,
) {
    match active_ui_page.setting_state.apply_lua_state(lua_state) {
        SettingLuaAction::None => {}
        SettingLuaAction::Back => {
            host_state_machine.top_level_state = TopLevelState::Home;
            host_state_machine.setting_state = SettingState::Hub;
        }
        SettingLuaAction::Confirm(confirm_action) => {
            host_state_machine.setting_state = confirm_action.to_setting_state();
        }
    }
}

fn handle_game_list_lua_state(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    lua_state: GameListLuaState,
) -> UiRuntimeResult<()> {
    match active_ui_page.game_list_state.apply_lua_state(lua_state) {
        GameListLuaAction::None => {}
        GameListLuaAction::Back => {
            host_state_machine.top_level_state = TopLevelState::Home;
        }
        GameListLuaAction::Confirm(game_uid) => {
            if let Some(game_module) = active_ui_page
                .game_modules
                .iter()
                .find(|game_module| game_module.uid == game_uid)
                .cloned()
            {
                active_ui_page.game_session =
                    Some(script_loader::load_new_game(lua_runtime, game_module)?);
                host_state_machine.game_list_state = GameListState::Game;
            }
        }
    }
    Ok(())
}

fn handle_mod_list_lua_state(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    lua_state: ModListLuaState,
) -> UiRuntimeResult<()> {
    match active_ui_page.mod_list_state.apply_lua_state(lua_state) {
        ModListLuaAction::None => {}
        ModListLuaAction::Back => {
            host_state_machine.setting_state = SettingState::Hub;
        }
        ModListLuaAction::StateChanged(mod_state) => {
            let host_bridge = &lua_runtime.lua_runtime_environment.host_bridge;
            let mut current_context = host_bridge.runtime_context();
            current_context.mod_state = mod_state;
            host_bridge.set_runtime_context(current_context);
        }
    }
    Ok(())
}

fn handle_language_lua_state(
    host_bridge: &HostLuaBridge,
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    lua_state: LanguageLuaState,
) -> UiRuntimeResult<()> {
    match active_ui_page.language_state.apply_lua_state(lua_state) {
        LanguageLuaAction::None => {}
        LanguageLuaAction::Back => {
            host_state_machine.setting_state = SettingState::Hub;
        }
        LanguageLuaAction::Confirm(language_code) => {
            persist_language_code(language_code.as_str())?;
            crate::host_engine::boot::i18n::i18n::reload(language_code.as_str())?;
            active_ui_page.home_state.refresh_language();
            active_ui_page
                .game_list_state
                .refresh_language(language_code.clone());
            active_ui_page
                .mod_list_state
                .refresh_language(language_code.clone());
            active_ui_page.setting_state.refresh_language();
            active_ui_page.memory_state.refresh_language();
            host_bridge.set_language_code(language_code);
        }
    }
    Ok(())
}

fn handle_memory_lua_state(
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    lua_state: MemoryLuaState,
) {
    match active_ui_page.memory_state.apply_lua_state(lua_state) {
        MemoryLuaAction::None => {}
        MemoryLuaAction::Back => {
            host_state_machine.setting_state = SettingState::Hub;
        }
        MemoryLuaAction::Confirm(_confirm_action) => match _confirm_action {
            MemoryConfirmAction::ClearCache => {
                host_state_machine.dialog_state = Some(DialogState::ClearCacheWarning);
            }
            MemoryConfirmAction::ClearData => {
                host_state_machine.dialog_state = Some(DialogState::ClearDataWarning);
            }
            MemoryConfirmAction::ShowStorageDetails => {
                host_state_machine.setting_state = SettingState::StorageDetails;
            }
        },
    }
}

fn handle_clear_warning_lua_state(
    active_ui_page: &mut ActiveUiPage,
    host_state_machine: &mut HostStateMachine,
    returned_state: Value,
) -> UiRuntimeResult<()> {
    let Value::Table(table) = returned_state else {
        return Ok(());
    };
    let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
    let back = table.get::<Option<bool>>("back")?.unwrap_or(false);

    if confirm {
        match host_state_machine.dialog_state {
            Some(DialogState::ClearCacheWarning) => {
                crate::host_engine::runtime::memory_cleanup::clear_cache()?;
            }
            Some(DialogState::ClearDataWarning) => {
                crate::host_engine::runtime::memory_cleanup::clear_data()?;
            }
            _ => {}
        }
        active_ui_page.memory_state.reset_lua_state();
        host_state_machine.dialog_state = None;
        host_state_machine.setting_state = SettingState::Memory;
        return Ok(());
    }

    if back {
        host_state_machine.dialog_state = None;
        host_state_machine.setting_state = SettingState::Memory;
    }

    Ok(())
}

fn clear_cache_root_state(lua: &mlua::Lua) -> mlua::Result<Table> {
    let text = crate::host_engine::boot::i18n::text();
    let language = lua.create_table()?;
    language.set("CLEAR_CACHE_CONFIRM", text.key.clear_cache_confirm)?;
    language.set("CLEAR_CACHE_CANCEL", text.key.clear_cache_cancel)?;
    language.set("CLEAR_CACHE_TITLE", text.clear_cache.title)?;
    language.set("CLEAR_CACHE_WARN", text.clear_cache.warn)?;
    language.set("CLEAR_CACHE_CACHE_PATH", text.clear_cache.cache_path)?;
    language.set("CLEAR_CACHE_LOG_PATH", text.clear_cache.log_path)?;
    language.set("CLEAR_CACHE_SECOND", text.clear_cache.second)?;

    let root_dir = root_dir();
    let dir = lua.create_table()?;
    dir.set(
        "cache_dir",
        root_dir.join("data").join("cache").display().to_string(),
    )?;
    dir.set(
        "log_dir",
        root_dir.join("data").join("log").display().to_string(),
    )?;

    let table = lua.create_table()?;
    table.set("language", language)?;
    table.set("dir", dir)?;
    Ok(table)
}

fn clear_data_root_state(lua: &mlua::Lua) -> mlua::Result<Table> {
    let text = crate::host_engine::boot::i18n::text();
    let language = lua.create_table()?;
    language.set("CLEAR_DATA_CONFIRM", text.key.clear_data_confirm)?;
    language.set("CLEAR_DATA_CANCEL", text.key.clear_data_cancel)?;
    language.set("CLEAR_DATA_TITLE", text.clear_data.title)?;
    language.set("CLEAR_DATA_WARN", text.clear_data.warn)?;
    language.set("CLEAR_DATA_PATH", text.clear_data.path)?;
    language.set("CLEAR_DATA_SECOND", text.clear_data.second)?;

    let dir = lua.create_table()?;
    dir.set("data_dir", root_dir().join("data").display().to_string())?;

    let table = lua.create_table()?;
    table.set("language", language)?;
    table.set("dir", dir)?;
    Ok(table)
}

fn handle_storage_details_lua_state(
    host_state_machine: &mut HostStateMachine,
    returned_state: Value,
) -> UiRuntimeResult<()> {
    let Value::Table(table) = returned_state else {
        return Ok(());
    };
    let back = table.get::<Option<bool>>("back")?.unwrap_or(false);
    if back {
        host_state_machine.setting_state = SettingState::Memory;
    }
    Ok(())
}

fn persist_language_code(language_code: &str) -> UiRuntimeResult<()> {
    let path = root_dir().join("data/profiles/language.txt");
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, language_code)?;
    Ok(())
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

fn handle_non_home_lua_state(
    host_bridge: &HostLuaBridge,
    page_key: UiPageKey,
    returned_state: Value,
) -> UiRuntimeResult<()> {
    let Value::Table(table) = returned_state else {
        return Ok(());
    };
    let exit = table.get::<Option<bool>>("exit")?.unwrap_or(false);
    if !exit {
        return Ok(());
    }

    match page_key {
        UiPageKey::WarningNeededSize => {
            let mode = table
                .get::<Option<String>>("mode")?
                .unwrap_or_else(|| "root".to_string());
            if mode == "game" {
                // TODO: game 模式接入游戏运行态后，在这里返回游戏列表。
            } else {
                host_bridge.push_message(HostLuaMessage::ExitGame);
            }
        }
        _ => {}
    }
    Ok(())
}
