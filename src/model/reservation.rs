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
    let end_time = start_time + 3600;

    let request = InsertReservationRequest {
      seat_id: 1,
      start_time,
      end_time,
    };

    assert!(validate_reservation(&request).is_ok());

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time - 3600;

    let request = InsertReservationRequest {
      seat_id: 1,
      start_time,
      end_time,
    };

    assert!(validate_reservation(&request).is_err());
  }

  #[test]
  fn test_validate_update_reservation() {
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time + 3600;

    let new_start_time = start_time + 7200;
    let new_end_time = new_start_time + 3600;

    let request = UpdateReservationRequest {
      start_time,
      end_time,
      new_start_time,
      new_end_time,
    };

    assert!(validate_update_reservation(&request).is_ok());

    let now = get_now();
    let tomorrow = now.date() + chrono::Duration::days(1);

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time + 3600;

    let new_start_time =
      naive_date_to_timestamp(tomorrow, now.hour(), now.minute(), now.second()).unwrap();
    let new_end_time = new_start_time + 3600;

    let request = UpdateReservationRequest {
      start_time,
      end_time,
      new_start_time,
      new_end_time,
    };

    assert!(validate_update_reservation(&request).is_err());
  }

  #[test]
  fn test_validate_delete_reservation() {
    // valid case
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time + 3600;

    let request = DeleteReservationRequest {
      start_time,
      end_time,
    };

    assert!(validate_delete_reservation(&request).is_ok());

    // invalid case
    let now = get_now();

    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute(), now.second()).unwrap();
    let end_time = start_time - 3600;

    let request = DeleteReservationRequest {
      start_time,
      end_time,
    };

    assert!(validate_delete_reservation(&request).is_err());
  }
}
