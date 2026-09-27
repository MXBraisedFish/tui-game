use super::*;

const MAX_I64_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

pub(super) fn math(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  install_constants(&source)?;

  source.raw_set("abs", unary_number(lua, "math.abs", f64::abs)?)?;
  source.raw_set("ceil", rounded_integer(lua, "math.ceil", f64::ceil)?)?;
  source.raw_set("floor", rounded_integer(lua, "math.floor", f64::floor)?)?;
  source.raw_set("round", rounded_integer(lua, "math.round", f64::round)?)?;
  source.raw_set("round_to", round_to(lua)?)?;
  source.raw_set("fmod", fmod(lua)?)?;
  source.raw_set("pow", pow(lua)?)?;
  source.raw_set("exp", unary_number(lua, "math.exp", f64::exp)?)?;
  source.raw_set("log", log(lua)?)?;
  source.raw_set("lg", positive_unary(lua, "math.lg", f64::log10)?)?;
  source.raw_set("ln", positive_unary(lua, "math.ln", f64::ln)?)?;
  source.raw_set("sqrt", non_negative_unary(lua, "math.sqrt", f64::sqrt)?)?;
  source.raw_set("ldexp", ldexp(lua)?)?;
  source.raw_set("frexp", frexp(lua)?)?;
  source.raw_set("sin", unary_number(lua, "math.sin", f64::sin)?)?;
  source.raw_set("cos", unary_number(lua, "math.cos", f64::cos)?)?;
  source.raw_set("tan", unary_number(lua, "math.tan", f64::tan)?)?;
  source.raw_set("asin", unit_range_unary(lua, "math.asin", f64::asin)?)?;
  source.raw_set("acos", unit_range_unary(lua, "math.acos", f64::acos)?)?;
  source.raw_set("atan", unary_number(lua, "math.atan", f64::atan)?)?;
  source.raw_set("atan2", atan2(lua)?)?;
  source.raw_set("deg", unary_number(lua, "math.deg", f64::to_degrees)?)?;
  source.raw_set("rad", unary_number(lua, "math.rad", f64::to_radians)?)?;
  source.raw_set("normalize_angle", normalize_angle(lua)?)?;
  source.raw_set("max", extremum(lua, true)?)?;
  source.raw_set("min", extremum(lua, false)?)?;
  source.raw_set("modf", modf(lua)?)?;
  source.raw_set("tointeger", tointeger(lua)?)?;
  source.raw_set("type", numeric_type(lua)?)?;
  source.raw_set("ult", ult(lua)?)?;
  source.raw_set("approx_equal", approx_equal(lua)?)?;
  source.raw_set("percent", percent(lua)?)?;
  source.raw_set("factorial", factorial(lua)?)?;
  source.raw_set("combination", combination(lua)?)?;

  readonly::proxy(lua, source)
}

fn install_constants(source: &Table) -> mlua::Result<()> {
  for (name, value) in [
    ("PI", PI),
    ("E", E),
    ("POSITIVE_INFINITE", f64::INFINITY),
    ("INFINITE", f64::INFINITY),
    ("NEGATIVE_INFINITE", f64::NEG_INFINITY),
    ("DEG", 180.0 / PI),
    ("RAD", PI / 180.0),
  ] {
    source.raw_set(name, value)?;
  }
  source.raw_set("MAX_INTEGER", i64::MAX)?;
  source.raw_set("MIN_INTEGER", i64::MIN)?;
  Ok(())
}

fn unary_number(
  lua: &Lua,
  method: &'static str,
  operation: fn(f64) -> f64,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let value = finite_number(args::one(method, "value", values)?, method, "value")?;
    finite_result(method, operation(value))
  })
}

fn positive_unary(
  lua: &Lua,
  method: &'static str,
  operation: fn(f64) -> f64,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let value = finite_number(args::one(method, "value", values)?, method, "value")?;
    if value <= 0.0 {
      return Err(args::message(method, "value must be greater than zero"));
    }
    finite_result(method, operation(value))
  })
}

