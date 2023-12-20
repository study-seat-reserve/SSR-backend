use super::constant::*;
use crate::utils::{get_now, get_today};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_reservation", skip_on_field_errors = false))]
pub struct Reservation {
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
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
  pub new_start_time: u32,
  pub new_end_time: u32,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct DeleteReservation {
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
//TEST

#[cfg(test)]
mod test {
  use super::*;
  use chrono::Local;
  use validator::ValidationError;

  #[test]
  fn test_validate_seat_id() {
    assert!(validate_seat_id(0).is_err());
    assert!(validate_seat_id(109).is_ok());
    assert!(validate_seat_id(218).is_err());

    assert_eq!(
      validate_seat_id(0).err().unwrap(),
      ValidationError::new("Seat id out of range")
    );

    assert_eq!(
      validate_seat_id(218).err().unwrap(),
      ValidationError::new("Seat id out of range")
    );
  }

  #[test]
  fn test_validate_date() {
    // Valid
    let valid_date = Local::now().date_naive() + chrono::Duration::days(2);
    assert!(validate_date(&valid_date).is_ok());
    // Invalid
    let invalid_date = Local::now().date_naive() - chrono::Duration::days(1);
    assert!(validate_date(&invalid_date).is_err());

    let invalid_date = Local::now().date_naive() + chrono::Duration::days(4);
    assert!(validate_date(&invalid_date).is_err());

    assert_eq!(
      validate_date(&invalid_date).err().unwrap(),
      ValidationError::new("Invalid reservation date")
    );

    assert_eq!(
      validate_date(&invalid_date).err().unwrap(),
      ValidationError::new("Invalid reservation date")
    );
  }

  #[test]
  fn test_validate_datetime() {
    // end_time > start_time
    let result: Result<(), ValidationError> = validate_datetime(
      get_today() + Duration::days(1),
      get_now() + 1,
      get_now() + 2,
    );
    assert!(result.is_ok());

    // end_time < start_time
    let result = validate_datetime(
      get_today() + Duration::days(1),
      get_now() + 2,
      get_now() + 1,
    );
    assert!(result.is_err());

    // date == today && start_time < now
    let result = validate_datetime(get_today(), get_now() - 1, get_now() + 1);
    assert!(result.is_err());

    // date != today && start_time > now
    let result = validate_datetime(
      get_today() + Duration::days(1),
      get_now() + 1,
      get_now() + 2,
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_update_reservation() {
    let valid_update = UpdateReservation {
      date: get_today() + Duration::days(1),
      new_start_time: get_now() + 1,
      new_end_time: get_now() + 2,
    };
    assert!(validate_update_reservation(&valid_update).is_ok());

    // end_time < start_time
    let invalid_update = UpdateReservation {
      date: get_today(),
      new_start_time: get_now() + 2,
      new_end_time: get_now() + 1,
    };
    assert!(validate_update_reservation(&invalid_update).is_err());

    // date == today && new_start_time < now
    let invalid_update = UpdateReservation {
      date: get_today(),
      new_start_time: get_now() - 1,
      new_end_time: get_now() + 1,
    };
    assert!(validate_update_reservation(&invalid_update).is_err());
  }

  #[test]
  fn test_validate_reservation() {
    let valid_reservation = Reservation {
      seat_id: 1,
      date: get_today() + Duration::days(1),
      start_time: get_now() + 1,
      end_time: get_now() + 2,
    };
    assert!(validate_reservation(&valid_reservation).is_ok());

    // end_time < start_time
    let invalid_reservation = Reservation {
      seat_id: 1,
      date: get_today(),
      start_time: get_now() + 2,
      end_time: get_now() + 1,
    };
    assert!(validate_reservation(&invalid_reservation).is_err());

    // date == today && start_time < now
    let invalid_reservation = Reservation {
      seat_id: 1,
      date: get_today(),
      start_time: get_now() - 1,
      end_time: get_now() + 1,
    };
    assert!(validate_reservation(&invalid_reservation).is_err());
  }
}
