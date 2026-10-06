//! Positional date arguments and strict trailing options for Unix millisecond conversions.

use super::*;
use tg_core_date::{
  DateFields, DateTimeZone, date_to_timestamp, now_timestamp, timestamp_diff, timestamp_to_date,
};

/// Build the date library with shared time-zone and output-format constants.
///
/// # Errors
///
/// Propagate Lua allocation, table-construction, or callback-registration errors.
pub(super) fn date(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("LOCAL", "local"),
    ("UTC", "utc"),
    ("TIMESTAMP", "timestamp"),
    ("DATE", "date"),
  ] {
    source.raw_set(name, value)?;
  }
  for hours in -12_i32..=14 {
    if hours == 0 {
      continue;
    }
    let sign = if hours < 0 { "MINUS" } else { "PLUS" };
    source.raw_set(
      format!("UTC_{sign}_{}", hours.abs()),
      format!("utc{hours:+}"),
    )?;
  }
  source.raw_set(
    "now",
    lua.create_function(|lua, values: MultiValue| {
      let method = "date.now";
      let parsed = args::positional(lua, method, values, &[], &["timezone", "time_type"])?;
      let timezone = parse_timezone(parsed.options(), method)?;
      let time_type =
        args::optional_string(parsed.options(), method, "time_type", Some("timestamp"))?
          .unwrap_or_else(|| "timestamp".to_string());
      match time_type.as_str() {
        "timestamp" => Ok(Value::Integer(now_timestamp())),
        "date" => {
          let date = timestamp_to_date(now_timestamp(), timezone)
            .map_err(|error| args::message(method, error.to_string()))?;
          Ok(Value::Table(date_table(lua, date)?))
        }
        _ => Err(args::message(
          method,
          "time_type must be date.TIMESTAMP or date.DATE",
        )),
      }
    })?,
  )?;
  source.raw_set(
    "date_to_timestamp",
    lua.create_function(|lua, values: MultiValue| {
      let method = "date.date_to_timestamp";
      let parsed = args::positional(
        lua,
        method,
        values,
        &[
          "year",
          "month",
          "day",
          "hour",
          "minute",
          "second",
          "millisecond",
        ],
        &["timezone"],
      )?;
      let date = parse_date_fields(method, "", |index, name| {
        parsed.required(index, method, name)
      })?;
      date_to_timestamp(date, parse_timezone(parsed.options(), method)?)
        .map_err(|error| args::message(method, error.to_string()))
    })?,
  )?;
  source.raw_set(
    "timestamp_to_date",
    lua.create_function(|lua, values: MultiValue| {
      let method = "date.timestamp_to_date";
      let parsed = args::positional(lua, method, values, &["timestamp"], &["timezone"])?;
      let timestamp = args::integer(
        parsed.required(0, method, "timestamp")?,
        method,
        "timestamp",
      )?;
      let date = timestamp_to_date(timestamp, parse_timezone(parsed.options(), method)?)
        .map_err(|error| args::message(method, error.to_string()))?;
      date_table(lua, date)
    })?,
  )?;
  source.raw_set(
    "timestamp_diff",
    lua.create_function(|lua, values: MultiValue| {
      let method = "date.timestamp_diff";
      let parsed = args::positional(
        lua,
        method,
        values,
        &["early_timestamp", "later_timestamp"],
        &["timezone"],
      )?;
      let timezone = parse_timezone(parsed.options(), method)?;
      let early = parse_timestamp_or_date(
        parsed.required(0, method, "early_timestamp")?,
        timezone,
        method,
        "early_timestamp",
      )?;
      let later = parse_timestamp_or_date(
        parsed.required(1, method, "later_timestamp")?,
        timezone,
        method,
        "later_timestamp",
      )?;
      timestamp_diff(early, later).map_err(|error| args::message(method, error.to_string()))
    })?,
  )?;
  readonly::proxy(lua, source)
}

