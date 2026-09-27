use mlua::{Lua, MultiValue, Table, Value};

use super::args;

#[derive(Clone, Copy)]
enum Endian {
  Little,
  Big,
}

#[derive(Clone, Copy)]
enum Kind {
  Signed(usize),
  Unsigned(usize),
  Float32,
  Float64,
  FixedString(usize),
  ZeroString,
  LengthString(usize),
  Padding,
  Align(usize),
}

#[derive(Clone, Copy)]
struct Op {
  kind: Kind,
  endian: Endian,
  max_align: usize,
}

pub(super) fn install(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "binary_pack",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.binary_pack";
      let table = args::named(method, values, &["fmt", "values"])?;
      let format = args::string(args::required(&table, method, "fmt")?, method, "fmt")?;
      let values = args::values(&table, method)?;
      let operations = parse_format(&format, method)?;
      let mut output = Vec::new();
      let mut value_index = 0;
      for operation in operations {
        apply_alignment(&mut output, operation)?;
        pack_operation(
          operation,
          values.get(value_index).cloned(),
          &mut output,
          &mut value_index,
          method,
        )?;
        if output.len() > args::MAX_API_STRING_BYTES {
          return Err(args::message(method, "packed output exceeds 1 MiB"));
        }
      }
      if value_index != values.len() {
        return Err(args::message(method, "too many values for format string"));
      }
      Ok(Value::String(lua.create_string(output)?))
    })?,
  )?;

  source.raw_set(
    "binary_unpack",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.binary_unpack";
      let table = args::named(method, values, &["fmt", "data", "pos"])?;
      let format = args::string(args::required(&table, method, "fmt")?, method, "fmt")?;
      let data_value = args::required(&table, method, "data")?;
      let Value::String(data) = data_value else {
        return Err(args::invalid(method, "data", "string", &data_value));
      };
      if data.as_bytes().len() > args::MAX_API_STRING_BYTES {
        return Err(args::message(method, "input exceeds 1 MiB"));
      }
      let pos = args::optional_integer(&table, method, "pos", Some(1))?.unwrap();
      let mut offset = usize::try_from(pos.saturating_sub(1))
        .map_err(|_| args::message(method, "pos must be at least 1"))?;
      if pos < 1 || offset > data.as_bytes().len() {
        return Err(args::message(method, "pos is outside the input string"));
      }
      let operations = parse_format(&format, method)?;
      let results = lua.create_table()?;
      let mut result_index = 1;
      for operation in operations {
        offset = aligned_offset(offset, operation)?;
        if let Some(value) =
          unpack_operation(lua, operation, &data.as_bytes(), &mut offset, method)?
        {
          results.raw_set(result_index, value)?;
          result_index += 1;
        }
      }
      let result = lua.create_table()?;
      result.raw_set("values", results)?;
      result.raw_set("next_pos", offset + 1)?;
      Ok(result)
    })?,
  )?;

  source.raw_set(
    "binary_packsize",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.binary_packsize";
      let format = args::string(args::one(method, "fmt", values)?, method, "fmt")?;
      let mut size = 0_usize;
      for operation in parse_format(&format, method)? {
        size = aligned_offset(size, operation)?;
        size = size
          .checked_add(
            fixed_size(operation.kind)
              .ok_or_else(|| args::message(method, "format contains a variable-length string"))?,
          )
          .ok_or_else(|| args::message(method, "format size overflow"))?;
        if size > args::MAX_API_STRING_BYTES {
          return Err(args::message(method, "format size exceeds 1 MiB"));
        }
      }
      Ok(size)
    })?,
  )
}

fn parse_format(format: &str, method: &str) -> mlua::Result<Vec<Op>> {
  if format.len() > 8192 {
    return Err(args::message(method, "format string exceeds 8 KiB"));
  }
  let bytes = format.as_bytes();
  let mut index = 0;
  let mut endian = native_endian();
  let mut max_align = 1_usize;
  let mut output = Vec::new();
  while index < bytes.len() {
    let byte = bytes[index];
    index += 1;
    match byte {
      b' ' | b'\t' | b'\r' | b'\n' => continue,
      b'<' => {
        endian = Endian::Little;
        continue;
      }
      b'>' => {
        endian = Endian::Big;
        continue;
      }
      b'=' => {
        endian = native_endian();
        continue;
      }
      b'!' => {
        let value = parse_number(bytes, &mut index).unwrap_or(std::mem::size_of::<usize>());
        if value == 0 || value > 16 || !value.is_power_of_two() {
          return Err(args::message(
            method,
            "alignment must be a power of two in 1..=16",
          ));
        }
        max_align = value;
        continue;
      }
      b'X' => {
        let kind = parse_data_kind(bytes, &mut index, method)?;
        let size = alignment_size(kind)
          .ok_or_else(|| args::message(method, "X requires a fixed-size option"))?;
        output.push(Op {
          kind: Kind::Align(size),
          endian,
          max_align,
        });
        continue;
      }
      _ => {
        index -= 1;
        let kind = parse_data_kind(bytes, &mut index, method)?;
        output.push(Op {
          kind,
          endian,
          max_align,
        });
      }
    }
    if output.len() > 8192 {
      return Err(args::message(method, "format contains too many options"));
    }
  }
  Ok(output)
}

