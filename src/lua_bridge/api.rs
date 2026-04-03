use std::collections::BTreeMap;
use std::fs;
use std::io::{Stdout, Write, stdout};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::queue;
use crossterm::style::{
    Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use mlua::{Function, Lua, Table, Value};
use once_cell::sync::Lazy;
use serde_json::{Map, Number, Value as JsonValue};
use unicode_width::UnicodeWidthStr;

use crate::app::{i18n, stats};
use crate::utils::path_utils;

const EXIT_GAME_SENTINEL: &str = "__TUI_GAME_EXIT__"; // 娓告垙閫€鍑烘爣璁?
static OUT: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(stdout())); // 缁堢杈撳嚭鐨勫叏灞€閿?
static TERMINAL_DIRTY_FROM_LUA: AtomicBool = AtomicBool::new(false); // Lua 鏄惁淇敼浜嗙粓绔?
static RNG_STATE: AtomicU64 = AtomicU64::new(0); // 随机数状态
const ANCHOR_LEFT: i64 = 0;
const ANCHOR_CENTER: i64 = 1;
const ANCHOR_RIGHT: i64 = 2;
const ANCHOR_TOP: i64 = 0;
const ANCHOR_MIDDLE: i64 = 1;
const ANCHOR_BOTTOM: i64 = 2;

#[derive(Clone, Copy)]
enum AxisOrientation {
    Horizontal,
    Vertical,
}

fn resolve_axis_position(
    anchor: i64,
    terminal_extent: i64,
    content_extent: i64,
    offset: i64,
    orientation: AxisOrientation,
) -> i64 {
    let base = match orientation {
        AxisOrientation::Horizontal => match anchor {
            ANCHOR_CENTER => ((terminal_extent - content_extent).max(0) / 2) + 1,
            ANCHOR_RIGHT => (terminal_extent - content_extent).max(0) + 1,
            _ => 1,
        },
        AxisOrientation::Vertical => match anchor {
            ANCHOR_MIDDLE => ((terminal_extent - content_extent).max(0) / 2) + 1,
            ANCHOR_BOTTOM => (terminal_extent - content_extent).max(0) + 1,
            _ => 1,
        },
    };
    base + offset
}

// 鍚姩娓告垙妯″紡鐨勬灇涓?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchMode {
    New,
    Continue,
}

impl LaunchMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Continue => "continue",
        }
    }
}

