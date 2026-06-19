use mlua::{Lua, LuaOptions, StdLib, Value};

/// 安全沙箱 Lua 虚拟机。
/// 仅加载 math/string/table/utf8 四个安全库，禁用危险全局函数。
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

  pub fn eval(&self, code: &str) -> Result<String, String> {
    self
      .lua
      .load(code)
      .eval::<String>()
      .map_err(|error| error.to_string())
  }
}

/// 置空危险全局函数：文件加载、调试、元表操作、GC 控制。
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