fn parse_data_kind(bytes: &[u8], index: &mut usize, method: &str) -> mlua::Result<Kind> {
  let byte = *bytes
    .get(*index)
    .ok_or_else(|| args::message(method, "incomplete format option"))?;
  *index += 1;
  let kind = match byte {
    b'b' => Kind::Signed(1),
    b'B' => Kind::Unsigned(1),
    b'h' => Kind::Signed(2),
    b'H' => Kind::Unsigned(2),
    b'l' | b'j' => Kind::Signed(8),
    b'L' | b'J' | b'T' => Kind::Unsigned(8),
    b'i' => Kind::Signed(integer_size(bytes, index, method)?),
    b'I' => Kind::Unsigned(integer_size(bytes, index, method)?),
    b'f' => Kind::Float32,
    b'd' | b'n' => Kind::Float64,
    b'c' => {
      let size =
        parse_number(bytes, index).ok_or_else(|| args::message(method, "c requires a size"))?;
      Kind::FixedString(size)
    }
    b'z' => Kind::ZeroString,
    b's' => {
      let size = parse_number(bytes, index).unwrap_or(std::mem::size_of::<usize>());
      if !(1..=8).contains(&size) {
        return Err(args::message(method, "string length size must be in 1..=8"));
      }
      Kind::LengthString(size)
    }
    b'x' => Kind::Padding,
    _ => {
      return Err(args::message(
        method,
        format!("invalid format option '{}'", byte as char),
      ));
    }
  };
  Ok(kind)
}

fn integer_size(bytes: &[u8], index: &mut usize, method: &str) -> mlua::Result<usize> {
  let size = parse_number(bytes, index).unwrap_or(4);
  if (1..=16).contains(&size) {
    Ok(size)
  } else {
    Err(args::message(method, "integer size must be in 1..=16"))
  }
}

fn parse_number(bytes: &[u8], index: &mut usize) -> Option<usize> {
  let start = *index;
  let mut value = 0_usize;
  while let Some(byte @ b'0'..=b'9') = bytes.get(*index).copied() {
    value = value.checked_mul(10)?.checked_add((byte - b'0') as usize)?;
    *index += 1;
  }
  (*index > start).then_some(value)
}

fn apply_alignment(output: &mut Vec<u8>, operation: Op) -> mlua::Result<()> {
  let aligned = aligned_offset(output.len(), operation)?;
  output.resize(aligned, 0);
  Ok(())
}

fn aligned_offset(offset: usize, operation: Op) -> mlua::Result<usize> {
  let size = alignment_size(operation.kind).unwrap_or(1);
  let alignment = size.min(operation.max_align).max(1);
  let padding = (alignment - offset % alignment) % alignment;
  offset
    .checked_add(padding)
    .ok_or_else(|| mlua::Error::RuntimeError("serialization.binary: size overflow".to_string()))
}

fn alignment_size(kind: Kind) -> Option<usize> {
  match kind {
    Kind::Signed(size)
    | Kind::Unsigned(size)
    | Kind::FixedString(size)
    | Kind::LengthString(size)
    | Kind::Align(size) => Some(size),
    Kind::Float32 => Some(4),
    Kind::Float64 => Some(8),
    Kind::Padding | Kind::ZeroString => Some(1),
  }
}

fn fixed_size(kind: Kind) -> Option<usize> {
  match kind {
    Kind::Signed(size) | Kind::Unsigned(size) | Kind::FixedString(size) => Some(size),
    Kind::Float32 => Some(4),
    Kind::Float64 => Some(8),
    Kind::Padding => Some(1),
    Kind::Align(_) => Some(0),
    Kind::ZeroString | Kind::LengthString(_) => None,
  }
}