/// Read seven integral calendar fields from positional arguments or a date table.
///
/// # Arguments
///
/// * `method` - The public method name included in errors.
/// * `prefix` - The parent date argument name, or an empty string for positional fields.
/// * `read` - The reader for a field's position and name.
///
/// # Errors
///
/// Return an argument error for missing, non-integral, or out-of-range calendar fields.
fn parse_date_fields(
  method: &str,
  prefix: &str,
  read: impl Fn(usize, &str) -> mlua::Result<Value>,
) -> mlua::Result<DateFields> {
  let field_name = |name: &str| {
    if prefix.is_empty() {
      name.to_string()
    } else {
      format!("{prefix}.{name}")
    }
  };
  let integer = |index, name| args::integer(read(index, name)?, method, &field_name(name));
  let unsigned = |index, name| {
    u32::try_from(integer(index, name)?).map_err(|_| {
      args::message(
        method,
        format!("{} must be a non-negative 32-bit integer", field_name(name)),
      )
    })
  };
  Ok(DateFields {
    year: i32::try_from(integer(0, "year")?).map_err(|_| {
      args::message(
        method,
        format!("{} must be a signed 32-bit integer", field_name("year")),
      )
    })?,
    month: unsigned(1, "month")?,
    day: unsigned(2, "day")?,
    hour: unsigned(3, "hour")?,
    minute: unsigned(4, "minute")?,
    second: unsigned(5, "second")?,
    millisecond: unsigned(6, "millisecond")?,
  })
}

/// Resolve an integer timestamp or a seven-field date into Unix milliseconds.
///
/// # Arguments
///
/// * `value` - The timestamp or date supplied by the script.
/// * `timezone` - The time zone used only for a calendar date.
/// * `method` - The public method name included in errors.
/// * `name` - The early or later argument name included in errors.
///
/// # Errors
///
/// Return an argument error for invalid timestamps or fields, or a conversion error for an
/// invalid, unsupported, ambiguous, or unavailable calendar date.
fn parse_timestamp_or_date(
  value: Value,
  timezone: DateTimeZone,
  method: &str,
  name: &str,
) -> mlua::Result<i64> {
  if let Value::Table(table) = value {
    let date = parse_date_fields(method, name, |_, field| table.raw_get(field))?;
    date_to_timestamp(date, timezone)
      .map_err(|error| args::message(method, format!("{name}: {error}")))
  } else {
    args::integer(value, method, name)
  }
}

fn parse_timezone(options: &Table, method: &str) -> mlua::Result<DateTimeZone> {
  let value = args::optional_string(options, method, "timezone", Some("local"))?
    .unwrap_or_else(|| "local".to_string());
  match value.as_str() {
    "local" => return Ok(DateTimeZone::Local),
    "utc" => return Ok(DateTimeZone::Utc),
    _ => {}
  }
  if let Some(hours) = value
    .strip_prefix("utc")
    .and_then(|hours| hours.parse::<i32>().ok())
    && (-12..=14).contains(&hours)
    && hours != 0
    && value == format!("utc{hours:+}")
  {
    return Ok(DateTimeZone::FixedOffset(hours * 3600));
  }
  Err(args::message(
    method,
    "timezone must be date.LOCAL, date.UTC or a date.UTC_PLUS_n/date.UTC_MINUS_n constant",
  ))
}