// 灏咥PI娉ㄥ唽锛岃Lua鍙皟鐢?
pub fn register_api(lua: &Lua, mode: LaunchMode) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.set("ANCHOR_LEFT", ANCHOR_LEFT)?;
    globals.set("ANCHOR_CENTER", ANCHOR_CENTER)?;
    globals.set("ANCHOR_RIGHT", ANCHOR_RIGHT)?;
    globals.set("ANCHOR_TOP", ANCHOR_TOP)?;
    globals.set("ANCHOR_MIDDLE", ANCHOR_MIDDLE)?;
    globals.set("ANCHOR_BOTTOM", ANCHOR_BOTTOM)?;

    let get_key = lua.create_function(|_, blocking: bool| {
        touch_mod_watchdog();
        flush_output()?;

        if blocking {
            loop {
                if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                    if key.kind == KeyEventKind::Press {
                        return decode_key_event(key);
                    }
                }
            }
        }

        if event::poll(Duration::from_millis(0)).map_err(mlua::Error::external)? {
            if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                if key.kind == KeyEventKind::Press {
                    return decode_key_event(key);
                }
            }
        }
        Ok(String::new())
    })?;
    globals.set("get_key", get_key)?;

    let get_raw_key = lua.create_function(|_, blocking: bool| {
        touch_mod_watchdog();
        flush_output()?;

        if blocking {
            loop {
                if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                    if key.kind == KeyEventKind::Press {
                        return decode_key_event(key);
                    }
                }
            }
        }

        if event::poll(Duration::from_millis(0)).map_err(mlua::Error::external)? {
            if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                if key.kind == KeyEventKind::Press {
                    return decode_key_event(key);
                }
            }
        }
        Ok(String::new())
    })?;
    globals.set("get_raw_key", get_raw_key)?;

    let clear = lua.create_function(|_, ()| {
        touch_mod_watchdog();
        let mut out = lock_out()?;
        queue!(
            out,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )
        .map_err(mlua::Error::external)?;
        Ok(())
    })?;
    globals.set("clear", clear)?;

    let draw_text = lua.create_function(
        |lua, (x, y, text, fg, bg): (i64, i64, String, Option<String>, Option<String>)| {
            touch_mod_watchdog();
            draw_text_rich_impl(lua, x, y, &text, fg.as_deref(), bg.as_deref())
        },
    )?;
    globals.set("draw_text", draw_text)?;

    let draw_text_ex = lua.create_function(
        |lua,
         (x, y, text, fg, bg, max_width, align): (
            i64,
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
        )| {
            touch_mod_watchdog();
            let width = max_width.unwrap_or(text.len() as i64).max(0) as usize;
            let mut rendered = text.clone();
            if width > 0 {
                let w = UnicodeWidthStr::width(text.as_str());
                if w < width {
                    let pad = width - w;
                    match align.unwrap_or_else(|| "left".to_string()).as_str() {
                        "center" => {
                            let left = pad / 2;
                            let right = pad - left;
                            rendered = format!("{}{}{}", " ".repeat(left), text, " ".repeat(right));
                        }
                        "right" => rendered = format!("{}{}", " ".repeat(pad), text),
                        _ => {}
                    }
                }
            }
            draw_text_rich_impl(lua, x, y, &rendered, fg.as_deref(), bg.as_deref())
        },
    )?;
    globals.set("draw_text_ex", draw_text_ex)?;

    let sleep = lua.create_function(|_, ms: i64| {
        touch_mod_watchdog();
        flush_output()?;
        let ms = ms.max(0) as u64;
        std::thread::sleep(Duration::from_millis(ms));
        if ms >= 200 {
            drain_input_events();
        }
        Ok(())
    })?;
    globals.set("sleep", sleep)?;

    let clear_input_buffer = lua.create_function(|_, ()| {
        touch_mod_watchdog();
        drain_input_events();
        Ok(true)
    })?;
    globals.set("clear_input_buffer", clear_input_buffer)?;

    let random = lua.create_function(|_, max: i64| {
        touch_mod_watchdog();
        if max <= 0 {
            return Ok(0);
        }
        Ok((next_random_u64() % (max as u64)) as i64)
    })?;
    globals.set("random", random)?;

    let exit_game = lua.create_function(|_, ()| -> mlua::Result<()> {
        touch_mod_watchdog();
        Err(mlua::Error::RuntimeError(EXIT_GAME_SENTINEL.to_string()))
    })?;
    globals.set("exit_game", exit_game)?;

    let translate = lua.create_function(|_, key: String| Ok(i18n::t(&key)))?;
    globals.set("translate", translate)?;

    let get_terminal_size = lua.create_function(|_, ()| {
        touch_mod_watchdog();
        let (w, h) = crossterm::terminal::size().map_err(mlua::Error::external)?;
        Ok((w, h))
    })?;
    globals.set("get_terminal_size", get_terminal_size)?;

    let get_text_width =
        lua.create_function(|_, text: String| {
            touch_mod_watchdog();
            Ok(UnicodeWidthStr::width(text.as_str()) as i64)
        })?;
    globals.set("get_text_width", get_text_width)?;

    let get_text_size = lua.create_function(|_, text: String| {
        touch_mod_watchdog();
        let mut max_width = 0usize;
        let mut height = 0i64;
        for line in text.split('\n') {
            max_width = max_width.max(UnicodeWidthStr::width(line));
            height += 1;
        }
        if text.is_empty() {
            height = 1;
        }
        Ok((max_width as i64, height))
    })?;
    globals.set("get_text_size", get_text_size)?;

    let resolve_x = lua.create_function(
        |_, (anchor, content_width, offset): (i64, i64, Option<i64>)| {
            touch_mod_watchdog();
            let (term_w, _) = crossterm::terminal::size().map_err(mlua::Error::external)?;
            let resolved = resolve_axis_position(
                anchor,
                term_w as i64,
                content_width.max(0),
                offset.unwrap_or(0),
                AxisOrientation::Horizontal,
            );
            Ok(resolved)
        },
    )?;
    globals.set("resolve_x", resolve_x)?;

    let resolve_y = lua.create_function(
        |_, (anchor, content_height, offset): (i64, i64, Option<i64>)| {
            touch_mod_watchdog();
            let (_, term_h) = crossterm::terminal::size().map_err(mlua::Error::external)?;
            let resolved = resolve_axis_position(
                anchor,
                term_h as i64,
                content_height.max(0),
                offset.unwrap_or(0),
                AxisOrientation::Vertical,
            );
            Ok(resolved)
        },
    )?;
    globals.set("resolve_y", resolve_y)?;

    let resolve_rect = lua.create_function(
        |_,
         (h_anchor, v_anchor, width, height, offset_x, offset_y): (
            i64,
            i64,
            i64,
            i64,
            Option<i64>,
            Option<i64>,
        )| {
            touch_mod_watchdog();
            let (term_w, term_h) = crossterm::terminal::size().map_err(mlua::Error::external)?;
            let x = resolve_axis_position(
                h_anchor,
                term_w as i64,
                width.max(0),
                offset_x.unwrap_or(0),
                AxisOrientation::Horizontal,
            );
            let y = resolve_axis_position(
                v_anchor,
                term_h as i64,
                height.max(0),
                offset_y.unwrap_or(0),
                AxisOrientation::Vertical,
            );
            Ok((x, y))
        },
    )?;
    globals.set("resolve_rect", resolve_rect)?;

    let get_launch_mode = lua.create_function(move |_, ()| Ok(mode.as_str().to_string()))?;
    globals.set("get_launch_mode", get_launch_mode)?;

    let save_data = lua.create_function(|_, (key, value): (String, Value)| {
        touch_mod_watchdog();
        save_lua_data(&key, &value)?;
        Ok(true)
    })?;
    globals.set("save_data", save_data)?;

    let load_data = lua.create_function(|lua, key: String| {
        touch_mod_watchdog();
        load_lua_data(lua, &key)
    })?;
    globals.set("load_data", load_data)?;

    let save_game_slot = lua.create_function(|_, (game_id, value): (String, Value)| {
        touch_mod_watchdog();
        save_game_slot_data(&game_id, &value)?;
        Ok(true)
    })?;
    globals.set("save_game_slot", save_game_slot)?;

    let load_game_slot =
        lua.create_function(|lua, game_id: String| {
            touch_mod_watchdog();
            load_lua_data(lua, &game_slot_key(&game_id))
        })?;
    globals.set("load_game_slot", load_game_slot)?;

    let update_game_stats =
        lua.create_function(|_, (game_id, score, duration_sec): (String, i64, i64)| {
            touch_mod_watchdog();
            let score_u32 = score.max(0).min(u32::MAX as i64) as u32;
            let duration_u64 = duration_sec.max(0) as u64;
            stats::update_game_stats(&game_id, score_u32, duration_u64)
                .map_err(mlua::Error::external)?;
            Ok(true)
        })?;
    globals.set("update_game_stats", update_game_stats)?;

    Ok(())
}

