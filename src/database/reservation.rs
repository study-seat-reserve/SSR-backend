use crate::{model::*, utils::*};
use chrono::NaiveDate;
use sqlx::{query, query_as, query_scalar, Pool, Sqlite};

// 預約座位
pub async fn reserve_seat(
  pool: &Pool<Sqlite>,
  user_name: &str,
  seat_id: u16,
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<(), Status> {
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  let result: Option<i32> = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
            SELECT 1 FROM Reservations
            WHERE seat_id = ? AND date = ?
            AND  (MAX(?, start_time) < MIN(?, end_time))
        )",
      seat_id,
      date,
      start_time,
      end_time
    )
    .fetch_one(&mut *tx)
    .await,
    "Selecting overlapping reservations",
  )?;

  let is_overlapping: bool = result.map_or(false, |count| count != 0);

  if is_overlapping {
    log::warn!("Found overlapping reservation");

    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::Conflict);
  }

  handle_sqlx(query!(
        "INSERT INTO Reservations (user_name, seat_id, date, start_time, end_time) VALUES (?, ?, ?, ?, ?)",
        user_name, seat_id, date, start_time, end_time
    )
    .execute(&mut *tx)
    .await
    , "Inserting new Reservation information")?;

  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}

pub async fn update_reservation_time(
  pool: &Pool<Sqlite>,
  user_name: &str,
  date: NaiveDate,
  new_start_time: u32,
  new_end_time: u32,
) -> Result<(), Status> {
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  let result: Option<i32> = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
            SELECT 1 FROM Reservations
            WHERE user_name != ?
            AND date = ?
            AND  (MAX(?, start_time) < MIN(?, end_time))
        )",
      user_name,
      date,
      new_start_time,
      new_end_time
    )
    .fetch_one(&mut *tx)
    .await,
    "Checking for overlapping reservations",
  )?;

  let is_overlapping: bool = result.map_or(false, |count| count != 0);

  if is_overlapping {
    log::warn!("Found overlapping reservation");

    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::Conflict);
  }

  let affected_rows = handle_sqlx(
    query!(
      "UPDATE Reservations SET start_time = ?, end_time = ? WHERE user_name = ? AND date = ?",
      new_start_time,
      new_end_time,
      user_name,
      date
    )
    .execute(&mut *tx)
    .await,
    "Updating reservation",
  )?
  .rows_affected();

  if affected_rows == 0 {
    log::warn!("No reservation found for updation");

    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::NotFound);
  } else {
    handle_sqlx(tx.commit().await, "Committing transaction")?;
  }

  Ok(())
}

pub async fn delete_reservation_time(
  pool: &Pool<Sqlite>,
  user_name: &str,
  date: NaiveDate,
) -> Result<(), Status> {
  let affected_rows = handle_sqlx(
    query!(
      "DELETE FROM Reservations WHERE user_name = ? AND date = ?",
      user_name,
      date
    )
    .execute(pool)
    .await,
    "Deleting reservation",
  )?
  .rows_affected();

  if affected_rows == 0 {
    log::warn!("No reservation found for deletion");

    return Err(Status::NotFound);
  }

  Ok(())
}

pub async fn get_user_reservations(
  pool: &Pool<Sqlite>,
  user_name: &str,
) -> Result<Vec<reservation::Reservation>, Status> {
  let date = get_today();

  let reservations = handle_sqlx(query_as::<_, reservation::Reservation>(
      "SELECT seat_id, date, start_time, end_time FROM Reservations WHERE user_name = ? AND date >= ?",
    )
    .bind(user_name)
    .bind(date)
    .fetch_all(pool)
    .await, "Selecting reservations")?;

  Ok(reservations)
}
