use mlua::{Lua, LuaOptions, StdLib, Value};

/// Lua 沙箱服务，提供安全的脚本执行环境
pub struct LuaService {
  lua: Lua,
}

impl LuaService {
  pub fn new() -> Self {
    let lua = Lua::new_with(
      StdLib::MATH | StdLib::UTF8 | StdLib::STRING | StdLib::TABLE,
      LuaOptions::default(),
    )
    .expect("Failed to create sandboxed Lua VM");
    install_sandbox(&lua);
    Self { lua }
  }

  pub fn lua(&self) -> &Lua {
    &self.lua
  }

  /// 在沙箱化 Lua 虚拟机中执行代码并返回结果
  pub fn eval(&self, code: &str) -> Result<String, String> {
    self
      .lua
      .load(code)
      .eval::<String>()
      .map_err(|error| error.to_string())
  }
}

// 移除危险的 Lua 全局函数（文件操作、元表操作等），创建安全沙箱环境
fn install_sandbox(lua: &Lua) {
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
    let _ = globals.set(name, Value::Nil);
  }
}
