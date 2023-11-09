use super::constant::*;
use crate::utils::*;
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_datetime", skip_on_field_errors = false))]
pub struct Reservation {
  pub user_id: String,
  #[validate(custom = "validate_seat_id")]
  pub seat_id: u16,
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
  pub start_time: u32,
  pub end_time: u32,
}

fn validate_seat_id(seat_id: u16) -> Result<(), ValidationError> {
  if seat_id < 1 || seat_id > NUMBER_OF_SEATS {
    return Err(ValidationError::new("Seat id out of range"));
  }

  Ok(())
}

fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
  let today = get_today();
  let three_days_later = today + Duration::days(3);

  if *date < today || *date > three_days_later {
    return Err(ValidationError::new("Invalid reservation date"));
  }

  Ok(())
}

fn validate_datetime(reservation: &Reservation) -> Result<(), ValidationError> {
  let today = get_today();
  let date: NaiveDate = reservation.date;
  let start_time: u32 = reservation.start_time;
  let end_time: u32 = reservation.end_time;
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
