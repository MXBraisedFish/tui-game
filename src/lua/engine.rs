use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use mlua::{Lua, RegistryKey, Table, Value};
use serde_json::{Map, Value as JsonValue};

use crate::app::i18n;
use crate::core::command::RuntimeCommand;
use crate::core::event::InputEvent;
use crate::core::screen::Canvas;
use crate::core::save;
use crate::core::stats;
use crate::game::registry::GameDescriptor;
use crate::lua::sandbox;
use crate::lua_bridge::api::LaunchMode;
use crate::mods;
use crate::terminal::{renderer, size_watcher};

struct RuntimeBridges {
    canvas: Arc<Mutex<Canvas>>,
    commands: Arc<Mutex<Vec<RuntimeCommand>>>,
    resize_flag: Arc<Mutex<bool>>,
    save_path: PathBuf,
    game: GameDescriptor,
    launch_mode: LaunchMode,
}

/// 新运行时下的 Lua 引擎包装。
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
        let save_path = runtime_save_path(&game)?;
        install_runtime_apis(
            &lua,
            RuntimeBridges {
                canvas: Arc::clone(&canvas),
                commands: Arc::clone(&commands),
                resize_flag: Arc::clone(&resize_flag),
                save_path: save_path.clone(),
                game: game.clone(),
                launch_mode,
            },
        )
        .map_err(anyhow_lua_error)?;

        let source = fs::read_to_string(&game.entry_path)
            .with_context(|| format!("failed to read runtime script: {}", game.entry_path.display()))?;
        lua.load(source.trim_start_matches('\u{feff}'))
            .set_name(game.entry_path.to_string_lossy().as_ref())
            .exec()
            .map_err(|err| anyhow!("failed to execute runtime script {}: {}", game.entry_path.display(), err))?;

        let init_game: mlua::Function = lua
            .globals()
            .get("init_game")
            .map_err(|err| anyhow!("runtime script missing init_game(): {}", err))?;
        let initial_state = init_game.call::<Value>(()).map_err(anyhow_lua_error)?;
        let state_key = lua.create_registry_value(initial_state).map_err(anyhow_lua_error)?;

        Ok(Self {
            lua,
            state_key,
            game: game.clone(),
            bridges: RuntimeBridges {
                canvas,
                commands,
                resize_flag,
                save_path,
                game,
                launch_mode,
            },
        })
    }

    pub fn game(&self) -> &GameDescriptor {
        &self.game
    }

    pub fn run(mut self) -> Result<()> {
        renderer::invalidate_canvas_cache();
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
                if event::poll(Duration::from_millis(16))? {
                    match event::read()? {
                        Event::Key(key)
                            if matches!(key.kind, KeyEventKind::Press)
                                && matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')) =>
                        {
                            break;
                        }
                        Event::Resize(width, height) => {
                            self.with_canvas(|canvas| canvas.resize(width, height));
                        }
                        _ => {}
                    }
                }
                continue;
            }

            let input_event = if event::poll(Duration::from_millis(16))? {
                Some(match event::read()? {
                    Event::Key(key) if matches!(key.kind, KeyEventKind::Press) => {
                        map_key_to_event(&self.game, key)
                    }
                    Event::Resize(width, height) => Some(InputEvent::Resize { width, height }),
                    _ => None,
                })
                .flatten()
            } else {
                Some(InputEvent::Tick { dt_ms: 16 })
            };

            if let Some(event) = input_event {
                if matches!(event, InputEvent::Resize { .. }) {
                    if let Ok(mut resize_flag) = self.bridges.resize_flag.lock() {
                        *resize_flag = true;
                    }
                }
                self.handle_event(&event)?;
            }

            let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
            self.with_canvas(|canvas| {
                if canvas.width() != width || canvas.height() != height {
                    canvas.resize(width, height);
                }
                canvas.clear();
            });
            self.render()?;
            {
                let canvas = self.bridges.canvas.lock().map_err(|_| anyhow!("canvas poisoned"))?;
                renderer::render_canvas(&canvas)?;
            }

            let commands = self.drain_commands()?;
            let mut should_exit = false;
            for command in commands {
                match command {
                    RuntimeCommand::ExitGame => should_exit = true,
                    RuntimeCommand::RefreshBestScore => self.persist_best_score()?,
                    RuntimeCommand::SaveRequest => {}
                    RuntimeCommand::ClearSave => {
                        let _ = fs::remove_file(&self.bridges.save_path);
                    }
                    RuntimeCommand::ShowToast { .. } => {}
                }
            }

            if should_exit {
                break;
            }
        }

        self.persist_best_score()?;
        renderer::invalidate_canvas_cache();
        Ok(())
    }

    fn handle_event(&mut self, event: &InputEvent) -> Result<()> {
        let handle_event: mlua::Function = self
            .lua
            .globals()
            .get("handle_event")
            .map_err(|err| anyhow!("runtime script missing handle_event(state, event): {}", err))?;
        let state = self.lua.registry_value::<Value>(&self.state_key).map_err(anyhow_lua_error)?;
        let event_table = to_lua_event_table(&self.lua, event).map_err(anyhow_lua_error)?;
        let new_state = handle_event
            .call::<Value>((state, event_table))
            .map_err(anyhow_lua_error)?;
        self.state_key = self.lua.create_registry_value(new_state).map_err(anyhow_lua_error)?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let render: mlua::Function = self
            .lua
            .globals()
            .get("render")
            .map_err(|err| anyhow!("runtime script missing render(state): {}", err))?;
        let state = self.lua.registry_value::<Value>(&self.state_key).map_err(anyhow_lua_error)?;
        render.call::<()>(state).map_err(anyhow_lua_error)?;
        Ok(())
    }

    fn persist_best_score(&mut self) -> Result<()> {
        let best_score: mlua::Function = match self.lua.globals().get("best_score") {
            Ok(func) => func,
            Err(_) => return Ok(()),
        };
        let state = self.lua.registry_value::<Value>(&self.state_key).map_err(anyhow_lua_error)?;
        let value = best_score.call::<Value>(state).map_err(anyhow_lua_error)?;
        if matches!(value, Value::Nil) {
            return Ok(());
        }
        let json = lua_value_to_json(&value)?;
        stats::write_runtime_best_score(&self.game.id, &json)?;
        Ok(())
    }

    fn with_canvas(&self, f: impl FnOnce(&mut Canvas)) {
        if let Ok(mut canvas) = self.bridges.canvas.lock() {
            f(&mut canvas);
        }
    }

    fn drain_commands(&self) -> Result<Vec<RuntimeCommand>> {
        let mut commands = self
            .bridges
            .commands
            .lock()
            .map_err(|_| anyhow!("command queue poisoned"))?;
        Ok(std::mem::take(&mut *commands))
    }
}

