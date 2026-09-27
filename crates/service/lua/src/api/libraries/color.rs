use super::*;

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
      lua.create_function(move |_, values: MultiValue| {
        let method = if hex { "color.hex" } else { "color.rgb" };
        let table = args::named(method, values, &["r", "g", "b"])?;
        let channel = |n| -> mlua::Result<u8> {
          let v = args::integer(args::required(&table, method, n)?, method, n)?;
          u8::try_from(v).map_err(|_| args::message(method, format!("{n} must be in 0..=255")))
        };
        let (r, g, b) = (channel("r")?, channel("g")?, channel("b")?);
        Ok(if hex {
          format!("#{r:02x}{g:02x}{b:02x}")
        } else {
          format!("rgb({r},{g},{b})")
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
