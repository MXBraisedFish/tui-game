//! 系统时间与日期转换

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Local, NaiveDate, TimeZone, Timelike};

/// 当前现实世界时间戳，毫秒。
pub fn current_timestamp_ms() -> mlua::Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(mlua::Error::external)?;
    i64::try_from(duration.as_millis()).map_err(mlua::Error::external)
}

/// 当前本地时间。
pub fn current_local_time() -> mlua::Result<chrono::DateTime<Local>> {
    let timestamp_ms = current_timestamp_ms()?;
    Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .ok_or_else(|| mlua::Error::external("invalid local time"))
}

/// 格式化时间戳。
pub fn timestamp_to_date(timestamp_ms: i64, format_text: &str) -> mlua::Result<String> {
    if timestamp_ms < 0 {
        return Err(mlua::Error::external("timestamp must be non-negative"));
    }
    let local_time = Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .ok_or_else(|| mlua::Error::external("invalid timestamp"))?;
    Ok(format_text
        .replace("{year}", &format!("{:04}", local_time.year()))
        .replace("{month}", &format!("{:02}", local_time.month()))
        .replace("{day}", &format!("{:02}", local_time.day()))
        .replace("{hour}", &format!("{:02}", local_time.hour()))
        .replace("{minute}", &format!("{:02}", local_time.minute()))
        .replace("{second}", &format!("{:02}", local_time.second())))
}

/// 日期参数转时间戳。
pub fn date_to_timestamp(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> mlua::Result<i64> {
    let year = i32::try_from(year).map_err(mlua::Error::external)?;
    let month = u32::try_from(month).map_err(mlua::Error::external)?;
    let day = u32::try_from(day).map_err(mlua::Error::external)?;
    let hour = u32::try_from(hour).map_err(mlua::Error::external)?;
    let minute = u32::try_from(minute).map_err(mlua::Error::external)?;
    let second = u32::try_from(second).map_err(mlua::Error::external)?;
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| mlua::Error::external("invalid date"))?;
    let datetime = date
        .and_hms_opt(hour, minute, second)
        .ok_or_else(|| mlua::Error::external("invalid time"))?;
    let local_time = Local
        .from_local_datetime(&datetime)
        .single()
        .ok_or_else(|| mlua::Error::external("ambiguous or invalid local datetime"))?;
    Ok(local_time.timestamp_millis())
}
