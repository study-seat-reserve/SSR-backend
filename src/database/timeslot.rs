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
    "Checking if the specified time within any unavailable time slots",
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
    query!(
      "INSERT INTO UnavailableTimeSlots 
        (start_time, end_time) 
      VALUES 
        (datetime(?, 'unixepoch', '+8 hours'), datetime(?, 'unixepoch', '+8 hours'))",
      start_time,
      end_time
    )
    .execute(&mut *tx)
    .await,
    "Inserting new UnavailableTimeSlot information",
  )?;

  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_is_overlapping_with_unavailable_timeslot() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    let start_time = 1672531200; // 2024-01-01 00:00:00
    let end_time = 1672540800; // 2024-01-01 02:00:00

    // Insert overlapping timeslot
    sqlx::query!(
      "INSERT INTO UnavailableTimeSlots (start_time, end_time)
       VALUES (datetime(1640993600, 'unixepoch'), datetime(1641002000, 'unixepoch'))"
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time).await;

    assert!(result.unwrap());

    // Insert non-overlapping timeslot
    sqlx::query!(
      "INSERT INTO UnavailableTimeSlots (start_time, end_time)
       VALUES (datetime(1641007200, 'unixepoch'), datetime(1641010800, 'unixepoch'))"
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time).await;

    assert!(!result.unwrap());
  }

  #[tokio::test]
  async fn test_is_within_unavailable_timeslot() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    let time = 1640995200; // 2022-01-01 01:00:00

    // Insert surrounding timeslot
    sqlx::query!(
      "INSERT INTO UnavailableTimeSlots (start_time, end_time)
       VALUES (datetime(1640993600, 'unixepoch'), datetime(1641002000, 'unixepoch'))"
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_within_unavailable_timeslot(&pool, time).await;

    assert!(result.unwrap());

    // Insert non-surrounding timeslot
    sqlx::query!(
      "INSERT INTO UnavailableTimeSlots (start_time, end_time)  
       VALUES (datetime(1641007200, 'unixepoch'), datetime(1641010800, 'unixepoch'))"
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_within_unavailable_timeslot(&pool, time).await;

    assert!(!result.unwrap());
  }

  #[tokio::test]
  async fn test_insert_unavailable_timeslot() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    let start_time = 1672531200; // 2024-01-01 00:00:00
    let end_time = 1672540800; // 2022-01-01 02:00:00

    insert_unavailable_timeslot(&pool, start_time, end_time)
      .await
      .unwrap();

    let result = sqlx::query!(
        "SELECT start_time, end_time FROM UnavailableTimeSlots WHERE start_time = datetime(?, 'unixepoch') AND end_time = datetime(?, 'unixepoch')",
        start_time,
        end_time
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());

    let record = result.unwrap();
    assert_eq!(record.start_time, start_time);
    assert_eq!(record.end_time, end_time);
  }
}
