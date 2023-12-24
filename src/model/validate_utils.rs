use super::{common::*, constant::*};
use crate::utils::{get_now, naive_datetime_to_timestamp, timestamp_to_naive_datetime};

pub fn validate_datetime(start_time: i64, end_time: i64) -> Result<(), ValidationError> {
  on_the_same_day(start_time, end_time)?;

  let now = get_now(); // 本地時間
  let current_timestamp: i64 =
    naive_datetime_to_timestamp(now).expect("Failed to convert naive datetime to timestamp");

  if start_time < current_timestamp {
    return Err(ValidationError::new(
      "Invalid reservation: Start time is greater than the current time",
    ));
  }

  if end_time < start_time {
    return Err(ValidationError::new(
      "Invalid reservation: start time: Start time is greater than end time",
    ));
  }

  Ok(())
}

pub fn on_the_same_day(time1: i64, time2: i64) -> Result<(), ValidationError> {
  let datetime1 = timestamp_to_naive_datetime(time1).expect("Invalid start_time timestamp");
  let datetime2 = timestamp_to_naive_datetime(time2).expect("Invalid end_time timestamp");

  if datetime1.date() != datetime2.date() {
    return Err(ValidationError::new(
      "Invalid reservation: The two dates are not on the same day",
    ));
  }

  Ok(())
}

pub fn validate_seat_id(seat_id: u16) -> Result<(), ValidationError> {
  if seat_id < 1 || seat_id > NUMBER_OF_SEATS {
    return Err(ValidationError::new("Seat id out of range"));
  }

  Ok(())
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::utils::{get_now, naive_date_to_timestamp};
  use chrono::Timelike;

  #[test]
  fn test_validate_datetime_success() {
    let now = get_now();
    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time + 3600;

    assert!(validate_datetime(start_time, end_time).is_ok());

    let now = get_now();
    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time - 3600;

    assert!(validate_datetime(start_time, end_time).is_err());

    let now = get_now();
    let tomorrow = now.date() + chrono::Duration::days(1);

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(tomorrow, now.hour(), now.minute(), now.second()).unwrap();

    assert!(validate_datetime(start_time, end_time).is_err());
  }

  #[test]
  fn test_on_the_same_day() {
    // today
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time + 3600;

    assert!(on_the_same_day(start_time, end_time).is_ok());
    // not today
    let now = get_now();
    let tomorrow = now.date() + chrono::Duration::days(1);

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(tomorrow, now.hour(), now.minute(), now.second()).unwrap();

    assert!(on_the_same_day(start_time, end_time).is_err());
  }

  #[test]
  fn test_validate_seat_id() {
    let seat_id = 100;
    assert!(validate_seat_id(seat_id).is_ok());

    let seat_id = 0;
    assert!(validate_seat_id(seat_id).is_err());

    let seat_id = 218;
    assert!(validate_seat_id(seat_id).is_err());
  }
}
