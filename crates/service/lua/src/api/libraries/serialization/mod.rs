mod binary;
mod ini;
mod value;
mod xml;

use mlua::{Lua, MultiValue, Table};

use super::*;

pub(super) fn serialization(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  install_json(lua, &source)?;
  install_csv(lua, &source)?;
  install_yaml(lua, &source)?;
  install_toml(lua, &source)?;
  ini::install(lua, &source)?;
  xml::install(lua, &source)?;
  binary::install(lua, &source)?;
  readonly::proxy(lua, source)
}

fn install_json(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "json_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.json_encode";
      let value = args::one(method, "value", values)?;
      let value = value::lua_to_json(value, method)?;
      value::bounded_text(
        method,
        serde_json::to_string(&value).map_err(|_| args::message(method, "JSON encoding failed"))?,
      )
    })?,
  )?;
  source.raw_set(
    "json_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.json_decode";
      let text = value::text_argument(values, method)?;
      let decoded: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| args::message(method, "invalid JSON data"))?;
      value::json_to_lua(lua, &decoded, method)
    })?,
  )
}

fn install_csv(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "csv_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.csv_encode";
      let data = value::lua_to_json(args::one(method, "rows", values)?, method)?;
      let serde_json::Value::Array(rows) = data else {
        return Err(args::message(method, "expected a two-dimensional array"));
      };
      let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
      for row in rows {
        let serde_json::Value::Array(columns) = row else {
          return Err(args::message(method, "expected a two-dimensional array"));
        };
        let record = columns
          .into_iter()
          .map(|value| scalar_text(value, method))
          .collect::<mlua::Result<Vec<_>>>()?;
        writer
          .write_record(record)
          .map_err(|_| args::message(method, "CSV encoding failed"))?;
      }
      let bytes = writer
        .into_inner()
        .map_err(|_| args::message(method, "CSV encoding failed"))?;
      let text =
        String::from_utf8(bytes).map_err(|_| args::message(method, "CSV encoding failed"))?;
      value::bounded_text(method, text)
    })?,
  )?;
  source.raw_set(
    "csv_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.csv_decode";
      let text = value::text_argument(values, method)?;
      let rows = lua.create_table()?;
      let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
      for (row_index, record) in reader.records().enumerate() {
        if row_index >= args::MAX_API_TABLE_ENTRIES {
          return Err(args::message(method, "CSV exceeds 16384 rows"));
        }
        let record = record.map_err(|_| args::message(method, "invalid CSV data"))?;
        let row = lua.create_table()?;
        for (index, field) in record.iter().enumerate() {
          row.raw_set(index + 1, field)?;
        }
        rows.raw_set(row_index + 1, row)?;
      }
      Ok(rows)
    })?,
  )
}

fn install_yaml(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "yaml_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.yaml_encode";
      let data = value::lua_to_json(args::one(method, "value", values)?, method)?;
      let text =
        serde_yaml::to_string(&data).map_err(|_| args::message(method, "YAML encoding failed"))?;
      value::bounded_text(method, text)
    })?,
  )?;
  source.raw_set(
    "yaml_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.yaml_decode";
      let text = value::text_argument(values, method)?;
      let yaml: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|_| args::message(method, "invalid YAML data"))?;
      reject_yaml_tags(&yaml, method)?;
      let data =
        serde_json::to_value(yaml).map_err(|_| args::message(method, "unsupported YAML value"))?;
      value::json_to_lua(lua, &data, method)
    })?,
  )
}

fn install_toml(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "toml_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.toml_encode";
      let data = value::lua_to_json(args::one(method, "value", values)?, method)?;
      if !data.is_object() {
        return Err(args::message(method, "TOML root must be an object table"));
      }
      let text =
        toml::to_string(&data).map_err(|_| args::message(method, "TOML encoding failed"))?;
      value::bounded_text(method, text)
    })?,
  )?;
  source.raw_set(
    "toml_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.toml_decode";
      let text = value::text_argument(values, method)?;
      let data: toml::Value =
        toml::from_str(&text).map_err(|_| args::message(method, "invalid TOML data"))?;
      let json =
        serde_json::to_value(data).map_err(|_| args::message(method, "unsupported TOML value"))?;
      value::json_to_lua(lua, &json, method)
    })?,
  )
}

fn scalar_text(value: serde_json::Value, method: &str) -> mlua::Result<String> {
  match value {
    serde_json::Value::Null => Ok(String::new()),
    serde_json::Value::Bool(value) => Ok(value.to_string()),
    serde_json::Value::Number(value) => Ok(value.to_string()),
    serde_json::Value::String(value) => Ok(value),
    _ => Err(args::message(method, "CSV cells must be scalar values")),
  }
}

fn reject_yaml_tags(value: &serde_yaml::Value, method: &str) -> mlua::Result<()> {
  match value {
    serde_yaml::Value::Tagged(_) => Err(args::message(method, "YAML tags are not supported")),
    serde_yaml::Value::Sequence(values) => {
      for value in values {
        reject_yaml_tags(value, method)?;
      }
      Ok(())
    }
    serde_yaml::Value::Mapping(values) => {
      for (key, value) in values {
        reject_yaml_tags(key, method)?;
        reject_yaml_tags(value, method)?;
      }
      Ok(())
    }
    _ => Ok(()),
  }
}
