use super::constant::*;
use crate::utils::{get_now, get_today};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_reservation", skip_on_field_errors = false))]
pub struct Reservation {
  pub user_id: String,
  #[validate(custom = "validate_seat_id")]
  pub seat_id: u16,
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
  pub start_time: u32,
  pub end_time: u32,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_update_reservation", skip_on_field_errors = false))]
pub struct UpdateReservation {
  pub user_id: String,
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
  pub new_start_time: u32,
  pub new_end_time: u32,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct DeleteReservation {
  pub user_id: String,
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
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

fn validate_datetime(
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

fn validate_update_reservation(
  update_reservation: &UpdateReservation,
) -> Result<(), ValidationError> {
  let date: NaiveDate = update_reservation.date;
  let start_time: u32 = update_reservation.new_start_time;
  let end_time: u32 = update_reservation.new_end_time;

  validate_datetime(date, start_time, end_time)
}

fn validate_reservation(reservation: &Reservation) -> Result<(), ValidationError> {
  let date: NaiveDate = reservation.date;
  let start_time: u32 = reservation.start_time;
  let end_time: u32 = reservation.end_time;

  validate_datetime(date, start_time, end_time)
}
