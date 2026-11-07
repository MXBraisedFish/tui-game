//! Lua color library bindings with validated arguments and session-owned host access.

use super::*;

/// Build and register the Lua color API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn color(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("BLACK", "black"),
    ("RED", "red"),
    ("GREEN", "green"),
    ("YELLOW", "yellow"),
    ("BLUE", "blue"),
    ("MAGENTA", "magenta"),
    ("CYAN", "cyan"),
    ("GRAY", "gray"),
    ("GREY", "gray"),
    ("BRIGHT_GRAY", "bright_gray"),
    ("BRIGHT_GREY", "bright_gray"),
    ("BRIGHT_RED", "bright_red"),
    ("BRIGHT_GREEN", "bright_green"),
    ("BRIGHT_YELLOW", "bright_yellow"),
    ("BRIGHT_BLUE", "bright_blue"),
    ("BRIGHT_MAGENTA", "bright_magenta"),
    ("BRIGHT_CYAN", "bright_cyan"),
    ("WHITE", "white"),
    ("NONE", "none"),
    ("TRANSPARENT", "transparent"),
  ] {
    source.raw_set(name, value)?;
  }
  for (name, hex) in [("rgb", false), ("hex", true)] {
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = if hex { "color.hex" } else { "color.rgb" };
        let parsed = args::positional(lua, method, values, &["red", "green", "blue"], &[])?;
        let channel = |index, name| -> mlua::Result<u8> {
          let value = args::integer(parsed.required(index, method, name)?, method, name)?;
          u8::try_from(value)
            .map_err(|_| args::message(method, format!("{name} must be in 0..=255")))
        };
        let red = channel(0, "red")?;
        let green = channel(1, "green")?;
        let blue = channel(2, "blue")?;
        Ok(if hex {
          format!("#{red:02x}{green:02x}{blue:02x}")
        } else {
          format!("rgb({red},{green},{blue})")
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
