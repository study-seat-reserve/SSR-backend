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
        WHERE 
          (MAX(datetime(?, 'unixepoch', '+8 hours'), start_time) < MIN(datetime(?, 'unixepoch', '+8 hours'), end_time))
      )",
      start_time,
      end_time
    )
    .fetch_one(pool)
    .await,
    "Selecting overlapping unavailable time slots",
  )?;

  let overlapping: bool = result.map_or(false, |count| count != 0);

  Ok(overlapping)
}

pub async fn is_within_unavailable_timeslot(
  pool: &Pool<Sqlite>,
  time: i64,
) -> Result<bool, Status> {
  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
        SELECT 1 FROM UnavailableTimeSlots
        WHERE 
          start_time <= datetime(?, 'unixepoch', '+8 hours') AND 
          end_time > datetime(?, 'unixepoch', '+8 hours')
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

  let sql = "
    INSERT INTO UnavailableTimeSlots 
      (start_time, end_time) 
    VALUES 
      (datetime(?, 'unixepoch', '+8 hours'), datetime(?, 'unixepoch', '+8 hours'))";

  handle_sqlx(
    query(sql)
      .bind(start_time)
      .bind(end_time)
      .execute(&mut *tx)
      .await,
    "Inserting new UnavailableTimeSlot information",
  )?;

  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}
