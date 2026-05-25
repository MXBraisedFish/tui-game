//! 边框字符配置

use mlua::{Table, Value};

/// 边框八方向字符。
#[derive(Clone, Debug, Default)]
pub struct BorderChars {
    pub top: Option<char>,
    pub top_right: Option<char>,
    pub right: Option<char>,
    pub bottom_right: Option<char>,
    pub bottom: Option<char>,
    pub bottom_left: Option<char>,
    pub left: Option<char>,
    pub top_left: Option<char>,
}

impl BorderChars {
    /// 从 Lua 表读取边框字符。
    pub fn from_lua_table(table: &Table) -> mlua::Result<Self> {
        Ok(Self {
            top: optional_char(table.get::<Value>("top")?),
            top_right: optional_char(table.get::<Value>("top_right")?),
            right: optional_char(table.get::<Value>("right")?),
            bottom_right: optional_char(table.get::<Value>("bottom_right")?),
            bottom: optional_char(table.get::<Value>("bottom")?),
            bottom_left: optional_char(table.get::<Value>("bottom_left")?),
            left: optional_char(table.get::<Value>("left")?),
            top_left: optional_char(table.get::<Value>("top_left")?),
        })
    }
}

fn optional_char(value: Value) -> Option<char> {
    match value {
        Value::String(value) => value.to_str().ok().and_then(|value| value.chars().next()),
        _ => None,
    }
}
