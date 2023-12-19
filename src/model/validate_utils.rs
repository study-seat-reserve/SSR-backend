use super::constant::*;
use crate::utils::{get_now, get_today, timestamp_to_naive_datetime};
use validator::ValidationError;

pub fn validate_datetime(start_time: i64, end_time: i64) -> Result<(), ValidationError> {
  on_the_same_day(start_time, end_time)?;

  let current_timestamp = get_now().timestamp();

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