// 鍚姩娓告垙鑴氭湰锛屽苟澶勭悊绋嬪簭鎺у埗鏉?
pub fn run_game_script(script_path: &Path, mode: LaunchMode) -> Result<()> {
    drain_input_events();
    let source = fs::read_to_string(script_path)?;
    let source = source.trim_start_matches('\u{feff}');
    let lua = Lua::new();
    register_api(&lua, mode).map_err(|e| anyhow!("Lua API registration error: {e}"))?;
    load_text_functions(&lua, script_path)
        .map_err(|e| anyhow!("Lua text command registration error: {e}"))?;

    let result = match lua
        .load(source)
        .set_name(script_path.to_string_lossy())
        .exec()
    {
        Ok(()) => Ok(()),
        Err(err) if err.to_string().contains(EXIT_GAME_SENTINEL) => Ok(()),
        Err(err) => Err(anyhow!("Lua runtime error: {err}")),
    };

    finalize_terminal_after_script();
    TERMINAL_DIRTY_FROM_LUA.store(true, Ordering::Release);
    result
}

// 妫€鏌ヨ繖娈垫椂闂碙ua鏄惁瀵圭粓绔湁杈撳叆琛屼负
pub fn take_terminal_dirty_from_lua() -> bool {
    TERMINAL_DIRTY_FROM_LUA.swap(false, Ordering::AcqRel)
}

fn touch_mod_watchdog() {
}

// 浠庡瓨鍌ㄤ腑璇诲彇鏈€杩戜繚瀛樼殑瀛樻。ID
pub fn latest_saved_game_id() -> Option<String> {
    crate::core::save::latest_saved_game_id()
}

// 娓呯悊褰撳墠娓告垙鐨勫厓鏁版嵁鍜屽瓨妗ｆЫ浣?
// 涓嶆槸娓呯悊鍏ㄩ儴娓告垙鏁版嵁
pub fn clear_active_game_save() -> Result<()> {
    crate::core::save::clear_active_game_save()
}

fn load_text_functions(lua: &Lua, script_path: &Path) -> mlua::Result<()> {
    let globals = lua.globals();
    if globals.get::<Table>("TEXT_COMMANDS").is_err() {
        globals.set("TEXT_COMMANDS", lua.create_table()?)?;
    }

    let register = lua.create_function(|lua, (name, func): (String, Function)| {
        let globals = lua.globals();
        let table = match globals.get::<Table>("TEXT_COMMANDS") {
            Ok(t) => t,
            Err(_) => {
                let t = lua.create_table()?;
                globals.set("TEXT_COMMANDS", t.clone())?;
                t
            }
        };
        table.set(name.trim().to_ascii_lowercase(), func)?;
        Ok(true)
    })?;
    globals.set("register_text_command", register)?;

    let mut dirs = Vec::<PathBuf>::new();
    if let Some(parent) = script_path.parent() {
        dirs.push(parent.join("text_function"));
        if parent.file_name().and_then(|s| s.to_str()) == Some("game") {
            if let Some(root) = parent.parent() {
                dirs.push(root.join("text_function"));
            }
        }
    }
    if let Ok(scripts_dir) = path_utils::scripts_dir() {
        dirs.push(scripts_dir.join("text_function"));
    }

    let mut unique_dirs = Vec::<PathBuf>::new();
    for dir in dirs {
        if !unique_dirs.iter().any(|d| d == &dir) {
            unique_dirs.push(dir);
        }
    }

    let mut loaded_any = false;
    for dir in unique_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        let mut entries: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(mlua::Error::external)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("lua"))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort();

        for file in entries {
            let source = fs::read_to_string(&file).map_err(mlua::Error::external)?;
            let source = source.trim_start_matches('\u{feff}');
            lua.load(source)
                .set_name(file.to_string_lossy().as_ref())
                .exec()?;
            loaded_any = true;
        }
    }

    if !loaded_any {
        let globals = lua.globals();
        if globals.get::<Table>("TEXT_COMMANDS").is_err() {
            globals.set("TEXT_COMMANDS", lua.create_table()?)?;
        }
    }

    Ok(())
}

fn draw_text_rich_impl(
    lua: &Lua,
    x: i64,
    y: i64,
    text: &str,
    fg: Option<&str>,
    bg: Option<&str>,
) -> mlua::Result<()> {
    if !text.starts_with("f%") {
        return draw_text_impl(x, y, text, fg, bg);
    }

    let default_fg = fg.map(|v| v.to_string());
    let default_bg = bg.map(|v| v.to_string());

    let mut state = RichStyleState {
        default_fg: default_fg.clone(),
        default_bg: default_bg.clone(),
        fg: default_fg,
        bg: default_bg,
        fg_count: None,
        bg_count: None,
        fg_need_clear: false,
        bg_need_clear: false,
    };

    let body = &text[2..];
    let mut chunks = Vec::<StyledChunk>::new();

    let mut i = 0usize;
    while i < body.len() {
        let mut iter = body[i..].chars();
        let ch = match iter.next() {
            Some(c) => c,
            None => break,
        };
        let ch_len = ch.len_utf8();

        if ch == '\\' {
            if let Some(next_ch) = iter.next() {
                push_styled_char(&mut chunks, next_ch, &mut state);
                i += ch_len + next_ch.len_utf8();
            } else {
                push_styled_char(&mut chunks, '\\', &mut state);
                i += ch_len;
            }
            continue;
        }

        if ch == '{' {
            if let Some((inner, consumed)) = read_command_block(body, i)? {
                if inner.trim().is_empty() {
                    push_error(&mut chunks, &rich_text_error("rich_text.error.empty_command"));
                    i += consumed;
                    continue;
                }

                match apply_command_block(lua, &inner, &mut state) {
                    Ok(()) => {}
                    Err(msg) => push_error(&mut chunks, &msg.to_string()),
                }

                i += consumed;
                continue;
            }

            push_error(
                &mut chunks,
                &rich_text_error("rich_text.error.unclosed_command"),
            );
            i += ch_len;
            continue;
        }

        if ch == '}' {
            push_error(
                &mut chunks,
                &rich_text_error("rich_text.error.unclosed_command"),
            );
            i += ch_len;
            continue;
        }

        push_styled_char(&mut chunks, ch, &mut state);
        i += ch_len;
    }

    if state.fg_need_clear || state.bg_need_clear {
        push_error(
            &mut chunks,
            &rich_text_error("rich_text.error.unterminated_style"),
        );
    }

    draw_styled_chunks(x, y, &chunks)
}

