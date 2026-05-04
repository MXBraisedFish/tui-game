//! 结构化文本解析

use serde_json::Value as JsonValue;

use super::csv_parser;
use super::xml_parser;

/// 数据格式类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuredFormat {
    Json,
    Xml,
    Yaml,
    Toml,
    Csv,
}

/// 将结构化文本解析为 JSON 中间值。
pub fn parse_structured_text(text: &str, file_format: StructuredFormat) -> mlua::Result<JsonValue> {
    match file_format {
        StructuredFormat::Json => serde_json::from_str::<JsonValue>(text)
            .map_err(|error| mlua::Error::external(format!("invalid json format: {error}"))),
        StructuredFormat::Xml => xml_parser::parse_xml_to_json(text)
            .map_err(|_| mlua::Error::external("invalid xml format")),
        StructuredFormat::Yaml => {
            let value = serde_yaml::from_str::<serde_yaml::Value>(text)
                .map_err(|error| mlua::Error::external(format!("invalid yaml format: {error}")))?;
            serde_json::to_value(value)
                .map_err(|error| mlua::Error::external(format!("invalid yaml format: {error}")))
        }
        StructuredFormat::Toml => {
            let value = text
                .parse::<toml::Value>()
                .map_err(|error| mlua::Error::external(format!("invalid toml format: {error}")))?;
            serde_json::to_value(value)
                .map_err(|error| mlua::Error::external(format!("invalid toml format: {error}")))
        }
        StructuredFormat::Csv => csv_parser::parse_csv_to_json(text)
            .map_err(|error| mlua::Error::external(format!("invalid csv format: {error}"))),
    }
}
