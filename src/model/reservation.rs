use super::{common::*, validate_utils::*};

#[derive(Debug, Deserialize, Serialize)]
pub struct Reservation {
  pub seat_id: u16,
  pub start_time: i64,
  pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_reservation", skip_on_field_errors = false))]
pub struct InsertReservationRequest {
  #[validate(custom = "validate_seat_id")]
  pub seat_id: u16,
  pub start_time: i64,
  pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_update_reservation", skip_on_field_errors = false))]
pub struct UpdateReservationRequest {
  pub start_time: i64,
  pub end_time: i64,
  pub new_start_time: i64,
  pub new_end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_delete_reservation", skip_on_field_errors = false))]
pub struct DeleteReservationRequest {
  pub start_time: i64,
  pub end_time: i64,
}

fn validate_reservation(request: &InsertReservationRequest) -> Result<(), ValidationError> {
  let start_time: i64 = request.start_time;
  let end_time: i64 = request.end_time;

  validate_datetime(start_time, end_time)
}

impl FromRow<'_, SqliteRow> for Reservation {
  fn from_row(row: &SqliteRow) -> Result<Self, Error> {
    let seat_id_i64: i64 = row.try_get("seat_id")?;
    let seat_id: u16 = seat_id_i64.try_into().map_err(|_| Error::RowNotFound)?;

    let start_time: i64 = row.try_get("start_time")?;
    let end_time: i64 = row.try_get("end_time")?;

    Ok(Reservation {
      seat_id,
      start_time,
      end_time,
    })
  }
}

fn validate_update_reservation(request: &UpdateReservationRequest) -> Result<(), ValidationError> {
  let start_time = request.start_time;
  let end_time = request.end_time;
  let new_start_time = request.new_start_time;
  let new_end_time = request.new_end_time;

  validate_datetime(start_time, end_time)?;
  validate_datetime(new_start_time, new_end_time)?;
  on_the_same_day(start_time, new_start_time)
}

fn validate_delete_reservation(request: &DeleteReservationRequest) -> Result<(), ValidationError> {
  let start_time = request.start_time;
  let end_time = request.end_time;

  validate_datetime(start_time, end_time)
}