fn non_negative_unary(
  lua: &Lua,
  method: &'static str,
  operation: fn(f64) -> f64,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let value = finite_number(args::one(method, "value", values)?, method, "value")?;
    if value < 0.0 {
      return Err(args::message(method, "value must not be negative"));
    }
    finite_result(method, operation(value))
  })
}

fn unit_range_unary(
  lua: &Lua,
  method: &'static str,
  operation: fn(f64) -> f64,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let value = finite_number(args::one(method, "value", values)?, method, "value")?;
    if !(-1.0..=1.0).contains(&value) {
      return Err(args::message(method, "value must be in -1..=1"));
    }
    finite_result(method, operation(value))
  })
}

fn rounded_integer(
  lua: &Lua,
  method: &'static str,
  operation: fn(f64) -> f64,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let value = args::one(method, "value", values)?;
    match value {
      Value::Integer(value) => Ok(value),
      value => {
        let value = finite_number(value, method, "value")?;
        finite_i64(method, operation(value))
      }
    }
  })
}

fn round_to(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.round_to", values, &["value", "digits"])?;
    let value = finite_number(
      args::required(&table, "math.round_to", "value")?,
      "math.round_to",
      "value",
    )?;
    let digits = args::integer(
      args::required(&table, "math.round_to", "digits")?,
      "math.round_to",
      "digits",
    )?;
    if !(-308..=308).contains(&digits) {
      return Err(args::message(
        "math.round_to",
        "digits must be in -308..=308",
      ));
    }

    let factor = 10_f64.powi(digits.unsigned_abs() as i32);
    let result = if digits >= 0 {
      if value.abs() > f64::MAX / factor {
        value
      } else {
        (value * factor).round() / factor
      }
    } else {
      (value / factor).round() * factor
    };
    finite_result("math.round_to", result)
  })
}

fn fmod(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.fmod", values, &["x", "y"])?;
    let x = args::integer(args::required(&table, "math.fmod", "x")?, "math.fmod", "x")?;
    let y = args::integer(args::required(&table, "math.fmod", "y")?, "math.fmod", "y")?;
    if y == 0 {
      return Err(args::message("math.fmod", "y must not be zero"));
    }
    Ok(x.checked_rem(y).unwrap_or(0))
  })
}

fn pow(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.pow", values, &["x", "y"])?;
    let x = finite_number(args::required(&table, "math.pow", "x")?, "math.pow", "x")?;
    let y = finite_number(args::required(&table, "math.pow", "y")?, "math.pow", "y")?;
    finite_result("math.pow", x.powf(y))
  })
}

fn log(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.log", values, &["value", "base"])?;
    let value = finite_number(
      args::required(&table, "math.log", "value")?,
      "math.log",
      "value",
    )?;
    let base = finite_number(
      args::required(&table, "math.log", "base")?,
      "math.log",
      "base",
    )?;
    if value <= 0.0 {
      return Err(args::message("math.log", "value must be greater than zero"));
    }
    if base <= 0.0 || base == 1.0 {
      return Err(args::message(
        "math.log",
        "base must be greater than zero and not equal to one",
      ));
    }
    finite_result("math.log", value.log(base))
  })
}

fn ldexp(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.ldexp", values, &["x", "exp"])?;
    let mut value = finite_number(
      args::required(&table, "math.ldexp", "x")?,
      "math.ldexp",
      "x",
    )?;
    let mut exponent = args::integer(
      args::required(&table, "math.ldexp", "exp")?,
      "math.ldexp",
      "exp",
    )?;
    if value == 0.0 {
      return Ok(value);
    }
    if exponent > 2097 {
      return Err(args::message(
        "math.ldexp",
        "result exceeds finite number range",
      ));
    }
    if exponent < -2097 {
      return Ok(0.0_f64.copysign(value));
    }

    while exponent > 1023 {
      value = finite_result("math.ldexp", value * 2_f64.powi(1023))?;
      exponent -= 1023;
    }
    while exponent < -1022 {
      value *= 2_f64.powi(-1022);
      if value == 0.0 {
        return Ok(value);
      }
      exponent += 1022;
    }
    finite_result("math.ldexp", value * 2_f64.powi(exponent as i32))
  })
}