pub fn run_game_descriptor(game: &GameDescriptor, mode: LaunchMode) -> Result<()> {
    LuaGameEngine::new(game.clone(), mode)?.run()
}

fn install_runtime_apis(lua: &Lua, bridges: RuntimeBridges) -> mlua::Result<()> {
    let globals = lua.globals();

    let canvas_ref = Arc::clone(&bridges.canvas);
    globals.set(
        "canvas_clear",
        lua.create_function(move |_, ()| {
            if let Ok(mut canvas) = canvas_ref.lock() {
                canvas.clear();
            }
            Ok(())
        })?,
    )?;

    let canvas_ref = Arc::clone(&bridges.canvas);
    globals.set(
        "canvas_draw_text",
        lua.create_function(move |_, (x, y, text, fg, bg): (u16, u16, String, Option<String>, Option<String>)| {
            if let Ok(mut canvas) = canvas_ref.lock() {
                canvas.draw_text(x, y, &text, fg, bg);
            }
            Ok(())
        })?,
    )?;

    let canvas_ref = Arc::clone(&bridges.canvas);
    globals.set(
        "canvas_fill_rect",
        lua.create_function(
            move |_, (x, y, width, height, ch, fg, bg): (u16, u16, u16, u16, String, Option<String>, Option<String>)| {
                if let Ok(mut canvas) = canvas_ref.lock() {
                    let fill = ch.chars().next().unwrap_or(' ');
                    canvas.fill_rect(
                        x,
                        y,
                        width,
                        height,
                        crate::core::screen::Cell {
                            ch: fill,
                            fg,
                            bg,
                        },
                    );
                }
                Ok(())
            },
        )?,
    )?;

    globals.set(
        "measure_text",
        lua.create_function(|_, text: String| Ok(Canvas::measure_text(&text)))?,
    )?;
    globals.set(
        "get_text_width",
        lua.create_function(|_, text: String| Ok(Canvas::measure_text(&text).0))?,
    )?;
    globals.set(
        "get_text_size",
        lua.create_function(|_, text: String| Ok(Canvas::measure_text(&text)))?,
    )?;
    globals.set(
        "get_terminal_size",
        lua.create_function(|_, ()| {
            let (w, h) = crossterm::terminal::size().unwrap_or((80, 24));
            Ok((w, h))
        })?,
    )?;
    globals.set("ANCHOR_LEFT", 0)?;
    globals.set("ANCHOR_CENTER", 1)?;
    globals.set("ANCHOR_RIGHT", 2)?;
    globals.set("ANCHOR_TOP", 0)?;
    globals.set("ANCHOR_MIDDLE", 1)?;
    globals.set("ANCHOR_BOTTOM", 2)?;
    globals.set(
        "resolve_x",
        lua.create_function(|_, (anchor, content_width, offset): (i64, u16, Option<i64>)| {
            let (term_w, _) = crossterm::terminal::size().unwrap_or((80, 24));
            Ok(resolve_axis_position(anchor, term_w, content_width, offset.unwrap_or(0)))
        })?,
    )?;
    globals.set(
        "resolve_y",
        lua.create_function(|_, (anchor, content_height, offset): (i64, u16, Option<i64>)| {
            let (_, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
            Ok(resolve_axis_position(anchor, term_h, content_height, offset.unwrap_or(0)))
        })?,
    )?;
    globals.set(
        "resolve_rect",
        lua.create_function(
            |_,
             (h_anchor, v_anchor, width, height, offset_x, offset_y): (
                i64,
                i64,
                u16,
                u16,
                Option<i64>,
                Option<i64>,
            )| {
                let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
                let x = resolve_axis_position(h_anchor, term_w, width, offset_x.unwrap_or(0));
                let y = resolve_axis_position(v_anchor, term_h, height, offset_y.unwrap_or(0));
                Ok((x, y))
            },
        )?,
    )?;

    let resize_ref = Arc::clone(&bridges.resize_flag);
    globals.set(
        "was_terminal_resized",
        lua.create_function(move |_, ()| {
            let flag = resize_ref.lock().map(|value| *value).unwrap_or(false);
            Ok(flag)
        })?,
    )?;
    let resize_ref = Arc::clone(&bridges.resize_flag);
    globals.set(
        "consume_resize_event",
        lua.create_function(move |_, ()| {
            let mut flag = resize_ref
                .lock()
                .map_err(|_| mlua::Error::RuntimeError("resize flag poisoned".to_string()))?;
            let value = *flag;
            *flag = false;
            Ok(value)
        })?,
    )?;

    let commands_ref = Arc::clone(&bridges.commands);
    globals.set(
        "request_exit",
        lua.create_function(move |_, ()| {
            if let Ok(mut commands) = commands_ref.lock() {
                commands.push(RuntimeCommand::ExitGame);
            }
            Ok(())
        })?,
    )?;
    let commands_ref = Arc::clone(&bridges.commands);
    globals.set(
        "request_refresh_best_score",
        lua.create_function(move |_, ()| {
            if let Ok(mut commands) = commands_ref.lock() {
                commands.push(RuntimeCommand::RefreshBestScore);
            }
            Ok(())
        })?,
    )?;

    let save_path = bridges.save_path.clone();
    let latest_game_id = bridges.game.id.clone();
    globals.set(
        "save_data",
        lua.create_function(move |_, (slot, value): (String, Value)| {
            let json = lua_value_to_json(&value).map_err(lua_runtime_error)?;
            save_runtime_slot(&save_path, &slot, &json).map_err(lua_runtime_error)?;
            let _ = save::set_latest_runtime_save_game(&latest_game_id);
            Ok(())
        })?,
    )?;

    let save_path = bridges.save_path.clone();
    globals.set(
        "load_data",
        lua.create_function(move |lua, slot: String| {
            let Some(value) = load_runtime_slot(&save_path, &slot).map_err(lua_runtime_error)? else {
                return Ok(Value::Nil);
            };
            json_to_lua_value(lua, &value)
        })?,
    )?;

    let namespace = bridges
        .game
        .package_info()
        .map(|package| package.namespace.clone())
        .or_else(|| bridges.game.mod_info.as_ref().map(|info| info.namespace.clone()));
    globals.set(
        "translate",
        lua.create_function(move |_, key: String| {
            if let Some(namespace) = &namespace {
                Ok(mods::resolve_mod_text_for_display(namespace, &key))
            } else {
                Ok(i18n::t_or(&key, &key))
            }
        })?,
    )?;

    let launch_mode = bridges.launch_mode;
    globals.set(
        "get_launch_mode",
        lua.create_function(move |_, ()| Ok(match launch_mode {
            LaunchMode::Continue => "continue".to_string(),
            LaunchMode::New => "new".to_string(),
        }))?,
    )?;

    Ok(())
}

fn to_lua_event_table(lua: &Lua, event: &InputEvent) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    match event {
        InputEvent::Action(name) => {
            table.set("type", "action")?;
            table.set("name", name.as_str())?;
        }
        InputEvent::Resize { width, height } => {
            table.set("type", "resize")?;
            table.set("width", *width)?;
            table.set("height", *height)?;
        }
        InputEvent::Tick { dt_ms } => {
            table.set("type", "tick")?;
            table.set("dt_ms", *dt_ms)?;
        }
        InputEvent::Quit => {
            table.set("type", "quit")?;
        }
    }
    Ok(table)
}