fn pack_operation(
  operation: Op,
  value: Option<Value>,
  output: &mut Vec<u8>,
  value_index: &mut usize,
  method: &str,
) -> mlua::Result<()> {
  match operation.kind {
    Kind::Padding | Kind::Align(_) => {
      if matches!(operation.kind, Kind::Padding) {
        output.push(0);
      }
      return Ok(());
    }
    _ => {}
  }
  let value = value.ok_or_else(|| args::message(method, "not enough values for format string"))?;
  *value_index += 1;
  match operation.kind {
    Kind::Signed(size) => write_signed(
      args::integer(value, method, "values")?,
      size,
      operation.endian,
      output,
      method,
    ),
    Kind::Unsigned(size) => {
      let value = args::integer(value, method, "values")?;
      if value < 0 {
        return Err(args::message(method, "unsigned integer cannot be negative"));
      }
      write_unsigned(value as u64, size, operation.endian, output, method)
    }
    Kind::Float32 => {
      let value = finite_number(value, method)? as f32;
      let bytes = match operation.endian {
        Endian::Little => value.to_le_bytes(),
        Endian::Big => value.to_be_bytes(),
      };
      output.extend_from_slice(&bytes);
      Ok(())
    }
    Kind::Float64 => {
      let value = finite_number(value, method)?;
      let bytes = match operation.endian {
        Endian::Little => value.to_le_bytes(),
        Endian::Big => value.to_be_bytes(),
      };
      output.extend_from_slice(&bytes);
      Ok(())
    }
    Kind::FixedString(size) => {
      let bytes = string_bytes(value, method)?;
      if bytes.len() > size {
        return Err(args::message(method, "string is longer than fixed field"));
      }
      output.extend_from_slice(&bytes);
      output.resize(output.len() + size - bytes.len(), 0);
      Ok(())
    }
    Kind::ZeroString => {
      let bytes = string_bytes(value, method)?;
      if bytes.contains(&0) {
        return Err(args::message(
          method,
          "zero-terminated string contains a zero byte",
        ));
      }
      output.extend_from_slice(&bytes);
      output.push(0);
      Ok(())
    }
    Kind::LengthString(size) => {
      let bytes = string_bytes(value, method)?;
      write_unsigned(bytes.len() as u64, size, operation.endian, output, method)?;
      output.extend_from_slice(&bytes);
      Ok(())
    }
    Kind::Padding | Kind::Align(_) => unreachable!(),
  }
}

fn unpack_operation(
  lua: &Lua,
  operation: Op,
  data: &[u8],
  offset: &mut usize,
  method: &str,
) -> mlua::Result<Option<Value>> {
  let result = match operation.kind {
    Kind::Align(_) => return Ok(None),
    Kind::Padding => {
      take(data, offset, 1, method)?;
      return Ok(None);
    }
    Kind::Signed(size) => Value::Integer(read_signed(
      take(data, offset, size, method)?,
      operation.endian,
      method,
    )?),
    Kind::Unsigned(size) => {
      let value = read_unsigned(take(data, offset, size, method)?, operation.endian, method)?;
      Value::Integer(
        i64::try_from(value)
          .map_err(|_| args::message(method, "unsigned integer does not fit a Lua integer"))?,
      )
    }
    Kind::Float32 => {
      let bytes: [u8; 4] = take(data, offset, 4, method)?.try_into().unwrap();
      Value::Number(match operation.endian {
        Endian::Little => f32::from_le_bytes(bytes),
        Endian::Big => f32::from_be_bytes(bytes),
      } as f64)
    }
    Kind::Float64 => {
      let bytes: [u8; 8] = take(data, offset, 8, method)?.try_into().unwrap();
      Value::Number(match operation.endian {
        Endian::Little => f64::from_le_bytes(bytes),
        Endian::Big => f64::from_be_bytes(bytes),
      })
    }
    Kind::FixedString(size) => Value::String(lua.create_string(take(data, offset, size, method)?)?),
    Kind::ZeroString => {
      let remaining = &data[*offset..];
      let length = remaining
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| args::message(method, "unfinished zero-terminated string"))?;
      let bytes = &remaining[..length];
      *offset += length + 1;
      Value::String(lua.create_string(bytes)?)
    }
    Kind::LengthString(size) => {
      let length = read_unsigned(take(data, offset, size, method)?, operation.endian, method)?;
      let length =
        usize::try_from(length).map_err(|_| args::message(method, "string length is too large"))?;
      Value::String(lua.create_string(take(data, offset, length, method)?)?)
    }
  };
  Ok(Some(result))
}

fn take<'a>(
  data: &'a [u8],
  offset: &mut usize,
  size: usize,
  method: &str,
) -> mlua::Result<&'a [u8]> {
  let end = offset
    .checked_add(size)
    .ok_or_else(|| args::message(method, "data position overflow"))?;
  let value = data
    .get(*offset..end)
    .ok_or_else(|| args::message(method, "input is shorter than the format requires"))?;
  *offset = end;
  Ok(value)
}

