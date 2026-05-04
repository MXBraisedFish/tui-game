//! 宿主与 Lua 通信桥占位

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::GameModule;
use crate::host_engine::boot::preload::init_environment::TerminalSize;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::CanvasState;
use crate::host_engine::boot::preload::lua_runtime::api::random_support::random_store::RandomStore;
use crate::host_engine::boot::preload::lua_runtime::api::timer_support::timer_store::TimerStore;

/// 宿主与 Lua 通信桥。
///
/// 当前只保留消息队列占位，后续公开自定义 API 时由 API 层写入消息，运行时事件循环消费消息。
#[derive(Clone, Debug)]
pub struct HostLuaBridge {
    messages: Arc<Mutex<Vec<HostLuaMessage>>>,
    runtime_context: Arc<Mutex<LuaRuntimeContext>>,
    canvas_state: Arc<Mutex<CanvasState>>,
    timer_store: Arc<Mutex<TimerStore>>,
    random_store: Arc<Mutex<RandomStore>>,
    started_at: Arc<Instant>,
}

/// Lua 向宿主发出的消息。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostLuaMessage {
    ExitGame,
    SkipEventQueue,
    ClearEventQueue,
    RenderNow,
    SaveBestScore,
    SaveGame,
}

/// Lua API 运行上下文。
#[derive(Clone, Debug, Default)]
pub struct LuaRuntimeContext {
    pub consumer: LuaRuntimeConsumer,
    pub current_game: Option<GameModule>,
    pub language_code: String,
    pub keybinds: Value,
    pub best_scores: Value,
    pub mod_state: Value,
    pub launch_mode: LaunchMode,
    pub terminal_size: TerminalSize,
}

/// Lua API 使用方。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LuaRuntimeConsumer {
    #[default]
    GamePackage,
    OfficialUiPackage,
}

/// 游戏启动模式。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LaunchMode {
    #[default]
    New,
    Continue,
}

impl LaunchMode {
    /// 转为 Lua API 返回字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Continue => "continue",
        }
    }
}

impl HostLuaBridge {
    /// 创建空通信桥。
    pub fn new() -> Self {
        Self::default()
    }

    /// 写入一条 Lua 到宿主的消息。
    pub fn push_message(&self, message: HostLuaMessage) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(message);
        }
    }

    /// 取出所有待处理消息。
    pub fn drain_messages(&self) -> Vec<HostLuaMessage> {
        self.messages
            .lock()
            .map(|mut messages| messages.drain(..).collect())
            .unwrap_or_default()
    }

    /// 设置 Lua API 运行上下文。
    pub fn set_runtime_context(&self, runtime_context: LuaRuntimeContext) {
        if let Ok(mut current_context) = self.runtime_context.lock() {
            *current_context = runtime_context;
        }
    }

    /// 读取 Lua API 运行上下文快照。
    pub fn runtime_context(&self) -> LuaRuntimeContext {
        self.runtime_context
            .lock()
            .map(|runtime_context| runtime_context.clone())
            .unwrap_or_default()
    }

    /// 操作当前虚拟画布。
    pub fn with_canvas_state(&self, operation: impl FnOnce(&mut CanvasState)) -> mlua::Result<()> {
        let mut canvas_state = self
            .canvas_state
            .lock()
            .map_err(|_| mlua::Error::external("canvas context is poisoned"))?;
        operation(&mut canvas_state);
        Ok(())
    }

    /// 读取当前虚拟画布快照。
    pub fn canvas_state(&self) -> CanvasState {
        self.canvas_state
            .lock()
            .map(|canvas_state| canvas_state.clone())
            .unwrap_or_default()
    }

    /// 操作当前计时器仓库。
    pub fn with_timer_store<T>(
        &self,
        operation: impl FnOnce(&mut TimerStore) -> mlua::Result<T>,
    ) -> mlua::Result<T> {
        let mut timer_store = self
            .timer_store
            .lock()
            .map_err(|_| mlua::Error::external("timer store is poisoned"))?;
        operation(&mut timer_store)
    }

    /// 当前运行时长，毫秒。
    pub fn running_time_ms(&self) -> i64 {
        self.started_at.elapsed().as_millis() as i64
    }

    /// 操作当前随机数生成器仓库。
    pub fn with_random_store<T>(
        &self,
        operation: impl FnOnce(&mut RandomStore) -> mlua::Result<T>,
    ) -> mlua::Result<T> {
        let mut random_store = self
            .random_store
            .lock()
            .map_err(|_| mlua::Error::external("random store is poisoned"))?;
        operation(&mut random_store)
    }
}

impl Default for HostLuaBridge {
    fn default() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            runtime_context: Arc::new(Mutex::new(LuaRuntimeContext::default())),
            canvas_state: Arc::new(Mutex::new(CanvasState::default())),
            timer_store: Arc::new(Mutex::new(TimerStore::default())),
            random_store: Arc::new(Mutex::new(RandomStore::default())),
            started_at: Arc::new(Instant::now()),
        }
    }
}
