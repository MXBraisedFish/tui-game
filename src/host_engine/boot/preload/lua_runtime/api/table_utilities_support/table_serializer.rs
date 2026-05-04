//! 表序列化入口

use serde_json::Value as JsonValue;

use super::csv_serializer;
use super::xml_serializer;

/// 表序列化目标格式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableFormat {
    Json,
    Yaml,
    Toml,
    Csv,
    Xml,
}

/// 将 JSON 中间值序列化为目标格式字符串。
pub fn serialize_table(value: &JsonValue, table_format: TableFormat) -> mlua::Result<String> {
    match table_format {
        TableFormat::Json => serde_json::to_string(value).map_err(mlua::Error::external),
        TableFormat::Yaml => serde_yaml::to_string(value).map_err(mlua::Error::external),
        TableFormat::Toml => toml::to_string_pretty(value).map_err(mlua::Error::external),
        TableFormat::Csv => csv_serializer::serialize_csv(value),
        TableFormat::Xml => Ok(xml_serializer::serialize_xml(value)),
    }
}
