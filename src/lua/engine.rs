use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use crossterm::event::{self, Event, KeyEventKind};
use mlua::{Lua, RegistryKey};

use crate::app::i18n;
use crate::core::command::RuntimeCommand;
use crate::core::event::InputEvent;
use crate::core::key::semantic_key_source;
use crate::core::runtime::LaunchMode;
use crate::core::screen::Canvas;
use crate::game::registry::GameDescriptor;
use crate::lua::{api, sandbox};
use crate::terminal::{renderer, size_watcher};

const DEFAULT_TARGET_FPS: u16 = 60;
const MAX_EVENTS_PER_FRAME: usize = 256;

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct RuntimeBridges {
    pub(crate) canvas: Arc<Mutex<Canvas>>,
    pub(crate) commands: Arc<Mutex<Vec<RuntimeCommand>>>,
    pub(crate) resize_flag: Arc<Mutex<bool>>,
    pub(crate) game: GameDescriptor,
    pub(crate) launch_mode: LaunchMode,
    pub(crate) started_at: Instant,
}

pub struct LuaGameEngine {
    lua: Lua,
    state_key: RegistryKey,
    game: GameDescriptor,
    bridges: RuntimeBridges,
}

impl LuaGameEngine {
    pub fn new(game: GameDescriptor, launch_mode: LaunchMode) -> Result<Self> {
        let lua = Lua::new();
        sandbox::install_sandbox(&lua).map_err(anyhow_lua_error)?;

        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let canvas = Arc::new(Mutex::new(Canvas::new(width, height)));
        let commands = Arc::new(Mutex::new(Vec::new()));
        let resize_flag = Arc::new(Mutex::new(false));
        let bridges = RuntimeBridges {
            canvas: Arc::clone(&canvas),
            commands: Arc::clone(&commands),
            resize_flag: Arc::clone(&resize_flag),
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

    pub fn game(&self) -> &GameDescriptor {
        &self.game
    }

    pub fn run(mut self) -> Result<()> {
        renderer::invalidate_canvas_cache();
        let frame_duration = frame_duration_for_fps(self.game.target_fps);
        let mut last_tick_at = Instant::now();

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

            // A frame-ending tick is always delivered exactly once.
            // Queue control commands such as skip/clear only affect the remaining
            // non-tick events in the current frame and pending host input buffers.
            let dt_ms = last_tick_at
                .elapsed()
                .as_millis()
                .clamp(1, u128::from(u32::MAX)) as u32;
            last_tick_at = Instant::now();
            self.handle_event(&InputEvent::Tick { dt_ms })?;
            if self.process_runtime_commands_for_pending_frame(None)? {
                self.exit_runtime()?;
                renderer::invalidate_canvas_cache();
                return Ok(());
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

    fn handle_event(&mut self, event: &InputEvent) -> Result<()> {
        self.state_key = api::callback_api::call_handle_event(&self.lua, &self.state_key, event)?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        api::callback_api::call_render(&self.lua, &self.state_key)
    }

    fn exit_runtime(&mut self) -> Result<()> {
        self.state_key = api::callback_api::call_exit_game(&self.lua, &self.state_key)?;
        api::callback_api::persist_best_score(&self.lua, &self.state_key, &self.game)?;
        Ok(())
    }

    fn with_canvas(&self, f: impl FnOnce(&mut Canvas)) {
        if let Ok(mut canvas) = self.bridges.canvas.lock() {
            f(&mut canvas);
        }
    }

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
                    api::direct_system_control_api::clear_pending_input_queue();
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

    fn drain_commands(&self) -> Result<Vec<RuntimeCommand>> {
        let mut commands = self
            .bridges
            .commands
            .lock()
            .map_err(|_| anyhow!("command queue poisoned"))?;
        Ok(std::mem::take(&mut *commands))
    }

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

fn frame_duration_for_fps(target_fps: u16) -> Duration {
    let fps = match target_fps {
        30 => 30,
        60 => 60,
        120 => 120,
        _ => DEFAULT_TARGET_FPS,
    };
    Duration::from_secs_f64(1.0 / f64::from(fps))
}

pub fn run_game_descriptor(game: &GameDescriptor, mode: LaunchMode) -> Result<()> {
    LuaGameEngine::new(game.clone(), mode)?.run()
}

fn map_semantic_key_to_event(game: &GameDescriptor, key_name: String) -> InputEvent {
    for (action, binding) in &game.actions {
        if binding
            .keys()
            .into_iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&key_name))
        {
            return InputEvent::Action(action.clone());
        }
    }
    InputEvent::Key(key_name)
}

pub(crate) fn anyhow_lua_error(err: mlua::Error) -> anyhow::Error {
    anyhow!(err.to_string())
}
