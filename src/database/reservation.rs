use crate::{model::*, utils::*};
use sqlx::{query, query_as, query_scalar, Pool, Sqlite};

// 預約座位
pub async fn reserve_seat(
  pool: &Pool<Sqlite>,
  user_name: &str,
  seat_id: u16,
  start_time: i64,
  end_time: i64,
) -> Result<(), Status> {
  // 使用transaction
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  // 查詢時間段是否重疊
  let result: Option<i32> = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
            SELECT 1 FROM Reservations
            WHERE seat_id = ?
            AND  (MAX(?, start_time) < MIN(?, end_time))
        )",
      seat_id,
      start_time,
      end_time
    )
    .fetch_one(&mut *tx)
    .await,
    "Selecting overlapping reservations",
  )?;
  let is_overlapping: bool = result.map_or(false, |count| count != 0);

  // 如果重疊
  if is_overlapping {
    log::warn!("Found overlapping reservation");

    // rollback
    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::Conflict);
  }

  /*
  判斷當天否有還沒完成的預約
   */

  // 新增一筆預約
  handle_sqlx(
    query!(
      "INSERT INTO Reservations (user_name, seat_id, start_time, end_time) VALUES (?, ?, ?, ?)",
      user_name,
      seat_id,
      start_time,
      end_time
    )
    .execute(&mut *tx)
    .await,
    "Inserting new Reservation information",
  )?;

  // 完成整筆transaction
  handle_sqlx(tx.commit().await, "Committing transaction")?;

  Ok(())
}

// 修改預約紀錄
pub async fn update_reservation_time(
  pool: &Pool<Sqlite>,
  user_name: &str,
  start_time: i64,
  end_time: i64,
  new_start_time: i64,
  new_end_time: i64,
) -> Result<(), Status> {
  // 使用transaction
  let mut tx = handle_sqlx(pool.begin().await, "Starting new transaction")?;

  // 查詢時間段是否重疊
  let result: Option<i32> = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
            SELECT 1 FROM Reservations
            WHERE user_name != ?
            AND  (MAX(?, start_time) < MIN(?, end_time))
        )",
      user_name,
      new_start_time,
      new_end_time
    )
    .fetch_one(&mut *tx)
    .await,
    "Checking for overlapping reservations",
  )?;
  let is_overlapping: bool = result.map_or(false, |count| count != 0);

  // 如果重疊
  if is_overlapping {
    log::warn!("Found overlapping reservation");

    // rollback
    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::Conflict);
  }

  // 執行更新
  let affected_rows = handle_sqlx(
    query!(
      "UPDATE Reservations SET start_time = ?, end_time = ? WHERE user_name = ? AND start_time = ? AND end_time = ?",
      new_start_time,
      new_end_time,
      user_name,
      start_time, end_time,
    )
    .execute(&mut *tx)
    .await,
    "Updating reservation",
  )?
  .rows_affected();

  // 檢查是否有成功更新
  // affected_rows == 0，此次操作無作用到任何資料
  if affected_rows == 0 {
    log::warn!("No reservation found for updation");

    // rollback
    handle_sqlx(tx.rollback().await, "Rolling back")?;
    return Err(Status::NotFound);
  } else {
    // 完成整筆transaction
    handle_sqlx(tx.commit().await, "Committing transaction")?;
  }

  Ok(())
}

pub async fn delete_reservation_time(
  pool: &Pool<Sqlite>,
  user_name: &str,
  start_time: i64,
  end_time: i64,
) -> Result<(), Status> {
  /*
  刪除預約紀錄
   */

  // 執行刪除
  let affected_rows = handle_sqlx(
    query!(
      "DELETE FROM Reservations WHERE user_name = ? AND start_time = ? AND end_time = ?",
      user_name,
      start_time,
      end_time,
    )
    .execute(pool)
    .await,
    "Deleting reservation",
  )?
  .rows_affected();

  // 檢查是否有成功更新
  // affected_rows == 0，此次操作無作用到任何資料
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
  /*
  獲取使用者的預約紀錄
   */
  let now = naive_datetime_to_timestamp(get_now())?;

  // 搜尋使用者今天之後的預約紀錄
  let reservations = handle_sqlx(
    query_as::<_, reservation::Reservation>(
      "SELECT seat_id, start_time, end_time FROM Reservations WHERE user_name = ? AND end_time > ?",
    )
    .bind(user_name)
    .bind(now)
    .fetch_all(pool)
    .await,
    "Selecting reservations",
  )?;

  Ok(reservations)
}
