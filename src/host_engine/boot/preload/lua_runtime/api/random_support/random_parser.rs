//! 随机数 API 参数解析

use mlua::{Value, Variadic};

/// 整数随机调用参数。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RandomIntArgs {
    Default,
    Id(String),
    Max(i64),
    MaxWithId { max: i64, id: String },
    Range { min: i64, max: i64 },
    RangeWithId { min: i64, max: i64, id: String },
}

/// 解析 random 参数。
pub fn parse_random_int_args(args: &Variadic<Value>) -> mlua::Result<RandomIntArgs> {
    match args.len() {
        0 => Ok(RandomIntArgs::Default),
        1 => match args.first() {
            Some(Value::String(value)) => Ok(RandomIntArgs::Id(value.to_str()?.to_string())),
            Some(value) => Ok(RandomIntArgs::Max(value_to_i64(value)?)),
            None => unreachable!(),
        },
        2 => match args.get(1) {
            Some(Value::String(value)) => Ok(RandomIntArgs::MaxWithId {
                max: value_to_i64(&args[0])?,
                id: value.to_str()?.to_string(),
            }),
            Some(value) => Ok(RandomIntArgs::Range {
                min: value_to_i64(&args[0])?,
                max: value_to_i64(value)?,
            }),
            None => unreachable!(),
        },
        3 => Ok(RandomIntArgs::RangeWithId {
            min: value_to_i64(&args[0])?,
            max: value_to_i64(&args[1])?,
            id: value_to_string(&args[2])?,
        }),
        _ => Err(mlua::Error::external("random expects 0-3 arguments")),
    }
}

/// 解析 random_float 可选 id。
pub fn parse_random_float_id(args: &Variadic<Value>) -> mlua::Result<Option<String>> {
    match args.len() {
        0 => Ok(None),
        1 => Ok(Some(value_to_string(&args[0])?)),
        _ => Err(mlua::Error::external("random_float expects 0-1 arguments")),
    }
}

fn value_to_i64(value: &Value) -> mlua::Result<i64> {
    match value {
        Value::Integer(value) => Ok(*value),
        Value::Number(value) => Ok(*value as i64),
        _ => Err(mlua::Error::external("argument must be an integer")),
    }
}

fn value_to_string(value: &Value) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::external("argument must be a string")),
    }
}