fn read_command_block(input: &str, start: usize) -> mlua::Result<Option<(String, usize)>> {
    let mut i = start + '{'.len_utf8();
    let mut escape = false;
    while i < input.len() {
        let c = match input[i..].chars().next() {
            Some(v) => v,
            None => break,
        };
        let clen = c.len_utf8();

        if escape {
            escape = false;
            i += clen;
            continue;
        }

        if c == '\\' {
            escape = true;
            i += clen;
            continue;
        }

        if c == '}' {
            let inner = input[start + 1..i].to_string();
            return Ok(Some((inner, i + clen - start)));
        }

        i += clen;
    }
    Ok(None)
}

fn split_unescaped(input: &str, sep: char) -> Vec<String> {
    let mut out = Vec::<String>::new();
    let mut cur = String::new();
    let mut escape = false;

    for c in input.chars() {
        if escape {
            cur.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == sep {
            out.push(cur.trim().to_string());
            cur.clear();
            continue;
        }
        cur.push(c);
    }

    if escape {
        cur.push('\\');
    }

    out.push(cur.trim().to_string());
    out
}

fn apply_command_block(lua: &Lua, block: &str, state: &mut RichStyleState) -> mlua::Result<()> {
    let entries = split_unescaped(block, '|');
    for entry in entries {
        if entry.trim().is_empty() {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.empty_command",
            )));
        }

        let mut parts = split_unescaped(&entry, ':');
        if parts.len() != 2 {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.missing_command_or_param",
            )));
        }

        let cmd = parts.remove(0).trim().to_ascii_lowercase();
        let param_expr = parts.remove(0);
        let params = split_unescaped(&param_expr, '>');

        if cmd.is_empty() {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.missing_command_or_param",
            )));
        }

        let result = apply_single_command(lua, &cmd, &params)?;
        apply_command_result(&cmd, result, state)?;
    }
    Ok(())
}

fn apply_single_command(
    lua: &Lua,
    cmd: &str,
    params: &[String],
) -> mlua::Result<TextCommandResult> {
    if let Some(via_lua) = apply_command_via_lua(lua, cmd, params)? {
        return Ok(via_lua);
    }

    if params.is_empty() || params[0].trim().is_empty() {
        return Err(mlua::Error::external(rich_text_error(
            "rich_text.error.missing_param",
        )));
    }

    let first = params[0].trim();
    if first.eq_ignore_ascii_case("clear") {
        if params.len() != 1 {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.unterminated_style",
            )));
        }
        return Ok(TextCommandResult {
            clear: true,
            color: None,
            count: None,
        });
    }

    if parse_color(Some(first)).is_none() {
        return Err(mlua::Error::external(rich_text_error(
            "rich_text.error.invalid_param",
        )));
    }

    let count = if params.len() >= 2 && !params[1].trim().is_empty() {
        let raw = params[1]
            .trim()
            .parse::<usize>()
            .map_err(|_| mlua::Error::external(rich_text_error("rich_text.error.invalid_param")))?;
        if raw == 0 {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.invalid_param",
            )));
        }
        Some(raw)
    } else {
        None
    };

    if params.len() > 2 {
        return Err(mlua::Error::external(rich_text_error(
            "rich_text.error.invalid_param",
        )));
    }

    Ok(TextCommandResult {
        clear: false,
        color: Some(first.to_string()),
        count,
    })
}

fn apply_command_via_lua(
    lua: &Lua,
    cmd: &str,
    params: &[String],
) -> mlua::Result<Option<TextCommandResult>> {
    let globals = lua.globals();
    let commands = match globals.get::<Table>("TEXT_COMMANDS") {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    let func = match commands.get::<Function>(cmd) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };

    let ptable = lua.create_table()?;
    for (idx, p) in params.iter().enumerate() {
        ptable.set((idx + 1) as i64, p.as_str())?;
    }

    let ret = func.call::<Value>(ptable)?;
    let t = match ret {
        Value::Table(t) => t,
        _ => {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.invalid_return_value",
            )));
        }
    };

    if let Ok(msg) = t.get::<String>("error") {
        if !msg.trim().is_empty() {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.invalid_custom_command",
            )));
        }
    }

    let clear = t.get::<bool>("clear").unwrap_or(false);
    let color = t.get::<String>("color").ok();
    let count = t
        .get::<i64>("count")
        .ok()
        .and_then(|v| if v > 0 { Some(v as usize) } else { None });

    if !clear {
        if let Some(c) = color.as_deref() {
            if parse_color(Some(c)).is_none() {
                return Err(mlua::Error::external(rich_text_error(
                    "rich_text.error.invalid_param",
                )));
            }
        } else {
            return Err(mlua::Error::external(rich_text_error(
                "rich_text.error.invalid_param",
            )));
        }
    }

    Ok(Some(TextCommandResult {
        clear,
        color,
        count,
    }))
}

fn apply_command_result(
    cmd: &str,
    result: TextCommandResult,
    state: &mut RichStyleState,
) -> mlua::Result<()> {
    match cmd {
        "tc" => {
            if result.clear {
                state.fg = state.default_fg.clone();
                state.fg_count = None;
                state.fg_need_clear = false;
                return Ok(());
            }
            let color = result
                .color
                .ok_or_else(|| mlua::Error::external(rich_text_error("rich_text.error.missing_param")))?;
            state.fg = Some(color);
            state.fg_count = result.count;
            state.fg_need_clear = result.count.is_none();
            Ok(())
        }
        "bg" => {
            if result.clear {
                state.bg = state.default_bg.clone();
                state.bg_count = None;
                state.bg_need_clear = false;
                return Ok(());
            }
            let color = result
                .color
                .ok_or_else(|| mlua::Error::external(rich_text_error("rich_text.error.missing_param")))?;
            state.bg = Some(color);
            state.bg_count = result.count;
            state.bg_need_clear = result.count.is_none();
            Ok(())
        }
        _ => Err(mlua::Error::external(rich_text_error(
            "rich_text.error.unknown_command",
        ))),
    }
}


