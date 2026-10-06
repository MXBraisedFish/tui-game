//! Run independent date conversion checks using only the system clock and local time-zone data.

use tg_core_date::{
  DateError, DateTimeZone, date_to_timestamp, now_timestamp, timestamp_diff, timestamp_to_date,
};

fn main() -> Result<(), DateError> {
  assert_eq!(timestamp_diff(-500, 1000)?, 1500);
  assert_eq!(timestamp_diff(1, 1)?, 0);
  assert_eq!(timestamp_diff(2, 1), Err(DateError::InvalidTimestampOrder));
  assert_eq!(
    timestamp_diff(i64::MIN, i64::MAX),
    Err(DateError::DifferenceOverflow)
  );
  let timezone = DateTimeZone::FixedOffset(8 * 3600);
  let date = timestamp_to_date(123, timezone)?;
  assert_eq!(
    (date.year, date.month, date.day, date.hour),
    (1970, 1, 1, 8)
  );
  assert_eq!(date.millisecond, 123);
  assert_eq!(date_to_timestamp(date, timezone)?, 123);
  assert_eq!(
    timestamp_to_date(i64::MAX, timezone),
    Err(DateError::OutOfRange)
  );
  let timestamp = now_timestamp();
  let local = timestamp_to_date(timestamp, DateTimeZone::Local)?;
  assert!(local.millisecond < 1000);
  assert_eq!(
    date_to_timestamp(
      timestamp_to_date(timestamp, DateTimeZone::Utc)?,
      DateTimeZone::Utc
    )?,
    timestamp
  );
  println!("date ok: millisecond conversion, UTC+8, local clock and invalid input");
  Ok(())
}
