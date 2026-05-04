//! 运行阶段官方 UI Lua 页面执行

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use mlua::{Function, Value};
use serde_json::Value as JsonValue;

use crate::LoadedResources;
use crate::LuaRuntimeState;
use crate::host_engine::boot::preload::lua_runtime::api::LuaEvent;
use crate::host_engine::boot::preload::lua_runtime::api::ApiScope;
use crate::host_engine::boot::preload::lua_runtime::api::callback_api;
use crate::host_engine::boot::preload::lua_runtime::{
    HostLuaBridge, HostLuaMessage, LuaRuntimeConsumer, LuaRuntimeContext,
};
use crate::host_engine::boot::preload::state_machine::{
    HostStateMachine, SettingState, TopLevelState,
};
use crate::host_engine::runtime::ui_state::action_map::UiActionMap;
use crate::host_engine::runtime::ui_state::home_state::HomeUiState;
use crate::host_engine::runtime::ui_state::lua_state::HomeLuaState;
use crate::host_engine::runtime::ui_state::needed_size_state::{
    NeededSizeMode, NeededSizeRootState,
};
use crate::host_engine::runtime::ui_state::root_state::HomeConfirmAction;
use crate::host_engine::runtime::ui_state::setting_state::{
    SettingLuaAction, SettingLuaState, SettingUiState,
};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

type UiRuntimeResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 当前激活的官方 UI 页面实例。
pub struct ActiveUiPage {
    package_root: PathBuf,
    manifest: JsonValue,
    page_key: UiPageKey,
    home_state: HomeUiState,
    setting_state: SettingUiState,
    needed_size_mode: NeededSizeMode,
    action_map: UiActionMap,
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
    let action_map = UiActionMap::from_manifest_page(&official_ui_package.manifest, page_key.as_str());

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
    let mut setting_state = SettingUiState::new();
    setting_state.reset_lua_state();
    Ok(ActiveUiPage {
        package_root: official_ui_package.root_dir.clone(),
        manifest: official_ui_package.manifest.clone(),
        page_key,
        home_state,
        setting_state,
        needed_size_mode: NeededSizeMode::Root,
        action_map,
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
}

/// 确保当前已加载指定 UI 页面脚本。
pub(crate) fn ensure_page(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &mut ActiveUiPage,
    page_key: UiPageKey,
) -> UiRuntimeResult<()> {
    if active_ui_page.page_key == page_key {
        return Ok(());
    }

    let action_map = UiActionMap::from_manifest_page(&active_ui_page.manifest, page_key.as_str());
    lua_runtime
        .lua_runtime_environment
        .host_bridge
        .set_current_ui_actions(action_map.actions_value());
    load_page_script(lua_runtime, active_ui_page, page_key)?;
    active_ui_page.page_key = page_key;
    active_ui_page.action_map = action_map;

    if page_key == UiPageKey::Home {
        active_ui_page.home_state.reset_lua_state();
    }
    if page_key == UiPageKey::Setting {
        active_ui_page.setting_state.reset_lua_state();
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
    let handle_event: Function = lua.globals().get("handle_event")?;
    let lua_state = match active_ui_page.page_key {
        UiPageKey::Home => active_ui_page.home_state.lua_state.to_lua_table(lua)?,
        UiPageKey::Setting => active_ui_page.setting_state.lua_state.to_lua_table(lua)?,
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
        if active_ui_page.page_key == UiPageKey::Setting {
            let lua_state = SettingLuaState::from_lua_value(returned_state)?;
            handle_setting_lua_state(active_ui_page, host_state_machine, lua_state);
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

/// 渲染当前 UI 页面。
pub(crate) fn render(
    lua_runtime: &LuaRuntimeState,
    active_ui_page: &ActiveUiPage,
) -> UiRuntimeResult<()> {
    let lua = &lua_runtime.lua_runtime_environment.lua;
    let root_state = match active_ui_page.page_key {
        UiPageKey::Home => active_ui_page.home_state.root_state.to_lua_table(lua)?,
        UiPageKey::Setting => active_ui_page.setting_state.root_state.to_lua_table(lua)?,
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

fn handle_home_confirm_action(host_state_machine: &mut HostStateMachine, confirm_action: HomeConfirmAction) {
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
            let mode = table.get::<Option<String>>("mode")?.unwrap_or_else(|| "root".to_string());
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
