use super::{common::*, validate_utils::*};

#[derive(Debug, Deserialize, Serialize)]
pub struct Reservation {
  pub seat_id: u16,
  pub start_time: i64,
  pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(
  function = "validate_reservation_request",
  skip_on_field_errors = false
))]
pub struct InsertReservationRequest {
  #[validate(custom = "validate_seat_id")]
  pub seat_id: u16,
  pub start_time: i64,
  pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(
  function = "validate_update_reservation_request",
  skip_on_field_errors = false
))]
pub struct UpdateReservationRequest {
  pub start_time: i64,
  pub end_time: i64,
  pub new_start_time: i64,
  pub new_end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(
  function = "validate_delete_reservation_request",
  skip_on_field_errors = false
))]
pub struct DeleteReservationRequest {
  pub start_time: i64,
  pub end_time: i64,
}

fn validate_reservation_request(request: &InsertReservationRequest) -> Result<(), ValidationError> {
  let start_time: i64 = request.start_time;
  let end_time: i64 = request.end_time;

  validate_datetime(start_time, end_time)
}

impl FromRow<'_, SqliteRow> for Reservation {
  fn from_row(row: &SqliteRow) -> Result<Self, Error> {
    let seat_id_i64: i64 = row.try_get("seat_id")?;
    let seat_id: u16 = seat_id_i64.try_into().map_err(|_| Error::RowNotFound)?;

    let start_time_str: String = row.try_get("start_time")?;
    let end_time_str: String = row.try_get("end_time")?;

    let start_time: i64 = start_time_str
      .parse()
      .map_err(|source| Error::ColumnDecode {
        index: "start_time".to_string(),
        source: Box::new(source),
      })?;

    let end_time: i64 = end_time_str.parse().map_err(|source| Error::ColumnDecode {
      index: "end_time".to_string(),
      source: Box::new(source),
    })?;

    Ok(Reservation {
      seat_id,
      start_time,
      end_time,
    })
  }
}

fn validate_update_reservation_request(
  request: &UpdateReservationRequest,
) -> Result<(), ValidationError> {
  let start_time = request.start_time;
  let end_time = request.end_time;
  let new_start_time = request.new_start_time;
  let new_end_time = request.new_end_time;

  validate_datetime(start_time, end_time)?;
  validate_datetime(new_start_time, new_end_time)?;
  on_the_same_day(start_time, new_start_time)
}

fn validate_delete_reservation_request(
  request: &DeleteReservationRequest,
) -> Result<(), ValidationError> {
  let start_time = request.start_time;
  let end_time = request.end_time;

  validate_datetime(start_time, end_time)
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::utils::{get_now, naive_date_to_timestamp};
  use chrono::Timelike;

  #[test]
  fn test_validate_reservation() {
    let now = get_now();
    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() + 1, now.minute(), now.second()).unwrap();

    let request = InsertReservationRequest {
      seat_id: 109,
      start_time,
      end_time,
    };

    assert!(validate_reservation_request(&request).is_ok());

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() - 1, now.minute(), now.second()).unwrap();

    let request = InsertReservationRequest {
      seat_id: 109,
      start_time,
      end_time,
    };

    assert!(validate_reservation_request(&request).is_err());
  }

  #[test]
  fn test_validate_update_reservation() {
    // Valid
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() + 1, now.minute(), now.second()).unwrap();

    let new_start_time =
      naive_date_to_timestamp(now.date(), now.hour() + 2, now.minute(), now.second()).unwrap();
    let new_end_time =
      naive_date_to_timestamp(now.date(), now.hour() + 3, now.minute(), now.second()).unwrap();

    let request = UpdateReservationRequest {
      start_time,
      end_time,
      new_start_time,
      new_end_time,
    };

    assert!(validate_update_reservation_request(&request).is_ok());

    // Invalid
    let now = get_now();
    let tomorrow = now.date() + chrono::Duration::days(1);

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() + 1, now.minute(), now.second()).unwrap();

    let new_start_time =
      naive_date_to_timestamp(tomorrow, now.hour(), now.minute(), now.second()).unwrap();
    let new_end_time =
      naive_date_to_timestamp(tomorrow, now.hour() + 1, now.minute(), now.second()).unwrap();

    let request = UpdateReservationRequest {
      start_time,
      end_time,
      new_start_time,
      new_end_time,
    };

    assert!(validate_update_reservation_request(&request).is_err());
  }

  #[test]
  fn test_validate_delete_reservation() {
    // Valid
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() + 1, now.minute(), now.second()).unwrap();

    let request = DeleteReservationRequest {
      start_time,
      end_time,
    };

    assert!(validate_delete_reservation_request(&request).is_ok());

    // Invalid
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time =
      naive_date_to_timestamp(now.date(), now.hour() - 1, now.minute(), now.second()).unwrap();

    let request = DeleteReservationRequest {
      start_time,
      end_time,
    };

    assert!(validate_delete_reservation_request(&request).is_err());
  }
}
