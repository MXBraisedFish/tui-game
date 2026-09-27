use base64::Engine;
use mlua::{Lua, MultiValue, Table, Value};

use super::*;

pub(super) fn encoding(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  install_base64(lua, &source)?;
  install_url(lua, &source)?;
  install_hex(lua, &source)?;
  readonly::proxy(lua, source)
}

fn install_base64(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "base64_encode",
    lua.create_function(|lua, values: MultiValue| {
      let bytes = bytes_argument(values, "encoding.base64_encode")?;
      let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
      Ok(Value::String(lua.create_string(encoded)?))
    })?,
  )?;
  source.raw_set(
    "base64_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "encoding.base64_decode";
      let bytes = bytes_argument(values, method)?;
      if bytes.len() > encoded_input_limit() {
        return Err(args::message(
          method,
          "input exceeds the encoded size limit",
        ));
      }
      let decoded = base64::engine::general_purpose::STANDARD
        .decode(bytes)
        .map_err(|_| args::message(method, "invalid Base64 data"))?;
      bounded_result(lua, method, decoded)
    })?,
  )
}

fn install_url(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "url_encode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "encoding.url_encode";
      let bytes = bytes_argument(values, method)?;
      let mut output = Vec::with_capacity(bytes.len().saturating_mul(3));
      const HEX: &[u8; 16] = b"0123456789ABCDEF";
      for byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
          output.push(byte);
        } else {
          output.extend_from_slice(&[b'%', HEX[(byte >> 4) as usize], HEX[(byte & 0x0f) as usize]]);
        }
        if output.len() > args::MAX_API_STRING_BYTES {
          return Err(args::message(method, "encoded output exceeds 1 MiB"));
        }
      }
      Ok(Value::String(lua.create_string(output)?))
    })?,
  )?;
  source.raw_set(
    "url_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "encoding.url_decode";
      let bytes = bytes_argument(values, method)?;
      let mut output = Vec::with_capacity(bytes.len());
      let mut index = 0;
      while index < bytes.len() {
        if bytes[index] == b'%' {
          if index + 2 >= bytes.len() {
            return Err(args::message(method, "incomplete percent escape"));
          }
          let high = hex_digit(bytes[index + 1])
            .ok_or_else(|| args::message(method, "invalid percent escape"))?;
          let low = hex_digit(bytes[index + 2])
            .ok_or_else(|| args::message(method, "invalid percent escape"))?;
          output.push((high << 4) | low);
          index += 3;
        } else {
          output.push(bytes[index]);
          index += 1;
        }
      }
      bounded_result(lua, method, output)
    })?,
  )
}

fn install_hex(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "hex_encode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "encoding.hex_encode";
      let bytes = bytes_argument(values, method)?;
      if bytes.len() > args::MAX_API_STRING_BYTES / 2 {
        return Err(args::message(method, "encoded output exceeds 1 MiB"));
      }
      const HEX: &[u8; 16] = b"0123456789abcdef";
      let mut output = Vec::with_capacity(bytes.len() * 2);
      for byte in bytes {
        output.extend_from_slice(&[HEX[(byte >> 4) as usize], HEX[(byte & 0x0f) as usize]]);
      }
      Ok(Value::String(lua.create_string(output)?))
    })?,
  )?;
  source.raw_set(
    "hex_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "encoding.hex_decode";
      let bytes = bytes_argument(values, method)?;
      if bytes.len() % 2 != 0 {
        return Err(args::message(
          method,
          "hexadecimal input must have an even length",
        ));
      }
      let mut output = Vec::with_capacity(bytes.len() / 2);
      for pair in bytes.chunks_exact(2) {
        let high =
          hex_digit(pair[0]).ok_or_else(|| args::message(method, "invalid hexadecimal data"))?;
        let low =
          hex_digit(pair[1]).ok_or_else(|| args::message(method, "invalid hexadecimal data"))?;
        output.push((high << 4) | low);
      }
      bounded_result(lua, method, output)
    })?,
  )
}

fn bytes_argument(values: MultiValue, method: &str) -> mlua::Result<Vec<u8>> {
  let value = args::one(method, "s", values)?;
  let Value::String(value) = value else {
    return Err(args::invalid(method, "s", "string", &value));
  };
  if value.as_bytes().len() > args::MAX_API_STRING_BYTES {
    return Err(args::message(method, "input exceeds 1 MiB"));
  }
  Ok(value.as_bytes().to_vec())
}

fn bounded_result(lua: &Lua, method: &str, output: Vec<u8>) -> mlua::Result<Value> {
  if output.len() > args::MAX_API_STRING_BYTES {
    return Err(args::message(method, "decoded output exceeds 1 MiB"));
  }
  Ok(Value::String(lua.create_string(output)?))
}

fn encoded_input_limit() -> usize {
  args::MAX_API_STRING_BYTES.saturating_mul(4) / 3 + 4
}

fn hex_digit(value: u8) -> Option<u8> {
  match value {
    b'0'..=b'9' => Some(value - b'0'),
    b'a'..=b'f' => Some(value - b'a' + 10),
    b'A'..=b'F' => Some(value - b'A' + 10),
    _ => None,
  }
}
