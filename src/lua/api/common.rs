/// Lua API 公共工具函数，参数校验与错误生成
/// 业务逻辑：
/// 参数数量校验
/// 参数类型校验
/// 错误消息生成
/// 类型名显示

use mlua::{Table, Value};

use crate::app::i18n;
use crate::utils::host_log;

pub(crate) fn expect_exact_arg_count(args: &[Value], expected: usize) -> mlua::Result<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(arg_count_error(&expected.to_string(), args.len()))
    }
}

pub(crate) fn expect_arg_count_range(args: &[Value], min: usize, max: usize) -> mlua::Result<()> {
    if (min..=max).contains(&args.len()) {
        Ok(())
    } else if min == max {
        Err(arg_count_error(&min.to_string(), args.len()))
    } else {
        Err(arg_count_error(&format!("{min}-{max}"), args.len()))
    }
}

pub(crate) fn expect_string_arg(args: &[Value], index: usize, arg_name: &str) -> mlua::Result<String> {
    match args.get(index) {
        Some(Value::String(value)) => Ok(value.to_str().map(|v| v.to_string()).unwrap_or_default()),
        Some(value) => Err(arg_type_error(arg_name, "string", value)),
        None => Err(arg_count_error(&(index + 1).to_string(), args.len())),
    }
}

pub(crate) fn expect_optional_string_arg(
    args: &[Value],
    index: usize,
    arg_name: &str,
) -> mlua::Result<Option<String>> {
    match args.get(index) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.to_str().map(|v| v.to_string()).unwrap_or_default())),
        Some(value) => Err(arg_type_error(arg_name, "string", value)),
    }
}

pub(crate) fn expect_i64_arg(args: &[Value], index: usize, arg_name: &str) -> mlua::Result<i64> {
    match args.get(index) {
        Some(Value::Integer(value)) => Ok(*value),
        Some(Value::Number(value)) => Ok(*value as i64),
        Some(value) => Err(arg_type_error(arg_name, "number", value)),
        None => Err(arg_count_error(&(index + 1).to_string(), args.len())),
    }
}

pub(crate) fn expect_optional_i64_arg(
    args: &[Value],
    index: usize,
    arg_name: &str,
) -> mlua::Result<Option<i64>> {
    match args.get(index) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Integer(value)) => Ok(Some(*value)),
        Some(Value::Number(value)) => Ok(Some(*value as i64)),
        Some(value) => Err(arg_type_error(arg_name, "number", value)),
    }
}

pub(crate) fn expect_table_arg(args: &[Value], index: usize, arg_name: &str) -> mlua::Result<Table> {
    match args.get(index) {
        Some(Value::Table(value)) => Ok(value.clone()),
        Some(value) => Err(arg_type_error(arg_name, "table", value)),
        None => Err(arg_count_error(&(index + 1).to_string(), args.len())),
    }
}

pub(crate) fn expect_optional_table_arg(
    args: &[Value],
    index: usize,
    arg_name: &str,
) -> mlua::Result<Option<Table>> {
    match args.get(index) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Table(value)) => Ok(Some(value.clone())),
        Some(value) => Err(arg_type_error(arg_name, "table", value)),
    }
}

pub(crate) fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "lightuserdata",
        Value::Integer(_) | Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        Value::Error(_) => "error",
        Value::Other(_) => "other",
    }
}

pub(crate) fn arg_count_error(expected: &str, actual: usize) -> mlua::Error {
    let actual_text = actual.to_string();
    host_log::append_host_error(
        "host.exception.api_arg_count_mismatch",
        &[("expected", expected), ("actual", &actual_text)],
    );
    mlua::Error::external(
        i18n::t_or(
            "host.exception.api_arg_count_mismatch",
            "API argument count mismatch: expected {expected} arguments, got {actual}",
        )
        .replace("{expected}", expected)
        .replace("{actual}", &actual_text),
    )
}

pub(crate) fn arg_type_error(
    arg_name: &str,
    expected_type: &str,
    actual_value: &Value,
) -> mlua::Error {
    let actual_type = value_type_name(actual_value);
    host_log::append_host_error(
        "host.exception.api_arg_type_error",
        &[
            ("arg_name", arg_name),
            ("expected_type", expected_type),
            ("actual_type", actual_type),
        ],
    );
    mlua::Error::external(
        i18n::t_or(
            "host.exception.api_arg_type_error",
            "API argument '{arg_name}' type error: expected {expected_type}, got {actual_type}",
        )
        .replace("{arg_name}", arg_name)
        .replace("{expected_type}", expected_type)
        .replace("{actual_type}", actual_type),
    )
}
