use mlua::Lua;

/// 新运行时沙箱安装入口。
pub fn install_sandbox(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let os: mlua::Table = globals.get("os")?;
    let _ = os.set("execute", mlua::Value::Nil);
    let _ = os.set("remove", mlua::Value::Nil);
    let _ = os.set("rename", mlua::Value::Nil);
    let _ = os.set("exit", mlua::Value::Nil);
    let _ = globals.set("io", mlua::Value::Nil);
    let _ = globals.set("debug", mlua::Value::Nil);
    Ok(())
}
