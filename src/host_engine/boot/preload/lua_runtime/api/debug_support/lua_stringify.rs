//! Lua 值字符串化

use mlua::Value;

/// 将任意 Lua 值转换为日志字符串。
pub fn stringify_value(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value
            .to_str()
            .map(|value| value.to_string())
            .unwrap_or_else(|_| "<invalid-string>".to_string()),
        Value::Table(_) => "<table>".to_string(),
        Value::Function(_) => "<function>".to_string(),
        Value::Thread(_) => "<thread>".to_string(),
        Value::UserData(_) => "<userdata>".to_string(),
        Value::LightUserData(_) => "<light-userdata>".to_string(),
        Value::Error(error) => error.to_string(),
        Value::Other(_) => "<other>".to_string(),
    }
}
