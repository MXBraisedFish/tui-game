// Lua 游戏引擎核心，负责游戏生命周期管理（初始化、事件循环、渲染、退出）。它是连接宿主 Rust 环境与 Lua 游戏脚本的桥梁，提供运行时桥接、事件分发、帧率控制和命令处理

use std::fs; // 读取 Lua 入口脚本文件
use std::sync::{Arc, Mutex}; // 线程安全的共享状态（画布、命令队列、计时器、随机数存储）
use std::time::{Duration, Instant}; // 帧计时、延迟计算

use anyhow::{Context, Result, anyhow}; // 错误处理
use crossterm::event::{self, Event, KeyEventKind}; // 终端事件轮询和键盘事件
use mlua::{Lua, RegistryKey}; // Lua 虚拟机核心类型

use crate::app::i18n; // 国际化错误消息
use crate::core::command::RuntimeCommand; // 游戏→宿主命令枚举
use crate::core::event::InputEvent; // 宿主→游戏事件枚举
use crate::core::key::semantic_key_source; // 全局键盘输入源
use crate::core::runtime::LaunchMode; // 游戏启动模式
use crate::core::screen::Canvas; // 虚拟画布
use crate::game::registry::GameDescriptor; // 游戏描述符
use crate::lua::{api, sandbox}; // Lua API 安装和沙箱
use crate::terminal::{renderer, size_watcher}; // 终端渲染和尺寸约束检查
use crate::utils::host_log; // 日志记录

const DEFAULT_TARGET_FPS: u16 = 60; // 当游戏未指定或指定非法 FPS 时的默认值
const MAX_EVENTS_PER_FRAME: usize = 256; // 单帧最多处理的事件数量，防止事件风暴
const MAX_CATCH_UP_TICKS_PER_FRAME: usize = 8; // 最大追赶滴答数：当帧渲染/事件处理耗时过长时，最多追补的逻辑帧数

// 用于将宿主资源传递给 Lua API 函数
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct RuntimeBridges {
    pub(crate) canvas: Arc<Mutex<Canvas>>,
    pub(crate) commands: Arc<Mutex<Vec<RuntimeCommand>>>,
    pub(crate) resize_flag: Arc<Mutex<bool>>,
    pub(crate) timers: Arc<Mutex<api::direct_timer_api::TimerStore>>,
    pub(crate) randoms: Arc<Mutex<api::direct_random_api::RandomStore>>,
    pub(crate) game: GameDescriptor,
    pub(crate) launch_mode: LaunchMode,
    pub(crate) started_at: Instant,
}

// 游戏引擎实例，封装了 Lua 状态和共享资源
pub struct LuaGameEngine {
    lua: Lua,
    state_key: RegistryKey,
    game: GameDescriptor,
    bridges: RuntimeBridges,
}

impl LuaGameEngine {
    // 创建 Lua 实例，安装沙箱，安装 API，加载并执行入口脚本，验证必需回调，初始化游戏状态
    pub fn new(game: GameDescriptor, launch_mode: LaunchMode) -> Result<Self> {
        let _log_object_guard = host_log::scoped_log_object(game.id.clone());
        let lua = Lua::new();
        sandbox::install_sandbox(&lua).map_err(anyhow_lua_error)?;

        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let canvas = Arc::new(Mutex::new(Canvas::new(width, height)));
        let commands = Arc::new(Mutex::new(Vec::new()));
        let resize_flag = Arc::new(Mutex::new(false));
        let timers = Arc::new(Mutex::new(api::direct_timer_api::TimerStore::default()));
        let randoms = Arc::new(Mutex::new(api::direct_random_api::RandomStore::default()));
        let bridges = RuntimeBridges {
            canvas: Arc::clone(&canvas),
            commands: Arc::clone(&commands),
            resize_flag: Arc::clone(&resize_flag),
            timers: Arc::clone(&timers),
            randoms: Arc::clone(&randoms),
            game: game.clone(),
            launch_mode,
            started_at: Instant::now(),
        };
        api::install_runtime_apis(&lua, &bridges).map_err(anyhow_lua_error)?;

        let source = fs::read_to_string(&game.entry_path).with_context(|| {
            i18n::t_or("host.error.read_script_failed", "Failed to read script: {path}")
                .replace("{path}", &game.entry_path.display().to_string())
        })?;
        lua.load(source.trim_start_matches('\u{feff}'))
            .set_name(game.entry_path.to_string_lossy().as_ref())
            .exec()
            .map_err(|err| {
                anyhow!(
                    "{}",
                    i18n::t_or("host.error.execute_script_failed", "Failed to execute script: {path}")
                        .replace("{path}", &game.entry_path.display().to_string())
                        + ": "
                        + &err.to_string()
                )
            })?;

        api::callback_api::validate_required_callbacks(&lua, &game)?;
        let state_key = api::callback_api::initialize_state(&lua, &game, launch_mode)?;

        Ok(Self {
            lua,
            state_key,
            game,
            bridges,
        })
    }

