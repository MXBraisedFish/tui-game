//! 富文本解析器

use serde_json::Value as JsonValue;

use crate::host_engine::boot::i18n::i18n;
use crate::host_engine::boot::preload::lua_runtime::api::debug_support::key_display;
use crate::host_engine::boot::preload::lua_runtime::{LuaRuntimeConsumer, LuaRuntimeContext};
use crate::host_engine::boot::preload::persistent_data::keybind_profile;

use super::drawing_parser::{
    STYLE_BLINK, STYLE_BOLD, STYLE_DIM, STYLE_HIDDEN, STYLE_ITALIC, STYLE_NORMAL, STYLE_REVERSE,
    STYLE_STRIKE, STYLE_UNDERLINE,
};

const RICH_TEXT_PREFIX: &str = "f%";
const MISSING_KEY_TEXT: &str = "[None]";
const ERROR_FG_COLOR: &str = "red";

/// 带样式的单个终端字符。
#[derive(Clone, Debug)]
pub struct StyledCharacter {
    pub character: char,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub styles: Vec<i64>,
    pub style_explicit: bool,
}

#[derive(Clone, Debug, Default)]
struct ActiveStyle {
    fg: Option<TimedValue<String>>,
    bg: Option<TimedValue<String>>,
    styles: Vec<TimedValue<i64>>,
}

#[derive(Clone, Debug)]
struct TimedValue<T> {
    value: T,
    remaining_chars: Option<usize>,
}

/// 解析富文本为可绘制的字符序列。
pub fn parse_rich_text(
    rich_text: &str,
    runtime_context: &LuaRuntimeContext,
) -> mlua::Result<Vec<StyledCharacter>> {
    let source = rich_text
        .strip_prefix(RICH_TEXT_PREFIX)
        .unwrap_or(rich_text);
    let mut output = Vec::new();
    let mut active_style = ActiveStyle::default();
    let source_chars = source.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < source_chars.len() {
        let character = source_chars[index];
        match character {
            '\\' => {
                emit_escaped_or_literal_backslash(
                    &source_chars,
                    &mut index,
                    &mut active_style,
                    &mut output,
                );
            }
            '{' => {
                if let Some((command_text, end_index)) = read_command_text(&source_chars, index + 1)
                {
                    apply_command_block(
                        command_text.as_str(),
                        runtime_context,
                        &mut active_style,
                        &mut output,
                    );
                    index = end_index + 1;
                } else {
                    emit_character(character, &active_style, &mut output);
                    active_style.tick();
                    index += 1;
                }
            }
            '}' => {
                emit_character(character, &active_style, &mut output);
                active_style.tick();
                index += 1;
            }
            _ => {
                emit_character(character, &active_style, &mut output);
                active_style.tick();
                index += 1;
            }
        }
    }

    Ok(output)
}

fn read_command_text(source_chars: &[char], mut index: usize) -> Option<(String, usize)> {
    let mut command_text = String::new();
    while index < source_chars.len() {
        let character = source_chars[index];
        match character {
            '\\' => {
                command_text.push(character);
                index += 1;
                if let Some(escaped_character) = source_chars.get(index) {
                    command_text.push(*escaped_character);
                    index += 1;
                }
            }
            '}' => return Some((command_text, index)),
            '{' => return None,
            _ => {
                command_text.push(character);
                index += 1;
            }
        }
    }
    None
}

fn apply_command_block(
    command_text: &str,
    runtime_context: &LuaRuntimeContext,
    active_style: &mut ActiveStyle,
    output: &mut Vec<StyledCharacter>,
) {
    for command in split_command_block(command_text) {
        let command = command.trim();
        if command.is_empty() {
            emit_unknown_command(output);
            continue;
        }
        apply_single_command(command, runtime_context, active_style, output);
    }
}

