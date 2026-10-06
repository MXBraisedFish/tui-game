//! Unix millisecond timestamps and calendar dates in local, UTC, or fixed-offset time zones.
//!
//! This crate needs only the system clock and, for local dates, the operating system's
//! time-zone configuration. Run `cargo run -p tg-core-date --example date_smoke --locked`
//! without starting the host or providing game assets.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_date::{DateTimeZone, date_to_timestamp, timestamp_to_date};
//!
//! let date = timestamp_to_date(123, DateTimeZone::FixedOffset(8 * 3600))?;
//! assert_eq!((date.year, date.month, date.day, date.hour), (1970, 1, 1, 8));
//! assert_eq!(date.millisecond, 123);
//! assert_eq!(date_to_timestamp(date, DateTimeZone::FixedOffset(8 * 3600))?, 123);
//! # Ok::<(), tg_core_date::DateError>(())
//! ```

use chrono::{
  DateTime, Datelike, FixedOffset, Local, LocalResult, NaiveDate, TimeZone, Timelike, Utc,
};
use std::fmt;

/// A time zone used to interpret or display calendar dates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateTimeZone {
  /// The operating system's local time zone, including its daylight-saving rules.
  #[default]
  Local,
  /// Coordinated Universal Time, with no offset or daylight-saving transitions.
  Utc,
  /// A fixed number of seconds east of UTC; negative values are west of UTC.
  ///
  /// The absolute offset must be less than 24 hours. Fixed offsets never use daylight saving.
  FixedOffset(i32),
}

/// A Gregorian calendar date with millisecond precision.
///
/// # Fields
///
/// * `year` - The calendar year, using astronomical numbering (year zero is 1 BCE).
/// * `month` - The month in `1..=12`.
/// * `day` - The day in `1..=31`, subject to the selected month and leap year.
/// * `hour` - The hour in `0..=23`.
/// * `minute` - The minute in `0..=59`.
/// * `second` - The second in `0..=59`; leap-second input is not accepted.
/// * `millisecond` - The millisecond in `0..=999`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateFields {
  /// The calendar year.
  pub year: i32,
  /// The month, starting at one.
  pub month: u32,
  /// The day of the month, starting at one.
  pub day: u32,
  /// The hour on a 24-hour clock.
  pub hour: u32,
  /// The minute within the hour.
  pub minute: u32,
  /// The second within the minute.
  pub second: u32,
  /// The millisecond within the second.
  pub millisecond: u32,
}

/// A rejected calendar date, time-zone offset, or timestamp operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateError {
  /// The supplied calendar fields do not form a supported date and time.
  InvalidDate,
  /// The fixed UTC offset has an absolute value of 24 hours or more.
  InvalidOffset,
  /// The timestamp or offset-adjusted calendar date exceeds the supported range.
  OutOfRange,
  /// The local wall-clock time refers to two instants during a backward clock change.
  AmbiguousLocalTime,
  /// The local wall-clock time is skipped during a forward clock change or cannot be resolved.
  UnavailableLocalTime,
  /// The later timestamp is less than the early timestamp.
  InvalidTimestampOrder,
  /// The timestamp difference cannot fit in a signed 64-bit millisecond count.
  DifferenceOverflow,
}

impl fmt::Display for DateError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::InvalidDate => "invalid calendar date or time (milliseconds must be in 0..=999)",
      Self::InvalidOffset => "UTC offset must be less than 24 hours in either direction",
      Self::OutOfRange => "timestamp or time-zone-adjusted date is outside the supported range",
      Self::AmbiguousLocalTime => "local date is ambiguous; choose an explicit fixed UTC offset",
      Self::UnavailableLocalTime => {
        "local date does not exist or local time-zone data is unavailable"
      }
      Self::InvalidTimestampOrder => {
        "later_timestamp must be greater than or equal to early_timestamp"
      }
      Self::DifferenceOverflow => {
        "timestamp difference exceeds the signed 64-bit millisecond range"
      }
    })
  }
}

impl std::error::Error for DateError {}