    // 获取游戏描述符引用
    pub fn game(&self) -> &GameDescriptor {
        &self.game
    }

    // 主循环：处理尺寸约束→收集事件→分发事件→固定步长 Tick→渲染→处理命令→循环直到退出
    pub fn run(mut self) -> Result<()> {
        let _log_object_guard = host_log::scoped_log_object(self.game.id.clone());
        renderer::invalidate_canvas_cache();
        let frame_duration = frame_duration_for_fps(self.game.target_fps);
        let mut last_tick_at = Instant::now();
        let mut tick_accumulator = Duration::ZERO;

        loop {
            let constraints = size_watcher::SizeConstraints {
                min_width: self.game.min_width,
                min_height: self.game.min_height,
                max_width: self.game.max_width,
                max_height: self.game.max_height,
            };
            let size_state = size_watcher::check_constraints(constraints)?;
            if !size_state.size_ok {
                renderer::invalidate_canvas_cache();
                size_watcher::draw_size_warning_with_constraints(&size_state, constraints, true)?;
                if event::poll(frame_duration)? {
                    match event::read()? {
                        Event::Key(key) if matches!(key.kind, KeyEventKind::Press) => {
                            for key_name in semantic_key_source().record_crossterm_key(key) {
                                let input_event = map_semantic_key_to_event(&self.game, key_name);
                                self.handle_event(&input_event)?;
                                if self.process_runtime_commands_for_pending_frame(None)? {
                                    self.exit_runtime()?;
                                    renderer::invalidate_canvas_cache();
                                    return Ok(());
                                }
                            }
                        }
                        Event::Resize(width, height) => {
                            self.with_canvas(|canvas| canvas.resize(width, height));
                        }
                        _ => {}
                    }
                }
                continue;
            }

            let frame_deadline = Instant::now() + frame_duration;
            let mut frame_events = Vec::new();
            while frame_events.len() < MAX_EVENTS_PER_FRAME {
                let now = Instant::now();
                if now >= frame_deadline {
                    break;
                }
                let remaining = frame_deadline.saturating_duration_since(now);
                if !event::poll(remaining)? {
                    break;
                }
                match event::read()? {
                    Event::Key(key) if matches!(key.kind, KeyEventKind::Press) => {
                        for key_name in semantic_key_source().record_crossterm_key(key) {
                            if frame_events.len() >= MAX_EVENTS_PER_FRAME {
                                break;
                            }
                            frame_events.push(map_semantic_key_to_event(&self.game, key_name));
                        }
                    }
                    Event::Resize(width, height) => {
                        frame_events.push(InputEvent::Resize { width, height });
                    }
                    _ => {}
                }
            }

            if frame_events.len() < MAX_EVENTS_PER_FRAME {
                for key_name in semantic_key_source()
                    .drain_ready_rdev_keys(MAX_EVENTS_PER_FRAME - frame_events.len())
                {
                    frame_events.push(map_semantic_key_to_event(&self.game, key_name));
                }
            }

            let mut frame_events = std::collections::VecDeque::from(frame_events);
            while let Some(input_event) = frame_events.pop_front() {
                if matches!(input_event, InputEvent::Resize { .. })
                    && let Ok(mut resize_flag) = self.bridges.resize_flag.lock()
                {
                    *resize_flag = true;
                }
                self.handle_event(&input_event)?;
                if self.process_runtime_commands_for_pending_frame(Some(&mut frame_events))? {
                    self.exit_runtime()?;
                    renderer::invalidate_canvas_cache();
                    return Ok(());
                }
            }

            // Deliver fixed-step ticks. When rendering or event handling takes longer
            // than one frame, catch up logical time before rendering once.
            let now = Instant::now();
            tick_accumulator = tick_accumulator.saturating_add(now.saturating_duration_since(last_tick_at));
            last_tick_at = now;

            let fixed_dt_ms = frame_duration
                .as_millis()
                .clamp(1, u128::from(u32::MAX)) as u32;
            let mut catch_up_ticks = 0;
            while tick_accumulator >= frame_duration
                && catch_up_ticks < MAX_CATCH_UP_TICKS_PER_FRAME
            {
                tick_accumulator = tick_accumulator.saturating_sub(frame_duration);
                self.handle_event(&InputEvent::Tick { dt_ms: fixed_dt_ms })?;
                if self.process_runtime_commands_for_pending_frame(None)? {
                    self.exit_runtime()?;
                    renderer::invalidate_canvas_cache();
                    return Ok(());
                }
                catch_up_ticks += 1;
            }
            if catch_up_ticks == MAX_CATCH_UP_TICKS_PER_FRAME && tick_accumulator >= frame_duration {
                tick_accumulator = Duration::ZERO;
            }

            self.render_current_frame()?;

            if self.process_runtime_commands_after_frame()? {
                break;
            }
        }

        self.exit_runtime()?;
        renderer::invalidate_canvas_cache();
        Ok(())
    }

