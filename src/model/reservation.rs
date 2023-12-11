use super::constant::*;
use super::validate_utils::{validate_date, validate_datetime, validate_seat_id};
use crate::utils::{get_now, get_today};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Error, FromRow, Row};
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

impl FromRow<'_, SqliteRow> for Reservation {
  fn from_row(row: &SqliteRow) -> Result<Self, Error> {
    let seat_id_i64: i64 = row.try_get("seat_id")?;
    let seat_id: u16 = seat_id_i64.try_into().map_err(|_| Error::RowNotFound)?;
    let date = row.try_get("date")?;
    let start_time = row.try_get("start_time")?;
    let end_time = row.try_get("end_time")?;

    Ok(Reservation {
      seat_id,
      date,
      start_time,
      end_time,
    })
  }
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