// 瀵屾枃鏈潡缁撴瀯浣?
#[derive(Clone, Debug)]
struct StyledChunk {
    text: String,
    fg: Option<String>, // 鍓嶆櫙鑹插悕绉?
    bg: Option<String>, // 鑳屾櫙鑹插悕绉?
}

// 瀵屾枃鏈牱寮忕粨鏋勪綋鐘舵€佹満
#[derive(Clone, Debug)]
struct RichStyleState {
    default_fg: Option<String>, // 榛樿鍓嶆櫙鑹诧紙浠巇raw_text鍙傛暟浼犲叆锛?
    default_bg: Option<String>, // 榛樿鑳屾櫙鑹诧紙浠巇raw_text鍙傛暟浼犲叆锛?
    fg: Option<String>,         // 褰撳墠鍓嶆櫙鑹?
    bg: Option<String>,         // 褰撳墠鑳屾櫙鑹?
    fg_count: Option<usize>,    // 鍓嶆櫙鑹插墿浣欑敓鏁堝瓧绗︽暟
    bg_count: Option<usize>,    // 鑳屾櫙鑹插墿浣欑敓鏁堝瓧绗︽暟
    fg_need_clear: bool,        // 鏄惁闇€瑕佽嚜鍔ㄦ竻闄ゅ墠鏅壊锛堝綋count涓篘one鏃讹級
    bg_need_clear: bool,        // 鏄惁闇€瑕佽嚜鍔ㄦ竻闄よ儗鏅壊锛堝綋count涓篘one鏃讹級
}

// 瀵屾枃鏈懡浠よ繑鍥炵粨鏋滅粨鏋勪綋
#[derive(Clone, Debug)]
struct TextCommandResult {
    clear: bool,
    color: Option<String>,
    count: Option<usize>,
}

fn rich_text_error(key: &str) -> String {
    i18n::t(key).to_string()
}


// 鎶涘嚭寮傚父
fn push_error(chunks: &mut Vec<StyledChunk>, message: &str) {
    push_styled_text(
        chunks,
        &format!("{{{message}}}"),
        Some("red".to_string()),
        None,
    );
}

// 澶勭悊瀛楃娓叉煋闀垮害
fn push_styled_char(chunks: &mut Vec<StyledChunk>, ch: char, state: &mut RichStyleState) {
    // 灏嗗瓧绗﹁浆鎹负瀛楃涓插苟娣诲姞鍒板潡鍒楄〃
    let mut s = String::new();
    s.push(ch);
    push_styled_text(chunks, &s, state.fg.clone(), state.bg.clone());

    // 澶勭悊瀛椾綋棰滆壊
    if let Some(rem) = state.fg_count {
        if rem <= 1 {
            state.fg_count = None;
            state.fg = state.default_fg.clone();
        } else {
            state.fg_count = Some(rem - 1);
        }
    }

    // 澶勭悊鑳屾櫙棰滆壊
    if let Some(rem) = state.bg_count {
        if rem <= 1 {
            state.bg_count = None;
            state.bg = state.default_bg.clone();
        } else {
            state.bg_count = Some(rem - 1);
        }
    }
}

// 鏂囨湰娣诲姞鍜屽悎骞?鍑忓皯缁堢鐨勮皟鐢ㄥ拰鍛戒护鎵ц鎻愰珮鏁堢巼
fn push_styled_text(
    chunks: &mut Vec<StyledChunk>,
    text: &str,
    fg: Option<String>,
    bg: Option<String>,
) {
    // 蹇界暐绌烘枃鏈?
    if text.is_empty() {
        return;
    }

    // 妫€鏌ユ槸鍚﹀彲浠ュ悎骞?
    if let Some(last) = chunks.last_mut() {
        if last.fg == fg && last.bg == bg {
            last.text.push_str(text);
            return;
        }
    }

    // 鏍峰紡涓嶅悓灏卞垱寤烘柊鐨勫潡
    chunks.push(StyledChunk {
        text: text.to_string(),
        fg,
        bg,
    });
}

// 璁＄畻鏍峰紡鍧楁牱寮忓潡娓叉煋
fn draw_styled_chunks(x: i64, y: i64, chunks: &[StyledChunk]) -> mlua::Result<()> {
    // 褰撳墠鍏夋爣鏈煡
    let mut cursor_x = x;

    for chunk in chunks {
        // 璺宠繃绌哄潡
        if chunk.text.is_empty() {
            continue;
        }

        // 缁樺埗褰撳墠鍧?
        draw_text_impl(
            cursor_x,
            y,
            &chunk.text,
            chunk.fg.as_deref(),
            chunk.bg.as_deref(),
        )?;

        // 璁＄畻鏂囨湰鐨勫疄闄呭搴﹀苟绉诲姩鍏夋爣
        cursor_x += UnicodeWidthStr::width(chunk.text.as_str()) as i64;
    }
    Ok(())
}

// 瀹為檯鐨勭粯鍒跺嚱鏁?
fn draw_text_impl(
    x: i64,
    y: i64,
    text: &str,
    fg: Option<&str>,
    bg: Option<&str>,
) -> mlua::Result<()> {
    // 鑾峰彇缁堢杈撳嚭鐨勯攣
    let mut out = lock_out()?;

    // 璁剧疆鏂囧瓧棰滆壊
    if let Some(color) = parse_color(fg) {
        queue!(out, SetForegroundColor(color)).map_err(mlua::Error::external)?;
    }

    // 璁剧疆鑳屾櫙鑹?
    if let Some(color) = parse_color(bg) {
        queue!(out, SetBackgroundColor(color)).map_err(mlua::Error::external)?;
    }

    // 绉诲姩鍏夋爣骞惰緭鍑烘枃鏈紝鐒跺悗閲嶇疆棰滆壊
    queue!(
        out,
        crossterm::cursor::MoveTo(coord_to_terminal(x), coord_to_terminal(y)),
        Print(text),
        ResetColor
    )
    .map_err(mlua::Error::external)?;
    Ok(())
}

