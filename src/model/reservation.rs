use super::constant::*;
use chrono::{Duration, Local, NaiveDate, NaiveTime};
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
  pub start_time: NaiveTime,
  #[validate(custom = "validate_end_time")]
  pub end_time: NaiveTime,
}

fn validate_seat_id(seat_id: u16) -> Result<(), ValidationError> {
  if seat_id < 1 || seat_id > NUMBER_OF_SEATS {
    return Err(ValidationError::new("Seat id out of range"));
  }

  Ok(())
}

fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
  let today = Local::now().date_naive();
  let three_days_later = today + Duration::days(3);

  if *date < today || *date > three_days_later {
    return Err(ValidationError::new("Invalid reservation date"));
  }

  Ok(())
}

fn validate_end_time(end_time: &NaiveTime) -> Result<(), ValidationError> {
  println!("end time: {:?}", end_time);
  println!("test: {:?}", NaiveTime::from_hms_opt(24, 0, 0));

  Ok(())
}

fn validate_datetime(reservation: &Reservation) -> Result<(), ValidationError> {
  let today = Local::now().date_naive();
  let date: NaiveDate = reservation.date;
  let start_time: NaiveTime = reservation.start_time;
  let end_time: NaiveTime = reservation.end_time;
  let now_time = Local::now().naive_local().time();

  if date == today && start_time < now_time {
    return Err(ValidationError::new("Invalid reservation start time"));
  }

  if end_time < start_time {
    return Err(ValidationError::new("Invalid reservation start time"));
  }

  Ok(())
}
