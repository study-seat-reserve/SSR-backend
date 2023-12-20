use super::common::*;

pub async fn is_overlapping_with_unavailable_timeslot(
  pool: &Pool<Sqlite>,
  start_time: i64,
  end_time: i64,
) -> Result<bool, Status> {
  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
              SELECT 1 FROM UnavailableTimeSlots
              WHERE (MAX(?, start_time) < MIN(?, end_time))
          )",
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
  time: i64,
) -> Result<bool, Status> {
  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
              SELECT 1 FROM UnavailableTimeSlots
              WHERE start_time <= ? AND end_time > ?
          )",
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

pub async fn insert_unavailable_timeslot(
  pool: &Pool<Sqlite>,
  start_time: i64,
  end_time: i64,
) -> Result<(), Status> {
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  handle_sqlx(
    query("INSERT INTO UnavailableTimeSlots (start_time, end_time) VALUES (?, ?)")
      .bind(start_time)
      .bind(end_time)
      .execute(&mut *tx)
      .await,
    "Inserting new UnavailableTimeSlot information",
  )?;

  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}