    // 调用 Lua 回调 handle_event，更新 state_key
    fn handle_event(&mut self, event: &InputEvent) -> Result<()> {
        self.state_key = api::callback_api::call_handle_event(&self.lua, &self.state_key, event)?;
        Ok(())
    }

    // 调用 Lua 回调 render
    fn render(&mut self) -> Result<()> {
        api::callback_api::call_render(&self.lua, &self.state_key)
    }

    // 调用 Lua 回调 exit_game，然后保存最佳成绩
    fn exit_runtime(&mut self) -> Result<()> {
        self.state_key = api::callback_api::call_exit_game(&self.lua, &self.state_key)?;
        api::callback_api::persist_best_score(&self.lua, &self.state_key, &self.game)?;
        Ok(())
    }

    // 安全地获取画布锁并执行操作
    fn with_canvas(&self, f: impl FnOnce(&mut Canvas)) {
        if let Ok(mut canvas) = self.bridges.canvas.lock() {
            f(&mut canvas);
        }
    }

    // 处理帧结束后剩余的命令，返回是否应该退出
    fn process_runtime_commands_after_frame(&mut self) -> Result<bool> {
        let mut should_exit = false;
        for command in self.drain_commands()? {
            match command {
                RuntimeCommand::ExitGame => should_exit = true,
                RuntimeCommand::SaveBestScore if self.game.has_best_score => {
                    api::callback_api::persist_best_score(&self.lua, &self.state_key, &self.game)?;
                }
                RuntimeCommand::SaveGame if self.game.save => {
                    api::callback_api::persist_save_game(&self.lua, &self.state_key, &self.game)?;
                }
                RuntimeCommand::SaveBestScore
                | RuntimeCommand::SaveGame
                | RuntimeCommand::SkipEventQueue
                | RuntimeCommand::ClearEventQueue
                | RuntimeCommand::RenderNow
                | RuntimeCommand::ShowToast { .. } => {}
            }
        }
        Ok(should_exit)
    }