/// Return milliseconds since the Unix epoch from the system wall clock.
///
/// This value describes an instant and does not change with the chosen time zone. It is
/// not a monotonic clock and must not be used to measure frame durations.
pub fn now_timestamp() -> i64 {
  Utc::now().timestamp_millis()
}

/// Return the non-negative difference between two Unix millisecond timestamps.
///
/// Require `later_timestamp >= early_timestamp`; equal timestamps return zero. Subtraction does
/// not depend on time zones and accepts negative timestamps without converting them to calendar dates.
///
/// # Errors
///
/// Return [`DateError::InvalidTimestampOrder`] when the later timestamp is less than
/// the early timestamp, or [`DateError::DifferenceOverflow`] when the difference exceeds
/// the signed 64-bit range.
///
/// # Examples
///
/// ```rust
/// use tg_core_date::timestamp_diff;
///
/// assert_eq!(timestamp_diff(-500, 1000)?, 1500);
/// assert_eq!(timestamp_diff(1000, 1000)?, 0);
/// # Ok::<(), tg_core_date::DateError>(())
/// ```
pub fn timestamp_diff(early_timestamp: i64, later_timestamp: i64) -> Result<i64, DateError> {
  if later_timestamp < early_timestamp {
    return Err(DateError::InvalidTimestampOrder);
  }
  later_timestamp
    .checked_sub(early_timestamp)
    .ok_or(DateError::DifferenceOverflow)
}

/// Convert a Gregorian calendar date in the selected time zone into Unix milliseconds.
///
/// Negative timestamps represent instants before `1970-01-01T00:00:00Z`. Ambiguous local
/// dates are rejected instead of silently choosing one daylight-saving offset.
///
/// # Errors
///
/// Return [`DateError::InvalidDate`] for invalid calendar fields or unsupported years,
/// [`DateError::InvalidOffset`] for an invalid fixed offset, [`DateError::OutOfRange`] for
/// UTC adjustment overflow, or a local-time error for ambiguous or unavailable local dates.
pub fn date_to_timestamp(date: DateFields, timezone: DateTimeZone) -> Result<i64, DateError> {
  let local_date = NaiveDate::from_ymd_opt(date.year, date.month, date.day)
    .and_then(|day| day.and_hms_milli_opt(date.hour, date.minute, date.second, date.millisecond))
    .filter(|_| date.millisecond < 1000)
    .ok_or(DateError::InvalidDate)?;
  let offset = match timezone {
    DateTimeZone::Local => resolve_local_offset(Local.offset_from_local_datetime(&local_date))?,
    DateTimeZone::Utc => 0,
    DateTimeZone::FixedOffset(seconds) => FixedOffset::east_opt(seconds)
      .ok_or(DateError::InvalidOffset)?
      .local_minus_utc(),
  };
  let utc = local_date
    .checked_sub_signed(chrono::Duration::seconds(i64::from(offset)))
    .ok_or(DateError::OutOfRange)?;
  Ok(utc.and_utc().timestamp_millis())
}

/// Convert Unix milliseconds into a Gregorian calendar date in the selected time zone.
///
/// Preserve the millisecond remainder for negative timestamps as well as positive ones.
///
/// # Errors
///
/// Return [`DateError::OutOfRange`] when the timestamp or offset-adjusted date is unsupported,
/// or [`DateError::InvalidOffset`] when the fixed offset is outside the allowed range.
///
/// # Panics
///
/// The local-time backend may panic if the operating system cannot supply UTC-offset data.
pub fn timestamp_to_date(timestamp: i64, timezone: DateTimeZone) -> Result<DateFields, DateError> {
  let utc = DateTime::<Utc>::from_timestamp_millis(timestamp).ok_or(DateError::OutOfRange)?;
  let offset = match timezone {
    DateTimeZone::Local => Local
      .offset_from_utc_datetime(&utc.naive_utc())
      .local_minus_utc(),
    DateTimeZone::Utc => 0,
    DateTimeZone::FixedOffset(seconds) => FixedOffset::east_opt(seconds)
      .ok_or(DateError::InvalidOffset)?
      .local_minus_utc(),
  };
  // Check the shifted calendar range before reading fields; naive_local would panic at the edge.
  let date = utc
    .naive_utc()
    .checked_add_signed(chrono::Duration::seconds(i64::from(offset)))
    .ok_or(DateError::OutOfRange)?;
  Ok(DateFields {
    year: date.year(),
    month: date.month(),
    day: date.day(),
    hour: date.hour(),
    minute: date.minute(),
    second: date.second(),
    millisecond: date.and_utc().timestamp_subsec_millis(),
  })
}

