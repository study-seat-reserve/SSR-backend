use crate::{model::*, utils::*};
use chrono::NaiveDate;
use sqlx::{query_scalar, Pool, Sqlite};

// 查詢所有位置在特定時間點狀態
pub async fn get_all_seats_status(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  time: u32,
) -> Result<seat::AllSeatsStatus, Status> {
  let sql = "
          SELECT 
              Seats.seat_id,
              CASE
                  WHEN Seats.available = 0 THEN 'Unavailable'
                  WHEN Reservations.seat_id IS NULL THEN 'Available'
                  ELSE 'Borrowed'
              END as status
          FROM 
              Seats
          LEFT JOIN Reservations ON 
              Seats.seat_id = Reservations.seat_id AND
              Reservations.date = ? AND
              Reservations.start_time <= ? AND
              Reservations.end_time > ?
      ";

  let result = handle_sqlx(
    sqlx::query_as::<_, (u16, String)>(sql)
      .bind(date)
      .bind(time)
      .bind(time)
      .fetch_all(pool)
      .await,
    "Selecting all seats status",
  )?;

  let mut seats = Vec::new();

  for (seat_id, status_str) in result {
    let status = match status_str.as_str() {
      "Available" => seat::Status::Available,
      "Borrowed" => seat::Status::Borrowed,
      "Unavailable" => seat::Status::Unavailable,
      _ => return Err(Status::InternalServerError),
    };
    seats.push(seat::SeatStatus { seat_id, status });
  }

  let all_seats_status = seat::AllSeatsStatus { seats: seats };

  Ok(all_seats_status)
}

pub async fn get_seats_status_by_time(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<seat::AllSeatsStatus, Status> {
  let sql = "
        SELECT DISTINCT 
            Seats.seat_id,
            CASE
                WHEN Seats.available = 0 THEN 'Unavailable'
                WHEN Reservations.seat_id IS NULL THEN 'Available'
                ELSE 'Borrowed'
            END as status
        FROM 
            Seats
        LEFT JOIN Reservations ON 
            Seats.seat_id = Reservations.seat_id AND
            Reservations.date = ? AND
            (MAX(?, start_time) < MIN(?, end_time))
    ";

  let result = handle_sqlx(
    sqlx::query_as::<_, (u16, String)>(sql)
      .bind(date)
      .bind(start_time)
      .bind(end_time)
      .fetch_all(pool)
      .await,
    "Selecting seat status by time",
  )?;

  let mut seats = Vec::new();

  for (seat_id, status_str) in result {
    let status = match status_str.as_str() {
      "Available" => seat::Status::Available,
      "Borrowed" => seat::Status::Borrowed,
      "Unavailable" => seat::Status::Unavailable,
      _ => return Err(Status::InternalServerError),
    };
    seats.push(seat::SeatStatus { seat_id, status });
  }

  let all_seats_status = seat::AllSeatsStatus { seats };

  Ok(all_seats_status)
}

// 查詢特定位置狀態
pub async fn get_seat_reservations(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  seat_id: u16,
) -> Result<Vec<(u32, u32)>, Status> {
  let sql = "
        SELECT
            start_time, end_time
        FROM 
            Reservations
        WHERE
            date = ? AND seat_id = ?
    ";

  let timeslots = handle_sqlx(
    sqlx::query_as::<_, (u32, u32)>(sql)
      .bind(date)
      .bind(seat_id)
      .fetch_all(pool)
      .await,
    "Selecting reservations for a seat",
  )?;

  Ok(timeslots)
}

pub async fn is_seat_available(pool: &Pool<Sqlite>, seat_id: u16) -> Result<bool, Status> {
  let available: bool = handle_sqlx(
    query_scalar!("SELECT available FROM Seats WHERE seat_id = ?", seat_id)
      .fetch_one(pool)
      .await,
    "Selecting available from Seats",
  )?;

  Ok(available)
}
