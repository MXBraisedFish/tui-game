//! CSV 序列化

use csv::WriterBuilder;
use serde_json::Value as JsonValue;

/// 将二维数组 JSON 值序列化为 CSV 字符串。
pub fn serialize_csv(value: &JsonValue) -> mlua::Result<String> {
    let rows = value
        .as_array()
        .ok_or_else(|| mlua::Error::external("csv table must be a two-dimensional array"))?;
    let mut writer = WriterBuilder::new().from_writer(Vec::new());
    for row in rows {
        let columns = row
            .as_array()
            .ok_or_else(|| mlua::Error::external("csv row must be an array"))?;
        let record = columns
            .iter()
            .map(json_scalar_to_string)
            .collect::<Vec<_>>();
        writer.write_record(record).map_err(mlua::Error::external)?;
    }
    let bytes = writer.into_inner().map_err(mlua::Error::external)?;
    String::from_utf8(bytes).map_err(mlua::Error::external)
}

fn json_scalar_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