fn map_key_to_event(game: &GameDescriptor, key: KeyEvent) -> Option<InputEvent> {
    let key_name = normalize_key_name(key.code)?;
    for (action, binding) in &game.actions {
        if binding
            .keys()
            .into_iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&key_name))
        {
            return Some(InputEvent::Action(action.clone()));
        }
    }
    if matches!(key.code, KeyCode::Esc) {
        return Some(InputEvent::Quit);
    }
    None
}

fn normalize_key_name(code: KeyCode) -> Option<String> {
    Some(match code {
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Char(' ') => "space".to_string(),
        KeyCode::Char(ch) => ch.to_ascii_lowercase().to_string(),
        _ => return None,
    })
}

fn resolve_axis_position(anchor: i64, terminal_span: u16, content_span: u16, offset: i64) -> u16 {
    let base = match anchor {
        0 => 0i64,
        1 => i64::from(terminal_span.saturating_sub(content_span)) / 2,
        2 => i64::from(terminal_span.saturating_sub(content_span)),
        _ => 0,
    };
    let resolved = (base + offset).max(0);
    resolved.min(i64::from(u16::MAX)) as u16
}

fn runtime_save_path(game: &GameDescriptor) -> Result<PathBuf> {
    save::runtime_game_save_path(&game.id)
}

