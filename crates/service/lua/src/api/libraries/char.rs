use super::*;

pub(super) fn char_lib(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, chars) in [
    (
      "LINE",
      [
        "─", "┌", "│", "└", "─", "┘", "│", "┐", "├", "┴", "┤", "┬", "┼",
      ],
    ),
    (
      "BOLD_LINE",
      [
        "━", "┏", "┃", "┗", "━", "┛", "┃", "┓", "┣", "┻", "┫", "┳", "╋",
      ],
    ),
    (
      "DOUBLE_LINE",
      [
        "═", "╔", "║", "╚", "═", "╝", "║", "╗", "╠", "╩", "╣", "╦", "╬",
      ],
    ),
    (
      "ROUNDED_LINE",
      [
        "─", "╭", "│", "╰", "─", "╯", "│", "╮", "├", "┴", "┤", "┬", "┼",
      ],
    ),
  ] {
    let table = lua.create_table()?;
    for (key, value) in [
      "top",
      "left_top",
      "left",
      "left_bottom",
      "bottom",
      "right_bottom",
      "right",
      "right_top",
      "t_left",
      "t_bottom",
      "t_right",
      "t_top",
      "center",
    ]
    .into_iter()
    .zip(chars)
    {
      table.raw_set(key, value)?;
    }
    source.raw_set(name, readonly::proxy(lua, table)?)?;
  }
  let number = (b'0'..=b'9')
    .map(|v| Value::String(lua.create_string([v]).unwrap()))
    .collect::<Vec<_>>();
  let lower = (b'a'..=b'z')
    .map(|v| Value::String(lua.create_string([v]).unwrap()))
    .collect::<Vec<_>>();
  let upper = (b'A'..=b'Z')
    .map(|v| Value::String(lua.create_string([v]).unwrap()))
    .collect::<Vec<_>>();
  let symbols = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
    .chars()
    .map(|v| Value::String(lua.create_string(v.to_string()).unwrap()))
    .collect::<Vec<_>>();
  source.raw_set("ASCII_NUMBER", readonly::array(lua, number.clone())?)?;
  source.raw_set("ASCII_LOWERCASE", readonly::array(lua, lower.clone())?)?;
  source.raw_set("ASCII_UPPERCASE", readonly::array(lua, upper.clone())?)?;
  source.raw_set(
    "ASCII_LETTER",
    readonly::array(lua, lower.iter().chain(&upper).cloned())?,
  )?;
  source.raw_set("ASCII_CHARACTER", readonly::array(lua, symbols.clone())?)?;
  source.raw_set(
    "ASCII",
    readonly::array(
      lua,
      number.into_iter().chain(lower).chain(upper).chain(symbols),
    )?,
  )?;
  readonly::proxy(lua, source)
}