fn write_signed(
  value: i64,
  size: usize,
  endian: Endian,
  output: &mut Vec<u8>,
  method: &str,
) -> mlua::Result<()> {
  if size < 8 {
    let bits = size * 8;
    let min = -(1_i128 << (bits - 1));
    let max = (1_i128 << (bits - 1)) - 1;
    if !(min..=max).contains(&i128::from(value)) {
      return Err(args::message(method, "signed integer does not fit format"));
    }
  }
  let fill = if value < 0 { 0xff } else { 0 };
  let bytes = match endian {
    Endian::Little => value.to_le_bytes(),
    Endian::Big => value.to_be_bytes(),
  };
  if size <= 8 {
    match endian {
      Endian::Little => output.extend_from_slice(&bytes[..size]),
      Endian::Big => output.extend_from_slice(&bytes[8 - size..]),
    }
  } else if matches!(endian, Endian::Little) {
    output.extend_from_slice(&bytes);
    output.resize(output.len() + size - 8, fill);
  } else {
    output.resize(output.len() + size - 8, fill);
    output.extend_from_slice(&bytes);
  }
  Ok(())
}

fn write_unsigned(
  value: u64,
  size: usize,
  endian: Endian,
  output: &mut Vec<u8>,
  method: &str,
) -> mlua::Result<()> {
  if size < 8 && value >= (1_u64 << (size * 8)) {
    return Err(args::message(
      method,
      "unsigned integer does not fit format",
    ));
  }
  let bytes = match endian {
    Endian::Little => value.to_le_bytes(),
    Endian::Big => value.to_be_bytes(),
  };
  if size <= 8 {
    match endian {
      Endian::Little => output.extend_from_slice(&bytes[..size]),
      Endian::Big => output.extend_from_slice(&bytes[8 - size..]),
    }
  } else if matches!(endian, Endian::Little) {
    output.extend_from_slice(&bytes);
    output.resize(output.len() + size - 8, 0);
  } else {
    output.resize(output.len() + size - 8, 0);
    output.extend_from_slice(&bytes);
  }
  Ok(())
}

fn read_unsigned(bytes: &[u8], endian: Endian, method: &str) -> mlua::Result<u64> {
  if bytes.len() > 8 {
    let extension = if matches!(endian, Endian::Little) {
      &bytes[8..]
    } else {
      &bytes[..bytes.len() - 8]
    };
    if extension.iter().any(|byte| *byte != 0) {
      return Err(args::message(
        method,
        "unsigned integer does not fit a Lua integer",
      ));
    }
  }
  let significant = if bytes.len() <= 8 {
    bytes
  } else if matches!(endian, Endian::Little) {
    &bytes[..8]
  } else {
    &bytes[bytes.len() - 8..]
  };
  let mut buffer = [0_u8; 8];
  match endian {
    Endian::Little => buffer[..significant.len()].copy_from_slice(significant),
    Endian::Big => buffer[8 - significant.len()..].copy_from_slice(significant),
  }
  Ok(match endian {
    Endian::Little => u64::from_le_bytes(buffer),
    Endian::Big => u64::from_be_bytes(buffer),
  })
}

fn read_signed(bytes: &[u8], endian: Endian, method: &str) -> mlua::Result<i64> {
  let sign = if matches!(endian, Endian::Little) {
    bytes.first()
  } else {
    bytes.last()
  }
  .is_some_and(|byte| byte & 0x80 != 0);
  if bytes.len() > 8 {
    let extension = if matches!(endian, Endian::Little) {
      &bytes[8..]
    } else {
      &bytes[..bytes.len() - 8]
    };
    let expected = if sign { 0xff } else { 0 };
    if extension.iter().any(|byte| *byte != expected) {
      return Err(args::message(
        method,
        "signed integer does not fit a Lua integer",
      ));
    }
  }
  let significant = if bytes.len() <= 8 {
    bytes
  } else if matches!(endian, Endian::Little) {
    &bytes[..8]
  } else {
    &bytes[bytes.len() - 8..]
  };
  let mut buffer = [if sign { 0xff } else { 0 }; 8];
  match endian {
    Endian::Little => buffer[..significant.len()].copy_from_slice(significant),
    Endian::Big => buffer[8 - significant.len()..].copy_from_slice(significant),
  }
  Ok(match endian {
    Endian::Little => i64::from_le_bytes(buffer),
    Endian::Big => i64::from_be_bytes(buffer),
  })
}

fn string_bytes(value: Value, method: &str) -> mlua::Result<Vec<u8>> {
  let Value::String(value) = value else {
    return Err(args::invalid(method, "values", "string", &value));
  };
  Ok(value.as_bytes().to_vec())
}

fn finite_number(value: Value, method: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, "values")?;
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(
      method,
      "non-finite numbers are not supported",
    ))
  }
}

fn native_endian() -> Endian {
  if cfg!(target_endian = "little") {
    Endian::Little
  } else {
    Endian::Big
  }
}