fn split_command_block(command_text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current_command = String::new();
    let mut escaped = false;

    for character in command_text.chars() {
        if escaped {
            current_command.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == '|' {
            commands.push(std::mem::take(&mut current_command));
        } else {
            current_command.push(character);
        }
    }

    if escaped {
        current_command.push('\\');
    }
    commands.push(current_command);
    commands
}

fn apply_single_command(
    command: &str,
    runtime_context: &LuaRuntimeContext,
    active_style: &mut ActiveStyle,
    output: &mut Vec<StyledCharacter>,
) {
    if command.trim() == "reset" {
        *active_style = ActiveStyle::default();
        return;
    }

    let Some((command_name, parameter_text)) = command.split_once(':') else {
        emit_unknown_command(output);
        return;
    };

    match command_name.trim() {
        "tc" => apply_color_command(parameter_text, &mut active_style.fg, output),
        "bg" => apply_color_command(parameter_text, &mut active_style.bg, output),
        "ts" => apply_text_style_command(parameter_text, active_style, output),
        "key" => {
            let key_text = resolve_key_text(parameter_text, runtime_context);
            for character in key_text.chars() {
                emit_character(character, active_style, output);
                active_style.tick();
            }
        }
        _ => emit_unknown_command(output),
    }
}

fn apply_color_command(
    parameter_text: &str,
    target: &mut Option<TimedValue<String>>,
    output: &mut Vec<StyledCharacter>,
) {
    let Some((value, remaining_chars)) = parse_value_and_count(parameter_text) else {
        emit_unknown_parameter(output);
        return;
    };
    if value == "clear" {
        *target = None;
        return;
    }
    if !is_valid_color(value.as_str()) {
        emit_unknown_parameter(output);
        return;
    }
    *target = Some(TimedValue {
        value,
        remaining_chars,
    });
}

fn apply_text_style_command(
    parameter_text: &str,
    active_style: &mut ActiveStyle,
    output: &mut Vec<StyledCharacter>,
) {
    let Some((style_text, remaining_chars)) = parse_value_and_count(parameter_text) else {
        emit_unknown_parameter(output);
        return;
    };
    if style_text == "clear" {
        active_style.styles.clear();
        return;
    }
    if style_text == "normal" {
        active_style.styles.clear();
        active_style.styles.push(TimedValue {
            value: STYLE_NORMAL,
            remaining_chars: remaining_chars.or(Some(usize::MAX)),
        });
        return;
    }

    for style_name in style_text.split('+') {
        let Some(style) = parse_style_name(style_name.trim()) else {
            emit_unknown_parameter(output);
            return;
        };
        if style == STYLE_NORMAL {
            active_style.styles.clear();
            active_style.styles.push(TimedValue {
                value: STYLE_NORMAL,
                remaining_chars: remaining_chars.or(Some(usize::MAX)),
            });
            continue;
        }
        if let Some(existing_style) = active_style
            .styles
            .iter_mut()
            .find(|existing_style| existing_style.value == style)
        {
            existing_style.remaining_chars = remaining_chars;
        } else {
            active_style.styles.push(TimedValue {
                value: style,
                remaining_chars,
            });
        }
    }
}

fn parse_value_and_count(parameter_text: &str) -> Option<(String, Option<usize>)> {
    let mut parts = parameter_text.splitn(2, '>');
    let value = parts.next().unwrap_or_default().trim();
    if value.is_empty() {
        return None;
    }
    let count = match parts.next() {
        Some(count_text) => {
            let count = count_text.trim().parse::<usize>().ok()?;
            if count == 0 {
                return None;
            }
            Some(count)
        }
        None => None,
    };

    Some((value.to_string(), count))
}

fn parse_style_name(style_name: &str) -> Option<i64> {
    match style_name {
        "normal" => Some(STYLE_NORMAL),
        "bold" => Some(STYLE_BOLD),
        "italic" => Some(STYLE_ITALIC),
        "underline" => Some(STYLE_UNDERLINE),
        "strike" => Some(STYLE_STRIKE),
        "blink" => Some(STYLE_BLINK),
        "reverse" => Some(STYLE_REVERSE),
        "hidden" => Some(STYLE_HIDDEN),
        "dim" => Some(STYLE_DIM),
        _ => None,
    }
}

fn resolve_key_text(parameter_text: &str, runtime_context: &LuaRuntimeContext) -> String {
    let mut parts = parameter_text.splitn(2, '>');
    let action = parts.next().unwrap_or_default().trim();
    let mode = parts.next().unwrap_or("user").trim();
    if action.is_empty() {
        return MISSING_KEY_TEXT.to_string();
    }

    match runtime_context.consumer {
        LuaRuntimeConsumer::GamePackage => resolve_game_key_text(action, mode, runtime_context),
        LuaRuntimeConsumer::OfficialUiPackage => resolve_ui_key_text(action, mode, runtime_context),
        LuaRuntimeConsumer::ScreensaverPackage | LuaRuntimeConsumer::BossPackage => {
            MISSING_KEY_TEXT.to_string()
        }
    }
}

fn resolve_game_key_text(action: &str, mode: &str, runtime_context: &LuaRuntimeContext) -> String {
    let Some(game_module) = runtime_context.current_game.as_ref() else {
        return MISSING_KEY_TEXT.to_string();
    };
    let Some(action_binding) = game_module.game.actions.get(action) else {
        return MISSING_KEY_TEXT.to_string();
    };

    let key_value = if mode == "original" {
        &action_binding.key
    } else {
        find_user_key(&runtime_context.keybinds, game_module.uid.as_str(), action)
            .unwrap_or(&action_binding.key)
    };
    format_key_display(&key_display::display_key_value(
        key_value,
        game_module.game.case_sensitive,
    ))
}

fn resolve_ui_key_text(action: &str, mode: &str, runtime_context: &LuaRuntimeContext) -> String {
    let Some(action_value) = runtime_context.current_ui_actions.get(action) else {
        return MISSING_KEY_TEXT.to_string();
    };
    let Some(key_value) = action_value.get("key") else {
        return MISSING_KEY_TEXT.to_string();
    };
    let key_value = if mode == "original" || mode == "user" {
        key_value
    } else {
        return MISSING_KEY_TEXT.to_string();
    };
    format_key_display(&key_display::display_key_value(key_value, false))
}

fn find_user_key<'a>(
    keybinds: &'a JsonValue,
    game_uid: &str,
    action: &str,
) -> Option<&'a JsonValue> {
    keybinds
        .get(keybind_profile::GAME_SECTION)
        .and_then(|game_keybinds| game_keybinds.get(game_uid))
        .and_then(|game_keybinds| game_keybinds.get(action))
        .and_then(|action_keybind| action_keybind.get("key_user"))
}