fn resolve_local_offset(result: LocalResult<FixedOffset>) -> Result<i32, DateError> {
  match result {
    LocalResult::Single(offset) => Ok(offset.local_minus_utc()),
    LocalResult::Ambiguous(_, _) => Err(DateError::AmbiguousLocalTime),
    LocalResult::None => Err(DateError::UnavailableLocalTime),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn timestamp_differences_enforce_order_and_preserve_integer_boundaries() {
    for (left, right, difference) in [
      (1000, 2500, 1500),
      (-2500, -1000, 1500),
      (-500, 1000, 1500),
      (i64::MIN, i64::MIN + 1, 1),
      (i64::MAX - 1, i64::MAX, 1),
      (i64::MIN, -1, i64::MAX),
      (0, i64::MAX, i64::MAX),
      (0, 0, 0),
      (-1, -1, 0),
      (1000, 1000, 0),
      (i64::MIN, i64::MIN, 0),
      (i64::MAX, i64::MAX, 0),
    ] {
      assert_eq!(timestamp_diff(left, right), Ok(difference));
    }
    for (left, right) in [(1000, 500), (i64::MAX, i64::MIN)] {
      assert_eq!(
        timestamp_diff(left, right),
        Err(DateError::InvalidTimestampOrder)
      );
    }
    for (left, right) in [(i64::MIN, 0), (-1, i64::MAX), (i64::MIN, i64::MAX)] {
      assert_eq!(
        timestamp_diff(left, right),
        Err(DateError::DifferenceOverflow)
      );
    }
  }

  #[test]
  fn timestamps_preserve_milliseconds_before_and_after_the_epoch() {
    for timestamp in [
      -2_208_988_800_001,
      -1001,
      -1000,
      -999,
      -1,
      0,
      1,
      999,
      1001,
      1_700_000_000_123,
    ] {
      for timezone in [
        DateTimeZone::Utc,
        DateTimeZone::FixedOffset(8 * 3600),
        DateTimeZone::FixedOffset(-5 * 3600),
      ] {
        let date = timestamp_to_date(timestamp, timezone).unwrap();
        assert!(date.millisecond < 1000);
        assert_eq!(date_to_timestamp(date, timezone), Ok(timestamp));
      }
    }
    let date = timestamp_to_date(-1, DateTimeZone::Utc).unwrap();
    assert_eq!((date.year, date.month, date.day), (1969, 12, 31));
    assert_eq!(
      (date.hour, date.minute, date.second, date.millisecond),
      (23, 59, 59, 999)
    );
  }

  #[test]
  fn fixed_offsets_cross_calendar_boundaries_without_changing_the_instant() {
    let east = timestamp_to_date(0, DateTimeZone::FixedOffset(14 * 3600)).unwrap();
    let west = timestamp_to_date(0, DateTimeZone::FixedOffset(-12 * 3600)).unwrap();
    assert_eq!(
      (east.year, east.month, east.day, east.hour),
      (1970, 1, 1, 14)
    );
    assert_eq!(
      (west.year, west.month, west.day, west.hour),
      (1969, 12, 31, 12)
    );
    assert_eq!(
      date_to_timestamp(east, DateTimeZone::FixedOffset(14 * 3600)),
      Ok(0)
    );
    assert_eq!(
      date_to_timestamp(west, DateTimeZone::FixedOffset(-12 * 3600)),
      Ok(0)
    );
  }

  #[test]
  fn leap_years_and_clock_fields_are_validated() {
    let leap = DateFields {
      year: 2000,
      month: 2,
      day: 29,
      hour: 23,
      minute: 59,
      second: 59,
      millisecond: 999,
    };
    let timestamp = date_to_timestamp(leap, DateTimeZone::Utc).unwrap();
    assert_eq!(timestamp_to_date(timestamp, DateTimeZone::Utc), Ok(leap));
    for invalid in [
      DateFields { year: 1900, ..leap },
      DateFields { month: 0, ..leap },
      DateFields { month: 13, ..leap },
      DateFields { day: 0, ..leap },
      DateFields { day: 30, ..leap },
      DateFields { hour: 24, ..leap },
      DateFields { minute: 60, ..leap },
      DateFields { second: 60, ..leap },
      DateFields {
        millisecond: 1000,
        ..leap
      },
      DateFields {
        year: i32::MAX,
        ..leap
      },
    ] {
      assert_eq!(
        date_to_timestamp(invalid, DateTimeZone::Utc),
        Err(DateError::InvalidDate)
      );
    }
  }

  #[test]
  fn oversized_timestamps_and_calendar_offset_overflow_return_errors() {
    for timestamp in [i64::MIN, i64::MAX] {
      assert_eq!(
        timestamp_to_date(timestamp, DateTimeZone::Utc),
        Err(DateError::OutOfRange)
      );
    }
    let min = NaiveDate::MIN
      .and_hms_opt(0, 0, 0)
      .unwrap()
      .and_utc()
      .timestamp_millis();
    let max = NaiveDate::MAX
      .and_hms_milli_opt(23, 59, 59, 999)
      .unwrap()
      .and_utc()
      .timestamp_millis();
    for (timestamp, seconds) in [(min, -3600), (max, 3600)] {
      assert_eq!(
        timestamp_to_date(timestamp, DateTimeZone::FixedOffset(seconds)),
        Err(DateError::OutOfRange)
      );
      let date = timestamp_to_date(timestamp, DateTimeZone::Utc).unwrap();
      assert_eq!(
        date_to_timestamp(date, DateTimeZone::FixedOffset(-seconds)),
        Err(DateError::OutOfRange)
      );
    }
  }

  #[test]
  fn invalid_fixed_offsets_are_rejected_in_both_directions() {
    let date = timestamp_to_date(0, DateTimeZone::Utc).unwrap();
    for seconds in [i32::MIN, -86400, 86400, i32::MAX] {
      let timezone = DateTimeZone::FixedOffset(seconds);
      assert_eq!(
        date_to_timestamp(date, timezone),
        Err(DateError::InvalidOffset)
      );
      assert_eq!(
        timestamp_to_date(0, timezone),
        Err(DateError::InvalidOffset)
      );
    }
  }

  #[test]
  fn local_offset_resolution_rejects_missing_and_ambiguous_wall_times() {
    let standard = FixedOffset::east_opt(-5 * 3600).unwrap();
    let daylight = FixedOffset::east_opt(-4 * 3600).unwrap();
    assert_eq!(
      resolve_local_offset(LocalResult::Single(standard)),
      Ok(-5 * 3600)
    );
    assert_eq!(
      resolve_local_offset(LocalResult::Ambiguous(standard, daylight)),
      Err(DateError::AmbiguousLocalTime)
    );
    assert_eq!(
      resolve_local_offset(LocalResult::None),
      Err(DateError::UnavailableLocalTime)
    );
  }

  #[test]
  fn current_local_date_matches_the_system_timezone() {
    let timestamp = now_timestamp();
    let expected = Local.timestamp_millis_opt(timestamp).single().unwrap();
    let date = timestamp_to_date(timestamp, DateTimeZone::Local).unwrap();
    assert_eq!(
      (date.year, date.month, date.day, date.hour, date.minute),
      (
        expected.year(),
        expected.month(),
        expected.day(),
        expected.hour(),
        expected.minute()
      )
    );
    assert_eq!(date.millisecond, expected.timestamp_subsec_millis());
  }
}
