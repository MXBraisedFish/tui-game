//! API 参数基础校验

use mlua::{Value, Variadic};

/// 精确参数数量校验。
pub fn expect_exact_arg_count(args: &Variadic<Value>, expected: usize) -> mlua::Result<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "argument count mismatch: expected {expected}, got {}",
            args.len()
        )))
    }
}

/// 参数数量范围校验。
pub fn expect_arg_count_range(
    args: &Variadic<Value>,
    min_count: usize,
    max_count: usize,
) -> mlua::Result<()> {
    if (min_count..=max_count).contains(&args.len()) {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "argument count mismatch: expected {min_count}-{max_count}, got {}",
            args.len()
        )))
    }
}

/// 读取 string 参数。
pub fn expect_string_arg(args: &Variadic<Value>, index: usize) -> mlua::Result<String> {
    match args.get(index) {
        Some(Value::String(value)) => Ok(value.to_str()?.to_string()),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected string, got {}",
            lua_type_name(value)
        ))),
        None => Err(mlua::Error::external("argument missing")),
    }
}

/// 读取整数参数。
pub fn expect_i64_arg(args: &Variadic<Value>, index: usize) -> mlua::Result<i64> {
    match args.get(index) {
        Some(Value::Integer(value)) => Ok(*value),
        Some(Value::Number(value)) => Ok(*value as i64),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected integer, got {}",
            lua_type_name(value)
        ))),
        None => Err(mlua::Error::external("argument missing")),
    }
}

/// 读取可选整数参数。
pub fn expect_optional_i64_arg(args: &Variadic<Value>, index: usize) -> mlua::Result<Option<i64>> {
    match args.get(index) {
        Some(Value::Nil) | None => Ok(None),
        Some(Value::Integer(value)) => Ok(Some(*value)),
        Some(Value::Number(value)) => Ok(Some(*value as i64)),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected integer, got {}",
            lua_type_name(value)
        ))),
    }
}

/// 读取可选 string 参数。
pub fn expect_optional_string_arg(
    args: &Variadic<Value>,
    index: usize,
) -> mlua::Result<Option<String>> {
    match args.get(index) {
        Some(Value::Nil) | None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.to_str()?.to_string())),
        Some(value) => Err(mlua::Error::external(format!(
            "argument type mismatch: expected string, got {}",
            lua_type_name(value)
        ))),
    }
}

fn lua_type_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "light_userdata",
        Value::Integer(_) => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        Value::Error(_) => "error",
        Value::Other(_) => "other",
    }
}