fn format_key_display(key_value: &JsonValue) -> String {
    match key_value {
        JsonValue::String(key) if !key.is_empty() => format!("[{key}]"),
        JsonValue::Array(keys) => {
            let labels = keys
                .iter()
                .filter_map(|key| match key {
                    JsonValue::String(key) if !key.is_empty() => Some(format!("[{key}]")),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if labels.is_empty() {
                MISSING_KEY_TEXT.to_string()
            } else {
                labels.join("/")
            }
        }
        _ => MISSING_KEY_TEXT.to_string(),
    }
}

fn emit_character(character: char, active_style: &ActiveStyle, output: &mut Vec<StyledCharacter>) {
    output.push(StyledCharacter {
        character,
        fg: active_style.fg.as_ref().map(|fg| fg.value.clone()),
        bg: active_style.bg.as_ref().map(|bg| bg.value.clone()),
        styles: active_style
            .styles
            .iter()
            .filter(|style| style.value != STYLE_NORMAL)
            .map(|style| style.value)
            .collect(),
        style_explicit: !active_style.styles.is_empty(),
    });
}

fn emit_escaped_or_literal_backslash(
    source_chars: &[char],
    index: &mut usize,
    active_style: &mut ActiveStyle,
    output: &mut Vec<StyledCharacter>,
) {
    *index += 1;
    let Some(next_character) = source_chars.get(*index).copied() else {
        emit_character('\\', active_style, output);
        active_style.tick();
        return;
    };

    match next_character {
        'n' => {
            emit_character('\n', active_style, output);
            active_style.tick();
            *index += 1;
        }
        '\\' | '{' | '}' | '|' => {
            emit_character(next_character, active_style, output);
            active_style.tick();
            *index += 1;
        }
        other => {
            emit_character('\\', active_style, output);
            active_style.tick();
            emit_character(other, active_style, output);
            active_style.tick();
            *index += 1;
        }
    }
}

impl ActiveStyle {
    fn tick(&mut self) {
        tick_timed_option(&mut self.fg);
        tick_timed_option(&mut self.bg);
        for style in &mut self.styles {
            if let Some(remaining_chars) = style.remaining_chars.as_mut() {
                *remaining_chars = remaining_chars.saturating_sub(1);
            }
        }
        self.styles.retain(|style| style.remaining_chars != Some(0));
    }
}

fn tick_timed_option<T>(value: &mut Option<TimedValue<T>>) {
    if let Some(value) = value {
        if let Some(remaining_chars) = value.remaining_chars.as_mut() {
            *remaining_chars = remaining_chars.saturating_sub(1);
        }
    }
    if value
        .as_ref()
        .is_some_and(|value| value.remaining_chars == Some(0))
    {
        *value = None;
    }
}

fn is_valid_color(color: &str) -> bool {
    if is_valid_hex_color(color) || is_valid_rgb_color(color) {
        return true;
    }
    if color
        .trim()
        .parse::<i64>()
        .is_ok_and(|value| (0..=15).contains(&value))
    {
        return true;
    }

    matches!(
        color,
        "black"
            | "red"
            | "green"
            | "yellow"
            | "blue"
            | "magenta"
            | "cyan"
            | "white"
            | "grey"
            | "gray"
            | "dark_red"
            | "dark_green"
            | "dark_yellow"
            | "dark_blue"
            | "dark_magenta"
            | "dark_cyan"
            | "dark_grey"
            | "dark_gray"
    )
}

fn is_valid_hex_color(color: &str) -> bool {
    let Some(hex) = color.strip_prefix('#') else {
        return false;
    };
    hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn is_valid_rgb_color(color: &str) -> bool {
    let Some(body) = color
        .strip_prefix("rgb(")
        .and_then(|body| body.strip_suffix(')'))
    else {
        return false;
    };
    let values = body
        .split(',')
        .map(|value| value.trim().parse::<u8>().ok())
        .collect::<Option<Vec<_>>>();
    values.is_some_and(|values| values.len() == 3)
}

fn emit_unknown_command(output: &mut Vec<StyledCharacter>) {
    let message = i18n::text().error.unknown_command;
    emit_error_text(message.as_str(), output);
}

fn emit_unknown_parameter(output: &mut Vec<StyledCharacter>) {
    let message = i18n::text().error.unknown_parameter;
    emit_error_text(message.as_str(), output);
}

fn emit_error_text(message: &str, output: &mut Vec<StyledCharacter>) {
    for character in message.chars() {
        output.push(StyledCharacter {
            character,
            fg: Some(ERROR_FG_COLOR.to_string()),
            bg: None,
            styles: Vec::new(),
            style_explicit: true,
        });
    }
}
