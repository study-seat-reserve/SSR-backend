use crate::utils::*;
use chrono::NaiveDate;
use sqlx::{query, query_scalar, Pool, Sqlite};

pub async fn is_overlapping_with_unavailable_timeslot(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<bool, Status> {
  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
              SELECT 1 FROM UnavailableTimeSlots
              WHERE date = ?
              AND (MAX(?, start_time) < MIN(?, end_time))
          )",
      date,
      start_time,
      end_time
    )
    .fetch_one(pool)
    .await,
    "Selecting overlapping unavailable time slots",
  )?;

  let is_overlapping: bool = result.map_or(false, |count| count != 0);

  Ok(is_overlapping)
}

pub async fn is_within_unavailable_timeslot(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  time: u32,
) -> Result<bool, Status> {
  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
              SELECT 1 FROM UnavailableTimeSlots
              WHERE date = ?
              AND start_time <= ? AND end_time > ?
          )",
      date,
      time,
      time
    )
    .fetch_one(pool)
    .await,
    "Selecting overlapping unavailable time slots",
  )?;

  let is_within_timeslot: bool = result.map_or(false, |count| count != 0);

  Ok(is_within_timeslot)
}

pub async fn insert_unavailable_timeslots(
  pool: &Pool<Sqlite>,
  date: NaiveDate,
  timelosts: Vec<(u32, u32)>,
) -> Result<(), Status> {
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  for (start_time, end_time) in timelosts {
    handle_sqlx(
      query("INSERT INTO UnavailableTimeSlots (date, start_time, end_time) VALUES (?, ?, ?)")
        .bind(date)
        .bind(start_time)
        .bind(end_time)
        .execute(&mut *tx)
        .await,
      "Inserting new UnavailableTimeSlot information",
    )?;
  }

  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}