fn frexp(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|lua, values: MultiValue| {
    let value = finite_number(
      args::one("math.frexp", "value", values)?,
      "math.frexp",
      "value",
    )?;
    let (mantissa, exponent) = frexp_parts(value);
    let result = lua.create_table()?;
    result.raw_set("mantissa", mantissa)?;
    result.raw_set("exponent", exponent)?;
    Ok(result)
  })
}

fn frexp_parts(value: f64) -> (f64, i64) {
  if value == 0.0 {
    return (value, 0);
  }

  let bits = value.to_bits();
  let exponent_bits = ((bits >> 52) & 0x7ff) as i64;
  if exponent_bits == 0 {
    let (mantissa, exponent) = frexp_parts(value * 2_f64.powi(64));
    return (mantissa, exponent - 64);
  }

  let mantissa_bits = (bits & (1_u64 << 63)) | (1022_u64 << 52) | (bits & ((1_u64 << 52) - 1));
  (f64::from_bits(mantissa_bits), exponent_bits - 1022)
}

fn atan2(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.atan2", values, &["y", "x"])?;
    let y = finite_number(
      args::required(&table, "math.atan2", "y")?,
      "math.atan2",
      "y",
    )?;
    let x = finite_number(
      args::required(&table, "math.atan2", "x")?,
      "math.atan2",
      "x",
    )?;
    finite_result("math.atan2", y.atan2(x))
  })
}

fn normalize_angle(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let value = args::integer(
      args::one("math.normalize_angle", "value", values)?,
      "math.normalize_angle",
      "value",
    )?;
    Ok(value.rem_euclid(360) as f64)
  })
}

fn extremum(lua: &Lua, maximum: bool) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let method = if maximum { "math.max" } else { "math.min" };
    let value = args::one(method, "values", values)?;
    let table = match value {
      Value::Table(table) => table,
      value => return Err(args::invalid(method, "values", "table", &value)),
    };
    let len = table.raw_len();
    if len == 0 {
      return Err(args::message(method, "values must not be empty"));
    }
    if len > args::MAX_API_TABLE_ENTRIES {
      return Err(args::message(method, "values contains too many entries"));
    }
    for pair in table.clone().pairs::<Value, Value>() {
      let (key, _) = pair?;
      let valid = match key {
        Value::Integer(index) => index >= 1 && index as usize <= len,
        Value::String(name) => name.to_str()?.as_ref() == "n",
        _ => false,
      };
      if !valid {
        return Err(args::message(method, "values must be a dense array"));
      }
    }

    let mut result = finite_number(table.raw_get::<Value>(1)?, method, "values")?;
    for index in 2..=len {
      let value = finite_number(table.raw_get::<Value>(index)?, method, "values")?;
      result = if maximum {
        result.max(value)
      } else {
        result.min(value)
      };
    }
    Ok(result)
  })
}

fn modf(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|lua, values: MultiValue| {
    let value = args::one("math.modf", "value", values)?;
    let (integer_part, fractional_part) = match value {
      Value::Integer(value) => (value, 0.0),
      value => {
        let value = finite_number(value, "math.modf", "value")?;
        let integer_part = finite_i64("math.modf", value.trunc())?;
        (integer_part, value - integer_part as f64)
      }
    };
    let result = lua.create_table()?;
    result.raw_set("integer_part", integer_part)?;
    result.raw_set("fractional_part", fractional_part)?;
    Ok(result)
  })
}

fn tointeger(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let value = args::one("math.tointeger", "value", values)?;
    Ok(match value {
      Value::Integer(value) => Value::Integer(value),
      Value::Number(value) if is_exact_i64(value) => Value::Integer(value as i64),
      _ => Value::Nil,
    })
  })
}

