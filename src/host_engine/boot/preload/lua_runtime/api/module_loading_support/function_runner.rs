//! 辅助脚本运行

use std::fs;
use std::path::Path;

use mlua::{Lua, Table, Value};

use super::function_environment;

/// 读取并执行辅助脚本，返回脚本环境表。
pub fn run_function_script(lua: &Lua, script_path: &Path) -> mlua::Result<Table> {
    let source = fs::read_to_string(script_path)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())
        .map_err(|error| mlua::Error::external(format!("failed to read helper script: {error}")))?;
    let environment = function_environment::create_function_environment(lua)?;

    let result = lua
        .load(source.as_str())
        .set_name(script_path.to_string_lossy().as_ref())
        .set_environment(environment.clone())
        .eval::<Value>()?;

    function_environment::clean_function_environment(&environment)?;
    if let Value::Table(returned_table) = result {
        return Ok(returned_table);
    }

    Ok(environment)
}
