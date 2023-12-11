use super::constant::*;
use crate::utils::{get_now, get_today};
use chrono::{Duration, NaiveDate};
use validator::{Validate, ValidationError};

pub fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
  let today = get_today();
  let three_days_later = today + Duration::days(3);

  if *date < today || *date > three_days_later {
    return Err(ValidationError::new("Invalid reservation date"));
  }

  Ok(())
}

pub fn validate_datetime(
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<(), ValidationError> {
  let today = get_today();
  let date: NaiveDate = date;
  let start_time: u32 = start_time;
  let end_time: u32 = end_time;
  let now = get_now();

  if date == today && start_time < now {
    return Err(ValidationError::new(
      "Invalid reservation start time: start time < current time",
    ));
  }

  if end_time < start_time {
    return Err(ValidationError::new(
      "Invalid reservation start time: start time > end time",
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
