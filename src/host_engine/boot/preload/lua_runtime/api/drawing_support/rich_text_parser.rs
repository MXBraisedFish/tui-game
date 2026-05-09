//! 富文本解析器

use serde_json::Value as JsonValue;

use crate::host_engine::boot::preload::lua_runtime::api::debug_support::key_display;
use crate::host_engine::boot::preload::lua_runtime::{LuaRuntimeConsumer, LuaRuntimeContext};
use crate::host_engine::boot::preload::persistent_data::keybind_profile;

use super::drawing_parser::{
    STYLE_BLINK, STYLE_BOLD, STYLE_DIM, STYLE_HIDDEN, STYLE_ITALIC, STYLE_REVERSE, STYLE_STRIKE,
    STYLE_UNDERLINE,
};

const RICH_TEXT_PREFIX: &str = "f%";
const MISSING_KEY_TEXT: &str = "[None]";

/// 带样式的单个终端字符。
#[derive(Clone, Debug)]
pub struct StyledCharacter {
    pub character: char,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub styles: Vec<i64>,
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
    let mut source_chars = source.chars().peekable();

    while let Some(character) = source_chars.next() {
        match character {
            '\\' => {
                let escaped_character = source_chars.next().unwrap_or('\\');
                emit_character(escaped_char(escaped_character), &active_style, &mut output);
                active_style.tick();
            }
            '{' => {
                let command_text = read_command_text(&mut source_chars)?;
                if command_text.trim().is_empty() {
                    return Err(mlua::Error::external("rich text command is empty"));
                }
                apply_command_block(
                    command_text.as_str(),
                    runtime_context,
                    &mut active_style,
                    &mut output,
                )?;
            }
            '}' => return Err(mlua::Error::external("rich text command is invalid")),
            _ => {
                emit_character(character, &active_style, &mut output);
                active_style.tick();
            }
        }
    }

    if active_style.has_unclosed_style() {
        return Err(mlua::Error::external("rich text style is not closed"));
    }

    Ok(output)
}

fn read_command_text(
    source_chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> mlua::Result<String> {
    let mut command_text = String::new();
    while let Some(character) = source_chars.next() {
        match character {
            '\\' => {
                command_text.push(character);
                if let Some(escaped_character) = source_chars.next() {
                    command_text.push(escaped_character);
                }
            }
            '}' => return Ok(command_text),
            '{' => return Err(mlua::Error::external("rich text command is invalid")),
            _ => command_text.push(character),
        }
    }
    Err(mlua::Error::external("rich text command is not closed"))
}

fn apply_command_block(
    command_text: &str,
    runtime_context: &LuaRuntimeContext,
    active_style: &mut ActiveStyle,
    output: &mut Vec<StyledCharacter>,
) -> mlua::Result<()> {
    for command in split_command_block(command_text) {
        let command = command.trim();
        if command.is_empty() {
            return Err(mlua::Error::external("rich text command is empty"));
        }
        apply_single_command(command, runtime_context, active_style, output)?;
    }
    Ok(())
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
) -> mlua::Result<()> {
    let Some((command_name, parameter_text)) = command.split_once(':') else {
        return Err(mlua::Error::external("rich text command is invalid"));
    };

    match command_name.trim() {
        "tc" => apply_color_command(parameter_text, &mut active_style.fg),
        "bg" => apply_color_command(parameter_text, &mut active_style.bg),
        "ts" => apply_text_style_command(parameter_text, active_style),
        "key" => {
            let key_text = resolve_key_text(parameter_text, runtime_context);
            for character in key_text.chars() {
                emit_character(character, active_style, output);
                active_style.tick();
            }
            Ok(())
        }
        _ => Err(mlua::Error::external("rich text command is invalid")),
    }
}

fn apply_color_command(
    parameter_text: &str,
    target: &mut Option<TimedValue<String>>,
) -> mlua::Result<()> {
    let (value, remaining_chars) = parse_value_and_count(parameter_text)?;
    if value == "clear" {
        *target = None;
        return Ok(());
    }
    *target = Some(TimedValue {
        value,
        remaining_chars,
    });
    Ok(())
}

fn apply_text_style_command(
    parameter_text: &str,
    active_style: &mut ActiveStyle,
) -> mlua::Result<()> {
    let (style_text, remaining_chars) = parse_value_and_count(parameter_text)?;
    if style_text == "clear" {
        active_style.styles.clear();
        return Ok(());
    }

    for style_name in style_text.split('+') {
        let style = parse_style_name(style_name.trim())?;
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
    Ok(())
}

fn parse_value_and_count(parameter_text: &str) -> mlua::Result<(String, Option<usize>)> {
    let mut parts = parameter_text.splitn(2, '>');
    let value = parts.next().unwrap_or_default().trim();
    if value.is_empty() {
        return Err(mlua::Error::external("rich text parameter is invalid"));
    }
    let count = parts
        .next()
        .map(|count_text| {
            let count = count_text
                .trim()
                .parse::<usize>()
                .map_err(|_| mlua::Error::external("rich text parameter count is invalid"))?;
            if count == 0 {
                return Err(mlua::Error::external(
                    "rich text parameter count is invalid",
                ));
            }
            Ok(count)
        })
        .transpose()?;

    Ok((value.to_string(), count))
}

fn parse_style_name(style_name: &str) -> mlua::Result<i64> {
    match style_name {
        "bold" => Ok(STYLE_BOLD),
        "italic" => Ok(STYLE_ITALIC),
        "underline" => Ok(STYLE_UNDERLINE),
        "strike" => Ok(STYLE_STRIKE),
        "blink" => Ok(STYLE_BLINK),
        "reverse" => Ok(STYLE_REVERSE),
        "hidden" => Ok(STYLE_HIDDEN),
        "dim" => Ok(STYLE_DIM),
        _ => Err(mlua::Error::external("rich text parameter is invalid")),
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
            .map(|style| style.value)
            .collect(),
    });
}

fn escaped_char(character: char) -> char {
    match character {
        'n' => '\n',
        other => other,
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

    fn has_unclosed_style(&self) -> bool {
        self.fg
            .as_ref()
            .is_some_and(|value| value.remaining_chars.is_none())
            || self
                .bg
                .as_ref()
                .is_some_and(|value| value.remaining_chars.is_none())
            || self
                .styles
                .iter()
                .any(|style| style.remaining_chars.is_none())
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