fn save_runtime_slot(path: &PathBuf, slot: &str, value: &JsonValue) -> Result<()> {
    let mut store = if path.exists() {
        let raw = fs::read_to_string(path)?;
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}')).unwrap_or_default()
    } else {
        Map::new()
    };
    store.insert(slot.to_string(), value.clone());
    fs::write(path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}

fn load_runtime_slot(path: &PathBuf, slot: &str) -> Result<Option<JsonValue>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let store = serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}')).unwrap_or_default();
    Ok(store.get(slot).cloned())
}

fn lua_value_to_json(value: &Value) -> Result<JsonValue> {
    Ok(match value {
        Value::Nil => JsonValue::Null,
        Value::Boolean(v) => JsonValue::Bool(*v),
        Value::Integer(v) => JsonValue::Number((*v).into()),
        Value::Number(v) => serde_json::Number::from_f64(*v)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(v) => JsonValue::String(v.to_str().map_err(anyhow_lua_error)?.to_string()),
        Value::Table(table) => {
            let mut map = Map::new();
            let mut array = Vec::new();
            let mut is_array = true;
            for pair in table.clone().pairs::<Value, Value>() {
                let (key, value) = pair.map_err(anyhow_lua_error)?;
                match key {
                    Value::Integer(index) if index > 0 => {
                        array.push((index as usize, lua_value_to_json(&value)?));
                    }
                    Value::String(key) => {
                        is_array = false;
                        map.insert(
                            key.to_str().map_err(anyhow_lua_error)?.to_string(),
                            lua_value_to_json(&value)?,
                        );
                    }
                    _ => {
                        is_array = false;
                    }
                }
            }
            if is_array && !array.is_empty() {
                array.sort_by_key(|(index, _)| *index);
                JsonValue::Array(array.into_iter().map(|(_, value)| value).collect())
            } else {
                JsonValue::Object(map)
            }
        }
        other => return Err(anyhow!("unsupported lua value for json conversion: {other:?}")),
    })
}

fn anyhow_lua_error(err: mlua::Error) -> anyhow::Error {
    anyhow!(err.to_string())
}

fn lua_runtime_error(err: anyhow::Error) -> mlua::Error {
    mlua::Error::RuntimeError(err.to_string())
}

fn json_to_lua_value(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    Ok(match value {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(v) => Value::Boolean(*v),
        JsonValue::Number(v) => {
            if let Some(integer) = v.as_i64() {
                Value::Integer(integer)
            } else {
                Value::Number(v.as_f64().unwrap_or_default())
            }
        }
        JsonValue::String(v) => Value::String(lua.create_string(v)?),
        JsonValue::Array(values) => {
            let table = lua.create_table()?;
            for (index, value) in values.iter().enumerate() {
                table.set((index + 1) as i64, json_to_lua_value(lua, value)?)?;
            }
            Value::Table(table)
        }
        JsonValue::Object(values) => {
            let table = lua.create_table()?;
            for (key, value) in values {
                table.set(key.as_str(), json_to_lua_value(lua, value)?)?;
            }
            Value::Table(table)
        }
    })
}