    // 处理帧结束后剩余的命令，返回是否应该退出
    fn process_runtime_commands_for_pending_frame(
        &mut self,
        mut pending_events: Option<&mut std::collections::VecDeque<InputEvent>>,
    ) -> Result<bool> {
        for command in self.drain_commands()? {
            match command {
                RuntimeCommand::ExitGame => return Ok(true),
                RuntimeCommand::SkipEventQueue => {
                    if let Some(pending_events) = pending_events.as_deref_mut() {
                        pending_events.clear();
                    }
                }
                RuntimeCommand::ClearEventQueue => {
                    if let Some(pending_events) = pending_events.as_deref_mut() {
                        pending_events.clear();
                    }
                    api::direct_system_request_api::clear_pending_input_queue();
                }
                RuntimeCommand::RenderNow => {
                    self.render_current_frame()?;
                }
                RuntimeCommand::SaveBestScore if self.game.has_best_score => {
                    api::callback_api::persist_best_score(&self.lua, &self.state_key, &self.game)?;
                }
                RuntimeCommand::SaveGame if self.game.save => {
                    api::callback_api::persist_save_game(&self.lua, &self.state_key, &self.game)?;
                }
                RuntimeCommand::SaveBestScore
                | RuntimeCommand::SaveGame
                | RuntimeCommand::ShowToast { .. } => {}
            }
        }
        Ok(false)
    }

    // 取出命令队列中的所有命令
    fn drain_commands(&self) -> Result<Vec<RuntimeCommand>> {
        let mut commands = self
            .bridges
            .commands
            .lock()
            .map_err(|_| anyhow!("command queue poisoned"))?;
        Ok(std::mem::take(&mut *commands))
    }

    // 清除画布，调用 Lua render，将画布渲染到终端
    fn render_current_frame(&mut self) -> Result<()> {
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        self.with_canvas(|canvas| {
            if canvas.width() != width || canvas.height() != height {
                canvas.resize(width, height);
            }
            canvas.clear();
        });
        self.render()?;
        let canvas = self
            .bridges
            .canvas
            .lock()
            .map_err(|_| anyhow!("canvas poisoned"))?;
        renderer::render_canvas(&canvas)?;
        Ok(())
    }
}

// 计算每帧时长，非法值回退到 60 FPS
fn frame_duration_for_fps(target_fps: u16) -> Duration {
    let fps = match target_fps {
        30 => 30,
        60 => 60,
        120 => 120,
        _ => DEFAULT_TARGET_FPS,
    };
    Duration::from_secs_f64(1.0 / f64::from(fps))
}

// 清空输入缓冲区，创建并运行引擎
pub fn run_game_descriptor(game: &GameDescriptor, mode: LaunchMode) -> Result<()> {
    clear_startup_input_buffer();
    LuaGameEngine::new(game.clone(), mode)?.run()
}

// 清空所有未处理的终端事件和键盘状态
fn clear_startup_input_buffer() {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
    semantic_key_source().clear_pending_keys();
}

// 根据动作绑定将语义键名映射为 InputEvent::Action，否则为 InputEvent::Key
fn map_semantic_key_to_event(game: &GameDescriptor, key_name: String) -> InputEvent {
    let event_key_name = if game.case_sensitive {
        key_name
    } else {
        key_name.to_lowercase()
    };
    for (action, binding) in &game.actions {
        if binding
            .keys()
            .into_iter()
            .any(|candidate| {
                if game.case_sensitive {
                    candidate == event_key_name
                } else {
                    candidate.eq_ignore_ascii_case(&event_key_name)
                }
            })
        {
            return InputEvent::Action(action.clone());
        }
    }
    InputEvent::Key(event_key_name)
}

// 将 Lua 错误转换为 anyhow 错误
pub(crate) fn anyhow_lua_error(err: mlua::Error) -> anyhow::Error {
    anyhow!(err.to_string())
}
