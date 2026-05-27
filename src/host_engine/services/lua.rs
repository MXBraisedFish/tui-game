// 引入mlua的lua脚本语言库
use mlua::Lua;

pub struct LuaService {
  lua: Lua
}

impl LuaService {
  pub fn new() -> Self {
    Self {
      lua: Lua::new()
    }
  }

  // 只读引用
  pub fn lua(&self) -> &Lua {
    &self.lua
  }

  // 执行Lua代码并返回结果
  pub fn eval(&self, code: &str) -> Result<String, String> {
    self.lua.load(code).eval::<String>().map_err(|error| error.to_string())
  }
}