// 鍏ㄥ眬浜掓枼閿?閬垮厤澶氫釜绾跨▼鍚屾椂鍐欏叆缁堢
fn lock_out() -> mlua::Result<MutexGuard<'static, Stdout>> {
    OUT.lock()
        .map_err(|_| mlua::Error::external("stdout lock poisoned"))
}

// 寮哄埗灏嗙紦鍐插尯鐨勫唴瀹硅緭鍑哄埌缁堢
fn flush_output() -> mlua::Result<()> {
    let mut out = lock_out()?;
    out.flush().map_err(mlua::Error::external)
}

// Lua鎵ц瀹屽悗,閲嶇疆缁堢鐘舵€佸苟娓呯┖杈撳叆缂撳啿鍖?
fn finalize_terminal_after_script() {
    if let Ok(mut out) = OUT.lock() {
        let _ = queue!(out, ResetColor, crossterm::cursor::MoveTo(0, 0));
        let _ = out.flush();
    }

    drain_input_events();
}

// 娓呯┖杈撳叆缂撳啿鍖?
fn drain_input_events() {
    loop {
        match event::poll(Duration::from_millis(0)) {
            Ok(true) => {
                let _ = event::read();
            }
            _ => break,
        }
    }
}

// 灏哻rossterm鐨凨eyCode鏋氫妇杞崲涓篖ua鍙瘑鍒殑瀛楃涓?
fn keycode_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::BackTab => "tab".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Char(' ') => "space".to_string(),
        KeyCode::Char(c) => c.to_ascii_lowercase().to_string(),
        _ => String::new(),
    }
}

// 澶勭悊鎸夐敭浜嬩欢鐩戝惉
fn decode_key_event(key: KeyEvent) -> mlua::Result<String> {
    // 涓嶆槸ESC鍒欑洿鎺ヨ浆鎹?
    if key.code != KeyCode::Esc {
        return Ok(keycode_to_string(key.code));
    }

    // 濡傛灉鏄疎SC鐪嬫槸鍚﹂渶瑕佺壒娈婅浆鎹?
    // 鏈変簺鐗规畩閿槸 ESC [ X
    if let Some(mapped) = try_read_escaped_arrow()? {
        // 杩斿洖瑙ｆ瀽
        return Ok(mapped);
    }

    // 鎴栬€呯湡鐨勬槸ESC
    Ok("esc".to_string())
}

// 鍒ゆ柇ESC [ X 杞崲
fn try_read_escaped_arrow() -> mlua::Result<Option<String>> {
    // 妫€鏌ユ槸鍚︽湁涓嬩竴涓簨浠?绛夊緟2sm)
    if !event::poll(Duration::from_millis(2)).map_err(mlua::Error::external)? {
        return Ok(None);
    }

    // 璇诲彇绗竴涓瓧绗?
    let first = match event::read().map_err(mlua::Error::external)? {
        Event::Key(k) if k.kind == KeyEventKind::Press => k,
        _ => return Ok(None),
    };

    // 璇诲彇绗簩涓瓧绗︽槸[杩樻槸O
    let prefix_ok = matches!(first.code, KeyCode::Char('[') | KeyCode::Char('O'));
    if !prefix_ok {
        return Ok(None);
    }

    // 璇诲彇绗笁涓瓧绗︼紝搴旇鏄?A/B/C/D
    if !event::poll(Duration::from_millis(2)).map_err(mlua::Error::external)? {
        return Ok(None);
    }
    let second = match event::read().map_err(mlua::Error::external)? {
        Event::Key(k) if k.kind == KeyEventKind::Press => k,
        _ => return Ok(None),
    };

    // 鏄犲皠涓烘柟鍚戦敭
    let mapped = match second.code {
        KeyCode::Char('A') | KeyCode::Char('a') => Some("up".to_string()),
        KeyCode::Char('B') | KeyCode::Char('b') => Some("down".to_string()),
        KeyCode::Char('C') | KeyCode::Char('c') => Some("right".to_string()),
        KeyCode::Char('D') | KeyCode::Char('d') => Some("left".to_string()),
        _ => None,
    };
    Ok(mapped)
}

// Lua鍧愭爣杞崲鏈粓绔潗鏍?1-base -> 0-base)
fn coord_to_terminal(v: i64) -> u16 {
    if v <= 0 {
        0
    } else {
        (v - 1).min(u16::MAX as i64) as u16
    }
}

// 棰滆壊瑙ｆ瀽
fn parse_color(name: Option<&str>) -> Option<CColor> {
    let raw = name.unwrap_or("").trim();

    // 瑙ｆ瀽鍗佸叚杩涘埗
    if let Some(hex) = parse_hex_color(raw) {
        return Some(hex);
    }

    // 瑙ｆ瀽RGB
    if let Some(rgb) = parse_rgb_color(raw) {
        return Some(rgb);
    }

    // 瑙ｆ瀽棰勮棰滆壊鍚?
    match raw.to_ascii_lowercase().as_str() {
        "black" => Some(CColor::Black),
        "white" => Some(CColor::White),
        "red" => Some(CColor::Red),
        "light_red" => Some(CColor::Red),
        "dark_red" => Some(CColor::DarkRed),
        "yellow" => Some(CColor::Yellow),
        "light_yellow" => Some(CColor::Yellow),
        "dark_yellow" => Some(CColor::DarkYellow),
        "orange" => Some(CColor::DarkYellow),
        "green" => Some(CColor::Green),
        "light_green" => Some(CColor::Green),
        "blue" => Some(CColor::Blue),
        "light_blue" => Some(CColor::Blue),
        "cyan" => Some(CColor::Cyan),
        "light_cyan" => Some(CColor::Cyan),
        "magenta" => Some(CColor::Magenta),
        "light_magenta" => Some(CColor::Magenta),
        "grey" | "gray" => Some(CColor::Grey),
        "dark_grey" | "dark_gray" => Some(CColor::DarkGrey),
        _ => None, // 鏈煡棰滆壊
    }
}