fn numeric_type(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|lua, values: MultiValue| {
    let value = args::one("math.type", "value", values)?;
    match value {
      Value::Integer(_) => Ok(Value::String(lua.create_string("integer")?)),
      Value::Number(_) => Ok(Value::String(lua.create_string("float")?)),
      _ => Ok(Value::Nil),
    }
  })
}

fn ult(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.ult", values, &["left", "right"])?;
    let left = args::integer(
      args::required(&table, "math.ult", "left")?,
      "math.ult",
      "left",
    )? as u64;
    let right = args::integer(
      args::required(&table, "math.ult", "right")?,
      "math.ult",
      "right",
    )? as u64;
    Ok(left < right)
  })
}

fn approx_equal(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.approx_equal", values, &["left", "right", "epsilon"])?;
    let left = finite_number(
      args::required(&table, "math.approx_equal", "left")?,
      "math.approx_equal",
      "left",
    )?;
    let right = finite_number(
      args::required(&table, "math.approx_equal", "right")?,
      "math.approx_equal",
      "right",
    )?;
    let epsilon = match table.get::<Value>("epsilon")? {
      Value::Nil => 1e-10,
      value => finite_number(value, "math.approx_equal", "epsilon")?,
    };
    if epsilon < 0.0 {
      return Err(args::message(
        "math.approx_equal",
        "epsilon must not be negative",
      ));
    }

    Ok(left == right || (left - right).abs() <= epsilon)
  })
}

fn percent(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.percent", values, &["value", "total", "as_percent"])?;
    let value = finite_number(
      args::required(&table, "math.percent", "value")?,
      "math.percent",
      "value",
    )?;
    let total = finite_number(
      args::required(&table, "math.percent", "total")?,
      "math.percent",
      "total",
    )?;
    if total == 0.0 {
      return Err(args::message("math.percent", "total must not be zero"));
    }
    let as_percent = args::optional_bool(&table, "math.percent", "as_percent", false)?;
    let result = value / total;
    finite_result(
      "math.percent",
      if as_percent { result * 100.0 } else { result },
    )
  })
}

fn factorial(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let n = args::integer(
      args::one("math.factorial", "n", values)?,
      "math.factorial",
      "n",
    )?;
    if !(0..=170).contains(&n) {
      return Err(args::message("math.factorial", "n must be in 0..=170"));
    }
    finite_result(
      "math.factorial",
      (1..=n).fold(1.0, |result, value| result * value as f64),
    )
  })
}

fn combination(lua: &Lua) -> mlua::Result<Function> {
  lua.create_function(|_, values: MultiValue| {
    let table = args::named("math.combination", values, &["n", "k"])?;
    let n = args::integer(
      args::required(&table, "math.combination", "n")?,
      "math.combination",
      "n",
    )?;
    let mut k = args::integer(
      args::required(&table, "math.combination", "k")?,
      "math.combination",
      "k",
    )?;
    if n < 0 || k < 0 || k > n {
      return Err(args::message("math.combination", "requires 0 <= k <= n"));
    }

    k = k.min(n - k);
    let mut result = 1_u128;
    for index in 1..=k {
      let numerator = (n - k + index) as u128;
      result = result * numerator / index as u128;
      if result > i64::MAX as u128 {
        return Err(args::message(
          "math.combination",
          "result exceeds integer range",
        ));
      }
    }
    Ok(result as i64)
  })
}

fn finite_number(value: Value, method: &str, parameter: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, parameter)?;
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(
      method,
      format!("invalid parameter '{parameter}': expected finite number"),
    ))
  }
}

fn finite_result(method: &str, value: f64) -> mlua::Result<f64> {
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(
      method,
      "result exceeds finite number range or is undefined",
    ))
  }
}

fn finite_i64(method: &str, value: f64) -> mlua::Result<i64> {
  if is_exact_i64(value) {
    Ok(value as i64)
  } else {
    Err(args::message(method, "result exceeds integer range"))
  }
}

fn is_exact_i64(value: f64) -> bool {
  value.is_finite() && value.fract() == 0.0 && value >= i64::MIN as f64 && value < MAX_I64_EXCLUSIVE
}