fn date_table(lua: &Lua, date: DateFields) -> mlua::Result<Table> {
  let result = lua.create_table()?;
  for (name, value) in [
    ("year", i64::from(date.year)),
    ("month", i64::from(date.month)),
    ("day", i64::from(date.day)),
    ("hour", i64::from(date.hour)),
    ("minute", i64::from(date.minute)),
    ("second", i64::from(date.second)),
    ("millisecond", i64::from(date.millisecond)),
  ] {
    result.raw_set(name, value)?;
  }
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn vm() -> Lua {
    let lua = Lua::new();
    lua.globals().set("date", date(&lua).unwrap()).unwrap();
    lua
  }

  #[test]
  fn timestamp_diff_returns_one_non_negative_integer_and_rejects_invalid_input() {
    vm().load(r#"
      local function fails(fn, expected)
        local ok, message = pcall(fn)
        assert(not ok and tostring(message):find(expected, 1, true), tostring(message))
      end
      for _, case in ipairs({
        {1000, 2500, 1500}, {-2500, -1000, 1500}, {-500, 1000, 1500},
        {math.mininteger, math.mininteger + 1, 1},
        {math.maxinteger - 1, math.maxinteger, 1},
        {math.mininteger, -1, math.maxinteger},
        {0, 0, 0}, {-1, -1, 0}, {1000, 1000, 0},
        {math.mininteger, math.mininteger, 0},
        {math.maxinteger, math.maxinteger, 0}
      }) do
        local difference = date.timestamp_diff(case[1], case[2])
        assert(math.type(difference) == 'integer' and difference == case[3])
        assert(select('#', date.timestamp_diff(case[1], case[2])) == 1)
      end
      fails(function() date.timestamp_diff(2, 1) end, 'greater than')
      fails(function() date.timestamp_diff(math.mininteger, math.maxinteger) end, 'range')
      fails(function() date.timestamp_diff() end, 'early_timestamp')
      fails(function() date.timestamp_diff(0) end, 'later_timestamp')
      fails(function() date.timestamp_diff(nil, 1) end, 'early_timestamp')
      fails(function() date.timestamp_diff(0, nil) end, 'later_timestamp')
      fails(function() date.timestamp_diff('0', 1) end, 'early_timestamp')
      fails(function() date.timestamp_diff(0, 1.5) end, 'later_timestamp')
      fails(function() date.timestamp_diff(0, math.huge) end, 'later_timestamp')
      assert(date.timestamp_diff(0, 1, {}) == 1)
      assert(date.timestamp_diff(0, 1, nil) == 1)
      fails(function() date.timestamp_diff({early_timestamp = 0, later_timestamp = 1}) end, 'later_timestamp')
      fails(function() date.timestamp_diff({}, 1) end, 'early_timestamp')
    "#).exec().unwrap();
  }

  #[test]
  fn timestamp_diff_accepts_calendar_dates_mixed_operands_and_shared_timezones() {
    vm().load(r#"
      for hours = -12, 14 do
        local timezone = date.UTC
        if hours < 0 then timezone = date['UTC_MINUS_' .. -hours] end
        if hours > 0 then timezone = date['UTC_PLUS_' .. hours] end
        for _, timestamp in ipairs({-1001, -1, 0, 123, 1700000000123}) do
          local early = date.timestamp_to_date(timestamp, {timezone = timezone})
          local later = date.timestamp_to_date(timestamp + 1500, {timezone = timezone})
          assert(date.timestamp_diff(early, later, {timezone = timezone}) == 1500)
          assert(date.timestamp_diff(early, timestamp + 1500, {timezone = timezone}) == 1500)
          assert(date.timestamp_diff(timestamp, later, {timezone = timezone}) == 1500)
          assert(date.timestamp_diff(early, timestamp, {timezone = timezone}) == 0)
          assert(date.timestamp_diff(timestamp, early, {timezone = timezone}) == 0)
          assert(select('#', date.timestamp_diff(early, later, {timezone = timezone})) == 1)
          assert(date.timestamp_diff(timestamp, timestamp + 1500, {timezone = timezone}) == 1500)
          assert(early.millisecond == date.timestamp_to_date(timestamp, {timezone = timezone}).millisecond)
        end
      end
      local early = date.timestamp_to_date(1700000000123)
      local later = date.timestamp_to_date(1700000000124)
      assert(date.timestamp_diff(early, later) == 1)
      assert(date.timestamp_diff(early, later, {}) == 1)
      assert(date.timestamp_diff(early, later, nil) == 1)
      assert(date.timestamp_diff(early, 1700000000124) == 1)
      assert(date.timestamp_diff(1700000000123, later) == 1)
      local utc_early = date.timestamp_to_date(0, {timezone = date.UTC})
      local utc_later = date.timestamp_to_date(1, {timezone = date.UTC})
      local east = date.timestamp_to_date(0, {timezone = date.UTC_PLUS_8})
      assert(date.timestamp_diff(utc_early, utc_later, {timezone = date.UTC}) == 1)
      assert(date.timestamp_diff(utc_early, east, {timezone = date.UTC}) == 8 * 3600000)
      local leap = {year = 2000, month = 2, day = 29, hour = 23, minute = 59, second = 59, millisecond = 999}
      local next_day = {year = 2000, month = 3, day = 1, hour = 0, minute = 0, second = 0, millisecond = 0}
      assert(date.timestamp_diff(leap, next_day, {timezone = date.UTC}) == 1)
      local current = date.now({time_type = date.DATE})
      assert(date.timestamp_diff(current, current) == 0)
    "#).exec().unwrap();
  }

  #[test]
  fn timestamp_diff_rejects_bad_date_fields_order_and_options_with_argument_context() {
    vm().load(r#"
      local function fails(fn, expected)
        local ok, message = pcall(fn)
        assert(not ok and tostring(message):find(expected, 1, true), tostring(message))
      end
      local early = date.timestamp_to_date(0, {timezone = date.UTC})
      local later = date.timestamp_to_date(1, {timezone = date.UTC})
      for _, field in ipairs({'year', 'month', 'day', 'hour', 'minute', 'second', 'millisecond'}) do
        local saved = early[field]
        for _, invalid in ipairs({'1', false, 1.5, math.huge}) do
          early[field] = invalid
          fails(function() date.timestamp_diff(early, 1, {timezone = date.UTC}) end, 'early_timestamp.' .. field)
          fails(function() date.timestamp_diff(0, early, {timezone = date.UTC}) end, 'later_timestamp.' .. field)
        end
        early[field] = nil
        fails(function() date.timestamp_diff(early, 1, {timezone = date.UTC}) end, 'early_timestamp.' .. field)
        early[field] = saved
      end
      for _, case in ipairs({
        {'year', math.maxinteger}, {'year', 2147483647}, {'month', 0}, {'month', 13},
        {'day', 0}, {'day', 32}, {'hour', 24}, {'minute', 60}, {'second', 60},
        {'millisecond', -1}, {'millisecond', 1000}, {'day', 4294967296}
      }) do
        local field, saved = case[1], early[case[1]]
        early[field] = case[2]
        fails(function() date.timestamp_diff(early, 1, {timezone = date.UTC}) end, 'early_timestamp')
        early[field] = saved
      end
      early.year = 1900; early.month = 2; early.day = 29
      fails(function() date.timestamp_diff(early, 1, {timezone = date.UTC}) end, 'invalid calendar')
      fails(function() date.timestamp_diff(later, 0, {timezone = date.UTC}) end, 'later_timestamp must')
      fails(function() date.timestamp_diff(1, date.timestamp_to_date(0, {timezone = date.UTC}), {timezone = date.UTC}) end, 'later_timestamp must')
      fails(function() date.timestamp_diff(0, 1, {unknown = true}) end, 'unknown option')
      fails(function() date.timestamp_diff(0, 1, {[1] = date.UTC}) end, 'option names')
      fails(function() date.timestamp_diff(0, 1, setmetatable({}, {})) end, 'metatable')
      fails(function() date.timestamp_diff(0, 1, {timezone = false}) end, 'timezone')
      fails(function() date.timestamp_diff(0, 1, {timezone = 'utc+15'}) end, 'timezone')
      fails(function() date.timestamp_diff(0, 1, date.UTC) end, 'table or nil')
      fails(function() date.timestamp_diff(0, 1, {}, {}) end, 'arguments')
      fails(function() date.timestamp_diff(false, 1) end, 'early_timestamp')
    "#).exec().unwrap();
  }

  #[test]
  fn now_returns_exactly_one_integer_or_a_seven_field_date() {
    let lua = vm();
    let before = now_timestamp();
    let actual: i64 = lua
      .load(
        r#"
      assert(select('#', date.now()) == 1)
      assert(select('#', date.now({time_type = date.DATE})) == 1)
      assert(type(date.now({})) == 'number' and type(date.now(nil)) == 'number')
      local t = date.now({timezone = date.UTC, time_type = date.DATE})
      local count = 0
      for _, value in pairs(t) do
        assert(math.type(value) == 'integer')
        count = count + 1
      end
      assert(count == 7 and t.millisecond >= 0 and t.millisecond <= 999)
      return date.now()
    "#,
      )
      .eval()
      .unwrap();
    assert!((before..=now_timestamp()).contains(&actual));
  }

  #[test]
  fn every_fixed_timezone_roundtrips_milliseconds_and_negative_timestamps() {
    vm().load(r#"
      for hours = -12, 14 do
        local timezone = date.UTC
        if hours < 0 then timezone = date['UTC_MINUS_' .. -hours] end
        if hours > 0 then timezone = date['UTC_PLUS_' .. hours] end
        for _, timestamp in ipairs({-1001, -1, 0, 123, 1700000000123}) do
          local t = date.timestamp_to_date(timestamp, {timezone = timezone})
          local restored = date.date_to_timestamp(t.year, t.month, t.day, t.hour, t.minute, t.second, t.millisecond, {timezone = timezone})
          assert(restored == timestamp)
          assert(select('#', date.date_to_timestamp(t.year, t.month, t.day, t.hour, t.minute, t.second, t.millisecond, {timezone = timezone})) == 1)
          assert(select('#', date.timestamp_to_date(timestamp, {timezone = timezone})) == 1)
        end
      end
      local epoch = date.timestamp_to_date(0, {timezone = date.UTC_PLUS_8})
      assert(epoch.year == 1970 and epoch.month == 1 and epoch.day == 1 and epoch.hour == 8)
      local before = date.timestamp_to_date(-1, {timezone = date.UTC})
      assert(before.year == 1969 and before.month == 12 and before.day == 31)
      assert(before.hour == 23 and before.minute == 59 and before.second == 59 and before.millisecond == 999)
    "#).exec().unwrap();
  }

  #[test]
  fn conversion_options_can_be_omitted_empty_or_nil() {
    vm().load(r#"
      local a = date.timestamp_to_date(1700000000123)
      local b = date.timestamp_to_date(1700000000123, {})
      local c = date.timestamp_to_date(1700000000123, nil)
      for field, value in pairs(a) do assert(b[field] == value and c[field] == value) end
      assert(date.date_to_timestamp(a.year, a.month, a.day, a.hour, a.minute, a.second, a.millisecond) == 1700000000123)
      assert(date.date_to_timestamp(a.year, a.month, a.day, a.hour, a.minute, a.second, a.millisecond, {}) == 1700000000123)
      assert(date.date_to_timestamp(a.year, a.month, a.day, a.hour, a.minute, a.second, a.millisecond, nil) == 1700000000123)
    "#).exec().unwrap();
  }

  #[test]
  fn invalid_positions_options_formats_and_dates_raise_lua_errors() {
    vm().load(r#"
      local valid = {2000, 2, 29, 23, 59, 59, 999}
      local function convert(options)
        return date.date_to_timestamp(valid[1], valid[2], valid[3], valid[4], valid[5], valid[6], valid[7], options)
      end
      local t = date.timestamp_to_date(convert({timezone = date.UTC}), {timezone = date.UTC})
      assert(t.year == 2000 and t.day == 29 and t.millisecond == 999)
      local function fails(fn, expected)
        local ok, message = pcall(fn)
        assert(not ok and tostring(message):find(expected, 1, true), tostring(message))
      end
      fails(function() date.now({unknown = true}) end, 'unknown option')
      fails(function() date.now({time_type = 'invalid'}) end, 'time_type')
      fails(function() date.now({timezone = 'utc+15'}) end, 'timezone')
      fails(function() date.now({timezone = 'utc-13'}) end, 'timezone')
      fails(function() date.now({timezone = false}) end, 'timezone')
      fails(function() date.now({time_type = 1}) end, 'time_type')
      fails(function() date.now({}, {}) end, 'arguments')
      fails(function() date.now({[1] = date.UTC}) end, 'option names')
      fails(function() date.now(setmetatable({}, {})) end, 'metatable')
      fails(function() date.date_to_timestamp() end, 'year')
      fails(function() date.date_to_timestamp(2000, 2, 29, 23, 59, 59) end, 'millisecond')
      fails(function() date.date_to_timestamp({year = 2000, month = 2, day = 29}) end, 'month')
      fails(function() date.timestamp_to_date() end, 'timestamp')
      fails(function() date.timestamp_to_date(nil) end, 'timestamp')
      fails(function() date.timestamp_to_date({timestamp = 0}) end, 'timestamp')
      fails(function() date.timestamp_to_date('0') end, 'timestamp')
      fails(function() date.timestamp_to_date(0.5) end, 'timestamp')
      fails(function() date.timestamp_to_date(math.huge) end, 'timestamp')
      fails(function() date.timestamp_to_date(math.maxinteger) end, 'supported range')
      fails(function() date.timestamp_to_date(math.mininteger) end, 'supported range')
      fails(function() date.timestamp_to_date(0, {time_type = date.DATE}) end, 'unknown option')
      fails(function() date.timestamp_to_date(0, {timestamp = 0}) end, 'unknown option')
      fails(function() date.timestamp_to_date(0, {[1] = date.UTC}) end, 'option names')
      fails(function() date.timestamp_to_date(0, setmetatable({}, {})) end, 'metatable')
      fails(function() date.timestamp_to_date(0, date.UTC) end, 'table or nil')
      fails(function() date.timestamp_to_date(0, {}, {}) end, 'arguments')
      fails(function() date.timestamp_to_date(0, {timezone = 'invalid'}) end, 'timezone')
      fails(function() convert({year = 2000}) end, 'unknown option')
      fails(function() convert({timezone = 8}) end, 'timezone')
      fails(function() convert(setmetatable({}, {})) end, 'metatable')
      fails(function() convert({[1] = date.UTC}) end, 'option names')
      fails(function() convert(date.UTC) end, 'table or nil')
      fails(function() date.date_to_timestamp(2000, 2, 29, 23, 59, 59, 999, {}, {}) end, 'arguments')
      for index, field in ipairs({'year', 'month', 'day', 'hour', 'minute', 'second', 'millisecond'}) do
        local saved = valid[index]
        valid[index] = nil
        fails(function() convert({timezone = date.UTC}) end, field)
        valid[index] = '1'
        fails(function() convert({timezone = date.UTC}) end, field)
        valid[index] = saved
      end
      valid[1] = 1900
      fails(function() convert({timezone = date.UTC}) end, 'invalid calendar')
      valid[1] = 2000
      valid[7] = 1000
      fails(function() convert({timezone = date.UTC}) end, 'invalid calendar')
      valid[7] = -1
      fails(function() convert({timezone = date.UTC}) end, 'millisecond')
      valid[7] = 0
      valid[3] = math.maxinteger
      fails(function() convert({timezone = date.UTC}) end, 'day')
      valid[3] = 29
      valid[1] = math.maxinteger
      fails(function() convert({timezone = date.UTC}) end, 'year')
    "#).exec().unwrap();
  }

  #[test]
  fn calendar_results_are_independent_and_constants_reject_assignment() {
    vm()
      .load(
        r#"
      local a = date.timestamp_to_date(123, {timezone = date.UTC})
      a.year = 2026
      local b = date.timestamp_to_date(123, {timezone = date.UTC})
      assert(a ~= b and b.year == 1970)
      assert(not pcall(function() date.UTC = 'other' end))
      assert(not pcall(function() date.extra = true end))
      assert(date.UTC == 'utc' and date.TIMESTAMP == 'timestamp' and date.DATE == 'date')
    "#,
      )
      .exec()
      .unwrap();
  }
}