// 瑙ｆ瀽鍗佸叚杩涘埗
fn parse_hex_color(raw: &str) -> Option<CColor> {
    // 鏄?涓瓧绗﹀苟涓斾互#寮€澶?
    if raw.len() != 7 || !raw.starts_with('#') {
        return None;
    }
    // 瑙ｆ瀽鍗佸叚杩涘埗鏁?
    let r = u8::from_str_radix(&raw[1..3], 16).ok()?;
    let g = u8::from_str_radix(&raw[3..5], 16).ok()?;
    let b = u8::from_str_radix(&raw[5..7], 16).ok()?;

    // RGB
    Some(CColor::Rgb { r, g, b })
}

// 瑙ｆ瀽RGB
fn parse_rgb_color(raw: &str) -> Option<CColor> {
    let lower = raw.to_ascii_lowercase();

    // 鏍煎紡妫€鏌?
    if !lower.starts_with("rgb(") || !lower.ends_with(')') {
        return None;
    }

    // 鎻愬彇鍐呭
    let inner = &lower[4..lower.len() - 1];

    // 鎸夐€楀彿鍒嗗壊骞惰В鏋愭湭u8
    let mut parts = inner.split(',').map(|s| s.trim().parse::<u8>().ok());

    let r = parts.next()??;
    let g = parts.next()??;
    let b = parts.next()??;

    // 纭繚娌℃湁澶氫綑鐨勯儴鍒?
    if parts.next().is_some() {
        return None;
    }

    // RGB
    Some(CColor::Rgb { r, g, b })
}

// 闅忔満鏁扮敓鎴愬櫒
// 绾跨▼瀹夊叏锛屼娇鐢ㄤ簡xorshift绠楁硶
fn next_random_u64() -> u64 {
    // 鑾峰彇褰撳墠鐘舵€?
    let mut cur = RNG_STATE.load(Ordering::Relaxed);

    // 濡傛灉鏄涓€娆¤皟鐢紝灏卞垵濮嬪寲绉嶅瓙
    if cur == 0 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15); // 鍥為€€绉嶅瓙
        let seeded = if seed == 0 {
            0xA409_3822_299F_31D0 // 澶囩敤绉嶅瓙
        } else {
            seed
        };

        // 鍘熷瓙鎿嶄綔璁剧疆绉嶅瓙
        let _ = RNG_STATE.compare_exchange(0, seeded, Ordering::SeqCst, Ordering::Relaxed);
        cur = RNG_STATE.load(Ordering::Relaxed);
    }

    // xorshift鐢熸垚涓嬩竴涓殢鏈烘暟
    loop {
        let mut x = cur;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        // 闃叉鍑虹幇0
        if x == 0 {
            x = 0x2545_F491_4F6C_DD1D;
        }

        // 鍘熷瓙鏇存柊鐘舵€?濡傛灉琚叾瀹冪嚎绋嬩慨鏀瑰垯閲嶈瘯
        match RNG_STATE.compare_exchange(cur, x, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => return x,
            Err(actual) => cur = actual,
        }
    }
}

// 鑾峰彇Lua鏁版嵁淇濆瓨鐨勮矾寰?
fn save_file_path() -> PathBuf {
    match path_utils::lua_saves_file() {
        Ok(path) => path,
        Err(_) => PathBuf::from("lua_saves.json"),
    }
}

// 浠庢枃浠跺姞杞絁SON瀛樺偍瀵硅薄,涓嶅瓨鍦ㄥ氨鍒涘缓鏂囦欢
fn load_json_store() -> mlua::Result<Map<String, JsonValue>> {
    // 鑾峰彇璺緞
    let path = save_file_path();
    
    // 濡傛灉涓嶅瓨鍦ㄥ氨鍒涘缓绌烘枃浠?
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(mlua::Error::external)?;
        }
        // 鍐欏叆绌哄璞?
        fs::write(&path, "{}").map_err(mlua::Error::external)?;
        return Ok(Map::new());
    }

    // 璇诲彇骞惰В鏋愮幇鏈夋枃浠?
    let raw = fs::read_to_string(path).map_err(mlua::Error::external)?;
    let parsed = serde_json::from_str::<JsonValue>(&raw).unwrap_or(JsonValue::Object(Map::new()));

    // 纭杩斿洖鐨勬槸瀵硅薄绫诲瀷
    if let JsonValue::Object(map) = parsed {
        Ok(map)
    } else {
        // 涓嶆槸灏辫繑鍥炵┖瀵硅薄
        Ok(Map::new())
    }
}

// 灏嗗瓨鍌ㄥ璞″啓鍏son
fn write_json_store(store: &Map<String, JsonValue>) -> mlua::Result<()> {
    let path = save_file_path();

    // 纭繚鐖剁洰褰曞瓨鍦?
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(mlua::Error::external)?;
    }
    // 灏哅ap杞崲涓烘牸寮忓寲鐨凧SON瀛楃涓?
    let payload = serde_json::to_string_pretty(store).map_err(mlua::Error::external)?;

    // 鍐欏叆鏂囦欢
    fs::write(path, payload).map_err(mlua::Error::external)?;
    Ok(())
}

// 淇濆瓨Lua鏁版嵁
fn save_lua_data(key: &str, value: &Value) -> mlua::Result<()> {
    // 鍔犺浇褰撳墠瀛樺偍
    let mut store = load_json_store()?;

    // 灏哃ua鍊艰浆鎹负JSON
    let json = lua_to_json(value)?;

    // 鎻掑叆鎴栨洿鏂伴敭鍊煎
    // 鎵€浠ヨ 閿€煎 鍜?閿鍊?搴旇鏄竴涓剰鎬濆惂
    store.insert(key.to_string(), json);

    // 鍐欏洖鏂囦欢
    write_json_store(&store)
}

