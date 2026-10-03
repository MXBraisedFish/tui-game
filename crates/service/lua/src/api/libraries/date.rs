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
      let integer =
        |index, name| args::integer(parsed.required(index, method, name)?, method, name);
      let unsigned = |index, name| {
        u32::try_from(integer(index, name)?).map_err(|_| {
          args::message(
            method,
            format!("{name} must be a non-negative 32-bit integer"),
          )
        })
      };
      let date = DateFields {
        year: i32::try_from(integer(0, "year")?)
          .map_err(|_| args::message(method, "year must be a signed 32-bit integer"))?,
        month: unsigned(1, "month")?,
        day: unsigned(2, "day")?,
        hour: unsigned(3, "hour")?,
        minute: unsigned(4, "minute")?,
        second: unsigned(5, "second")?,
        millisecond: unsigned(6, "millisecond")?,
      };
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
        &["left_timestamp", "right_timestamp"],
        &[],
      )?;
      let left = args::integer(
        parsed.required(0, method, "left_timestamp")?,
        method,
        "left_timestamp",
      )?;
      let right = args::integer(
        parsed.required(1, method, "right_timestamp")?,
        method,
        "right_timestamp",
      )?;
      timestamp_diff(left, right).map_err(|error| args::message(method, error.to_string()))
    })?,
  )?;
  readonly::proxy(lua, source)
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
      fails(function() date.timestamp_diff() end, 'left_timestamp')
      fails(function() date.timestamp_diff(0) end, 'right_timestamp')
      fails(function() date.timestamp_diff(nil, 1) end, 'left_timestamp')
      fails(function() date.timestamp_diff(0, nil) end, 'right_timestamp')
      fails(function() date.timestamp_diff('0', 1) end, 'left_timestamp')
      fails(function() date.timestamp_diff(0, 1.5) end, 'right_timestamp')
      fails(function() date.timestamp_diff(0, math.huge) end, 'right_timestamp')
      fails(function() date.timestamp_diff(0, 1, {}) end, 'expected 2')
      fails(function() date.timestamp_diff(0, 1, nil) end, 'expected 2')
      fails(function() date.timestamp_diff({left_timestamp = 0, right_timestamp = 1}) end, 'right_timestamp')
      fails(function() date.timestamp_diff({}, 1) end, 'left_timestamp')
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
