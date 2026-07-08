use mlua::{Lua, LuaOptions, StdLib, Value};

use super::{LogService, LogSource};

/// Lua 沙箱服务，提供安全的脚本执行环境
pub struct LuaService {
  lua: Lua,
}

impl LuaService {
  pub fn new(log: &mut LogService) -> Self {
    let lua = Lua::new_with(
      StdLib::MATH | StdLib::UTF8 | StdLib::STRING | StdLib::TABLE,
      LuaOptions::default(),
    )
    .expect("Failed to create sandboxed Lua VM");
    install_sandbox(&lua, log);
    Self { lua }
  }

  pub fn lua(&self) -> &Lua {
    &self.lua
  }

  /// 在沙箱化 Lua 虚拟机中执行代码并返回结果
  pub fn eval(&self, code: &str, log: &mut LogService) -> Result<String, String> {
    match self.lua.load(code).eval::<mlua::Value>() {
      Ok(val) => Ok(format!("{val:?}")),
      Err(error) => {
        log.warn(
          LogSource::Lua,
          format!("Lua eval error: {err}", err = error),
        );
        Err(error.to_string())
      }
    }
  }
}

// 移除危险的 Lua 全局函数（文件操作、元表操作等），创建安全沙箱环境
fn install_sandbox(lua: &Lua, log: &mut LogService) {
  let globals = lua.globals();
  for name in [
    "dofile",
    "loadfile",
    "load",
    "loadstring",
    "collectgarbage",
    "rawget",
    "rawset",
    "rawequal",
    "rawlen",
    "getmetatable",
    "setmetatable",
  ] {
    if let Err(error) = globals.set(name, Value::Nil) {
      log.warn(
        LogSource::Lua,
        format!("Failed to sandbox global '{}': {err}", name, err = error),
      );
    }
  }
}