// 淇濆瓨娓告垙瀛樻。,鑷姩娓呯悊鏃х殑瀛樻。,骞惰褰曟柊鐨勫唴瀹?
fn save_game_slot_data(game_id: &str, value: &Value) -> mlua::Result<()> {
    // 鍔犺浇褰撳墠瀛樻。
    let mut store = load_json_store()?;
    
    // 娓呯悊鏃у瓨妗?
    clear_game_slots(&mut store);
    
    // 杞崲瀛樻。鏁版嵁
    let json = lua_to_json(value)?;
    let game_id = game_id.trim().to_ascii_lowercase();
    
    // 淇濆瓨鏂板瓨妗?
    store.insert(game_slot_key(&game_id), json);
    
    // 璁板綍鏈€鏂板瓨妗D
    store.insert("__latest_save_game".to_string(), JsonValue::String(game_id));
    
    // 鍐欏洖鏂囦欢
    write_json_store(&store)
}


// 娓呯悊娓告垙瀛樻。
fn clear_game_slots(store: &mut Map<String, JsonValue>) {
    store.retain(|key, _| key != "__latest_save_game" && !key.starts_with("game:"));
}

// 灏嗘父鎴廔D杞崲涓哄瓨鍌ㄩ敭鍚?
fn game_slot_key(game_id: &str) -> String {
    format!("game:{}", game_id.trim().to_ascii_lowercase())
}

// 浠庡瓨鍌ㄤ腑鍔犺浇鎸囧畾閿悕,骞惰浆鎹负Lua鍊?
fn load_lua_data(lua: &Lua, key: &str) -> mlua::Result<Value> {
    let store = load_json_store()?;
    
    if let Some(v) = store.get(key) {
        // 閿瓨鍦?灏咼SON杞崲鍥濴ua鍊?
        json_to_lua(lua, v)
    } else {
        // 閿笉瀛樺湪,杩斿洖nil
        Ok(Value::Nil)
    }
}

// 灏哃ua鍊艰浆鎹负JSON鍊?
fn lua_to_json(value: &Value) -> mlua::Result<JsonValue> {
    match value {
        // 鍩烘湰绫诲瀷鐩存帴杞崲
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(v) => Ok(JsonValue::Bool(*v)),
        Value::Integer(v) => Ok(JsonValue::Number(Number::from(*v))),
        Value::Number(v) => Number::from_f64(*v)
            // f65鍙兘鏃犳硶绮惧噯杞崲鎴怞SON Number
            .map(JsonValue::Number)
            .ok_or_else(|| mlua::Error::external("invalid lua number")),
        Value::String(v) => Ok(JsonValue::String(v.to_str()?.to_string())),

        // 琛ㄧ殑鍑嗘崲闇€瑕佺壒娈婂鐞?
        Value::Table(t) => table_to_json(t),

        // 涓嶆敮鎸佺殑绫诲瀷鏃ф姏鍑哄紓甯?
        _ => Err(mlua::Error::external(
            "unsupported lua value type for save_data",
        )),
    }
}

// Lua琛ㄨ浆JSON绫诲瀷
fn table_to_json(table: &Table) -> mlua::Result<JsonValue> {
    let mut as_array: BTreeMap<usize, JsonValue> = BTreeMap::new();
    let mut as_object = Map::new();
    let mut array_only = true; // 鍋囪琛ㄩ粯璁ゆ槸涓€涓函鏁扮粍

    // 閬嶅巻鎵€鏈夌殑閿€煎
    for pair in table.pairs::<Value, Value>() {
        let (k, v) = pair?;
        match k {
            // 姝ｆ暣鏁伴敭 -> 鍙兘鏄暟缁勫厓绱?
            Value::Integer(i) if i > 0 => as_array.insert(i as usize, lua_to_json(&v)?),

            // 瀛楃涓查敭 -> 涓€瀹氭槸瀵硅薄
            Value::String(s) => {
                array_only = false;
                as_object.insert(s.to_str()?.to_string(), lua_to_json(&v)?);
                None
            }

            // 鍏朵粬绫诲瀷閿紙璐熸暟銆佹诞鐐规暟绛夛級鈫?杞负瀛楃涓蹭綔涓哄璞￠敭
            _ => {
                array_only = false;
                as_object.insert(format!("{k:?}"), lua_to_json(&v)?);
                None
            }
        };
    }

    // 鍒ゆ柇鏄暟缁勮繕鏄璞?
    if array_only && !as_array.is_empty() {
        // 绾暟缁?-> 杞崲涓篔SON鏁扮粍
        let mut list = Vec::new();
        let max = *as_array.keys().max().unwrap_or(&0);
        for idx in 1..=max {
            if let Some(v) = as_array.get(&idx) {
                list.push(v.clone());
            } else {
                // 璺宠繃鐨勭储寮曠敤null濉厖
                list.push(JsonValue::Null);
            }
        }
        Ok(JsonValue::Array(list))
    } else {
        for (k, v) in as_array {
            as_object.insert(k.to_string(), v);
        }
        Ok(JsonValue::Object(as_object))
    }
}

// JSON杞琇ua
fn json_to_lua(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    match value {
        // 鍩烘湰绫诲瀷鐩存帴杞崲
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(v) => Ok(Value::Boolean(*v)),
        JsonValue::Number(v) => {
            // 鍏堣浆鎹㈡垚鏁存暟,鍚﹀垯灏辫浆鎹负娴偣鏁?
            if let Some(i) = v.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = v.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(v) => Ok(Value::String(lua.create_string(v)?)),

        // JSON鏁扮粍 -> Lua琛?
        JsonValue::Array(items) => {
            let t = lua.create_table()?;
            for (idx, item) in items.iter().enumerate() {
                t.set((idx + 1) as i64, json_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(t))
        }

        // JSON瀵硅薄 -> Lua琛?
        JsonValue::Object(map) => {
            let t = lua.create_table()?;
            for (k, v) in map {
                t.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(t))
        }
    }
}


