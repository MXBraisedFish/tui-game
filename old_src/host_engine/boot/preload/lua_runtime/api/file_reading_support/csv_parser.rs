//! CSV 解析

use csv::StringRecord;
use serde_json::Value as JsonValue;

/// 将 CSV 文本转换为二维数组 JSON 值。
pub fn parse_csv_to_json(text: &str) -> Result<JsonValue, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        rows.push(csv_record_to_json(record?));
    }
    Ok(JsonValue::Array(rows))
}

fn csv_record_to_json(record: StringRecord) -> JsonValue {
    JsonValue::Array(
        record
            .iter()
            .map(|field| JsonValue::String(field.to_string()))
            .collect(),
    )
}
