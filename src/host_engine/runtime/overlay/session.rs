//! 覆盖层 Lua 会话。

use std::fs;

use mlua::{RegistryKey, Value};

use crate::host_engine::boot::preload::lua_runtime::api::{ApiScope, callback_api};
use crate::host_engine::boot::preload::lua_runtime::{
    HostLuaBridge, LuaRuntimeConsumer, LuaRuntimeContext, create_scoped_runtime,
};
use crate::host_engine::boot::preload::overlay_modules::{OverlayKind, OverlayPackage};
use crate::host_engine::runtime::overlay::script_path;

type OverlayResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 当前覆盖层类别。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlaySessionKind {
    Saver,
    Boss,
}

impl OverlaySessionKind {
    fn api_scope(self) -> ApiScope {
        match self {
            Self::Saver => ApiScope::saver_package(),
            Self::Boss => ApiScope::boss_package(),
        }
    }

    fn consumer(self) -> LuaRuntimeConsumer {
        match self {
            Self::Saver => LuaRuntimeConsumer::SaverPackage,
            Self::Boss => LuaRuntimeConsumer::BossPackage,
        }
    }
}

impl From<OverlayKind> for OverlaySessionKind {
    fn from(value: OverlayKind) -> Self {
        match value {
            OverlayKind::Saver => Self::Saver,
            OverlayKind::Boss => Self::Boss,
        }
    }
}

/// 一个独立 Lua VM 中运行的覆盖层。
pub struct OverlaySession {
    kind: OverlaySessionKind,
    package: OverlayPackage,
    lua_runtime: crate::host_engine::boot::preload::lua_runtime::LuaRuntimeEnvironment,
    state_key: RegistryKey,
    previous_context: LuaRuntimeContext,
}

impl OverlaySession {
    pub fn start(host_bridge: &HostLuaBridge, package: OverlayPackage) -> OverlayResult<Self> {
        let kind = OverlaySessionKind::from(package.kind);
        let previous_context = host_bridge.runtime_context();
        let script_root = package.root_dir.join("scripts");
        let script_path =
            script_path::resolve_script_path(&script_root, package.manifest.entry.as_str())?;

        let mut overlay_context = previous_context.clone();
        overlay_context.consumer = kind.consumer();
        overlay_context.current_game = None;
        overlay_context.current_overlay = Some(package.clone());
        overlay_context.current_ui_actions = serde_json::Value::Null;
        overlay_context.current_script_root = Some(script_root);
        host_bridge.set_runtime_context(overlay_context);

        let lua_runtime = create_scoped_runtime(host_bridge.clone(), kind.api_scope())?;
        let source = fs::read_to_string(&script_path)
            .map(|text| text.trim_start_matches('\u{feff}').to_string())?;
        lua_runtime
            .lua
            .load(source.as_str())
            .set_name(script_path.to_string_lossy().as_ref())
            .exec()?;
        callback_api::validate_required_callbacks(&lua_runtime.lua, kind.api_scope())?;

        let state_key = lua_runtime
            .lua
            .create_registry_value(Value::Table(lua_runtime.lua.create_table()?))?;

        Ok(Self {
            kind,
            package,
            lua_runtime,
            state_key,
            previous_context,
        })
    }

    pub fn kind(&self) -> OverlaySessionKind {
        self.kind
    }

    pub fn update_and_render(&mut self) -> OverlayResult<()> {
        let new_state_key = callback_api::call_update(&self.lua_runtime.lua, &self.state_key)?;
        self.state_key = new_state_key;
        callback_api::call_render(&self.lua_runtime.lua, &self.state_key)?;
        Ok(())
    }

    pub fn stop(self, host_bridge: &HostLuaBridge) {
        let mut previous_context = self.previous_context;
        previous_context.terminal_size = host_bridge.runtime_context().terminal_size;
        host_bridge.set_runtime_context(previous_context);
    }

    #[allow(dead_code)]
    pub fn package(&self) -> &OverlayPackage {
        &self.package
    }
}
