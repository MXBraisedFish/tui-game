//! Strict bindings for session-owned editable input widgets.

use super::*;
use crate::object_pool::LuaInputBinding;
use tg_core_style::TextStyle;
use tg_service_widget::{TextInputCursorShape, TextInputId, TextInputMode, VerticalAlign};

const MAX_INPUTS: usize = 1024;
const OPTION_FIELDS: &[&str] = &[
  "width",
  "height",
  "fg",
  "bg",
  "vertical_align",
  "horizontal_align",
  "text",
  "max_chars",
  "line_mode",
  "mouse",
  "enter_submit",
  "placeholder",
  "placeholder_fg",
  "cursor_shape",
  "cursor_blink",
  "bold",
  "italic",
  "underline",
  "strike",
  "blink",
  "reverse",
  "hidden",
  "dim",
];

/// Build the read-only input library and its constants.
///
/// # Errors
///
/// Propagate Lua allocation and function registration errors.
pub(super) fn input(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("SINGLELINE", "singleline"),
    ("MULTILINE", "multiline"),
    ("BLOCK", "█"),
    ("UNDERLINE", "_"),
    ("LINE", "▏"),
  ] {
    source.raw_set(name, value)?;
  }
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.input.create";
      let table = positional_table(
        lua,
        method,
        values,
        &["width", "height"],
        &OPTION_FIELDS[2..],
      )?;
      let mut binding = options(&table, method, LuaInputBinding::default())?;
      if matches!(table.raw_get::<Value>("enter_submit")?, Value::Nil) {
        binding.enter_submit = binding.options.mode == TextInputMode::SingleLine;
      }
      with_pool_mut(&create_state, method, |pool| {
        if pool.inputs.len() >= MAX_INPUTS {
          return Err(args::message(method, "input limit of 1024 was reached"));
        }
        let id = pool
          .text_input
          .create(&mut pool.ui, binding.options.clone());
        pool.text_input.configure(
          &mut pool.ui,
          id,
          binding.options.clone(),
          binding.enter_submit,
        );
        pool.inputs.insert(id.0, binding);
        Ok(format_id(id.0))
      })
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.input.set";
      let table = positional_table(lua, method, values, &["id"], OPTION_FIELDS)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(mut binding) = pool.inputs.get(&id).cloned() else {
          return Ok(false);
        };
        binding.options.initial_text = pool
          .text_input
          .get_text(&pool.ui, TextInputId(id))
          .unwrap_or_default()
          .into();
        let binding = options(&table, method, binding)?;
        pool.text_input.configure(
          &mut pool.ui,
          TextInputId(id),
          binding.options.clone(),
          binding.enter_submit,
        );
        pool.inputs.insert(id, binding);
        Ok(true)
      })
    })?,
  )?;
  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.input.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let coordinate = |name| {
        let value = args::integer(args::required(&table, method, name)?, method, name)?;
        i32::try_from(value)
          .map_err(|_| args::message(method, format!("{name} is out of i32 range")))
      };
      let command = LuaDrawCommand::Input {
        id,
        target: parse_draw_target(&table, method, &draw_state)?,
        x: coordinate("x")?,
        y: coordinate("y")?,
      };
      with_pool(&draw_state, method, |pool| {
        if !pool.inputs.contains_key(&id) {
          return Err(args::message(method, "unknown input id"));
        }
        Ok(())
      })?;
      draw::enqueue_draw(&draw_state, method, command)
    })?,
  )?;
  for (name, method) in [
    ("delete", "widget.input.delete"),
    ("exists", "widget.input.exists"),
    ("get_info", "widget.input.get_info"),
    ("focus", "widget.input.focus"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let table = positional_table(lua, method, values, &["id"], &[])?;
        let id = table_id(&table, method)?;
        let result = with_pool_mut(&state, method, |pool| match name {
          "delete" => Ok(Value::Boolean(pool.delete_input(id))),
          "exists" => Ok(Value::Boolean(pool.inputs.contains_key(&id))),
          "focus" => {
            if !pool.inputs.contains_key(&id) {
              return Ok(Value::Boolean(false));
            }
            if pool.text_input.is_focused(&pool.ui, TextInputId(id)) {
              return Ok(Value::Boolean(true));
            }
            pool.text_input.blur(&mut pool.ui);
            Ok(Value::Boolean(
              pool.text_input.focus(&mut pool.ui, TextInputId(id)),
            ))
          }
          _ => pool
            .inputs
            .get(&id)
            .map(|binding| {
              info(
                lua,
                id,
                binding,
                pool
                  .text_input
                  .get_text(&pool.ui, TextInputId(id))
                  .unwrap_or_default(),
              )
              .map(Value::Table)
            })
            .unwrap_or(Ok(Value::Nil)),
        })?;
        if name == "focus" && result == Value::Boolean(true) {
          let mut api = state.borrow_mut();
          api.input.close(true, true);
          api.input.close_text();
        }
        Ok(result)
      })?,
    )?;
  }
  for (name, method) in [
    ("clear", "widget.input.clear"),
    ("count", "widget.input.count"),
    ("list", "widget.input.list"),
    ("blur", "widget.input.blur"),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        args::no_args(method, values)?;
        with_pool_mut(&state, method, |pool| match name {
          "blur" => Ok(Value::Boolean(pool.text_input.blur(&mut pool.ui))),
          "clear" => {
            for id in pool.inputs.keys().copied().collect::<Vec<_>>() {
              pool.delete_input(id);
            }
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.inputs.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.inputs.iter().enumerate() {
              result.raw_set(
                index + 1,
                info(
                  lua,
                  *id,
                  binding,
                  pool
                    .text_input
                    .get_text(&pool.ui, TextInputId(*id))
                    .unwrap_or_default(),
                )?,
              )?;
            }
            result.raw_set("n", pool.inputs.len())?;
            Ok(Value::Table(result))
          }
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}

fn options(
  table: &Table,
  method: &str,
  mut binding: LuaInputBinding,
) -> mlua::Result<LuaInputBinding> {
  for (name, dimension) in [
    ("width", &mut binding.params.rect.width),
    ("height", &mut binding.params.rect.height),
  ] {
    if !matches!(table.raw_get::<Value>(name)?, Value::Nil) {
      *dimension = positive_u16(table, method, name)?;
    }
  }
  for (name, stored, parsed, background) in [
    ("fg", &mut binding.fg, &mut binding.params.fg, false),
    ("bg", &mut binding.bg, &mut binding.params.bg, true),
    (
      "placeholder_fg",
      &mut binding.placeholder_fg,
      &mut binding.params.placeholder_fg,
      false,
    ),
  ] {
    let value = table.raw_get::<Value>(name)?;
    if !matches!(value, Value::Nil) {
      *parsed = parse_color(value.clone(), method, name, background)?;
      *stored = args::string(value, method, name)?;
    }
  }
  binding.vertical_align = args::optional_string(
    table,
    method,
    "vertical_align",
    Some(&binding.vertical_align),
  )?
  .unwrap();
  binding.params.vertical_align = match binding.vertical_align.as_str() {
    "auto" | "top" => VerticalAlign::Top,
    "center" | "vertical_center" => VerticalAlign::Center,
    "bottom" => VerticalAlign::Bottom,
    _ => return Err(args::message(method, "invalid vertical_align constant")),
  };
  binding.horizontal_align = args::optional_string(
    table,
    method,
    "horizontal_align",
    Some(&binding.horizontal_align),
  )?
  .unwrap();
  binding.params.text_align = match binding.horizontal_align.as_str() {
    "auto" | "left" => TextAlign::Left,
    "center" | "horizontal_center" => TextAlign::Center,
    "right" => TextAlign::Right,
    _ => return Err(args::message(method, "invalid horizontal_align constant")),
  };
  binding.options.initial_text =
    args::optional_string(table, method, "text", Some(&binding.options.initial_text))?.unwrap();
  binding.params.placeholder = args::optional_string(
    table,
    method,
    "placeholder",
    Some(&binding.params.placeholder),
  )?
  .unwrap();
  match table.raw_get::<Value>("max_chars")? {
    Value::Nil => {}
    Value::Boolean(false) => binding.options.max_chars = None,
    _ => binding.options.max_chars = Some(positive_u16(table, method, "max_chars")? as usize),
  }
  if let Some(mode) = args::optional_string(table, method, "line_mode", None)? {
    binding.options.mode = match mode.as_str() {
      "singleline" => TextInputMode::SingleLine,
      "multiline" => TextInputMode::MultiLine,
      _ => return Err(args::message(method, "invalid line_mode constant")),
    };
  }
  binding.options.mouse = args::optional_bool(table, method, "mouse", binding.options.mouse)?;
  binding.enter_submit = args::optional_bool(table, method, "enter_submit", binding.enter_submit)?;
  binding.params.cursor_blink =
    args::optional_bool(table, method, "cursor_blink", binding.params.cursor_blink)?;
  match table.raw_get::<Value>("cursor_shape")? {
    Value::Nil => {}
    Value::Boolean(false) => {
      binding.cursor_shape = None;
      binding.params.cursor_character = None;
      binding.params.cursor_shape = Some(TextInputCursorShape::None);
    }
    value => {
      let marker = args::string(value, method, "cursor_shape")?;
      if marker.chars().any(char::is_control)
        || tg_core_unicode::graphemes(&marker).len() != 1
        || tg_core_unicode::display_width(&marker) != 1
      {
        return Err(args::message(
          method,
          "cursor_shape must be one visible terminal-cell grapheme or false",
        ));
      }
      binding.params.cursor_shape = Some(TextInputCursorShape::Block);
      binding.params.cursor_character = Some(marker.clone());
      binding.cursor_shape = Some(marker);
    }
  }
  let style = &mut binding.params.text_style;
  style.bold = args::optional_bool(table, method, "bold", style.bold)?;
  style.italic = args::optional_bool(table, method, "italic", style.italic)?;
  style.underline = args::optional_bool(table, method, "underline", style.underline)?;
  style.strike = args::optional_bool(table, method, "strike", style.strike)?;
  style.blink = args::optional_bool(table, method, "blink", style.blink)?;
  style.reverse = args::optional_bool(table, method, "reverse", style.reverse)?;
  style.hidden = args::optional_bool(table, method, "hidden", style.hidden)?;
  style.dim = args::optional_bool(table, method, "dim", style.dim)?;
  binding.params.placeholder_style = style.clone();
  binding.params.cursor_style = TextStyle {
    foreground: binding.params.fg.clone(),
    ..style.clone()
  };
  Ok(binding)
}

fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  args::string(args::required(table, method, "id")?, method, "id")?
    .strip_prefix("input_")
    .and_then(|value| value.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid input id"))
}
fn format_id(id: u64) -> String {
  format!("input_{id:03}")
}
fn info(lua: &Lua, id: u64, binding: &LuaInputBinding, text: &str) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  result.raw_set("id", format_id(id))?;
  result.raw_set("width", binding.params.rect.width)?;
  result.raw_set("height", binding.params.rect.height)?;
  result.raw_set("fg", binding.fg.as_str())?;
  result.raw_set("bg", binding.bg.as_str())?;
  result.raw_set("placeholder_fg", binding.placeholder_fg.as_str())?;
  result.raw_set("vertical_align", binding.vertical_align.as_str())?;
  result.raw_set("horizontal_align", binding.horizontal_align.as_str())?;
  result.raw_set("text", text)?;
  result.raw_set("max_chars", binding.options.max_chars)?;
  result.raw_set(
    "line_mode",
    if binding.options.mode == TextInputMode::SingleLine {
      "singleline"
    } else {
      "multiline"
    },
  )?;
  result.raw_set("mouse", binding.options.mouse)?;
  result.raw_set("enter_submit", binding.enter_submit)?;
  result.raw_set("placeholder", binding.params.placeholder.as_str())?;
  result.raw_set("cursor_shape", binding.cursor_shape.as_deref())?;
  result.raw_set("cursor_blink", binding.params.cursor_blink)?;
  let style = &binding.params.text_style;
  for (name, enabled) in [
    ("bold", style.bold),
    ("italic", style.italic),
    ("underline", style.underline),
    ("strike", style.strike),
    ("blink", style.blink),
    ("reverse", style.reverse),
    ("hidden", style.hidden),
    ("dim", style.dim),
  ] {
    result.raw_set(name, enabled)?;
  }
  Ok(result)
}
