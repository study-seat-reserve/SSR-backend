use super::common::*;

// 查詢所有位置在特定時間點狀態
pub async fn get_all_seats_status(
  pool: &Pool<Sqlite>,
  time: i64,
) -> Result<seat::AllSeatsStatus, Status> {
  /*
  查詢所有位置在特定時間點狀態
  */

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
      Reservations.start_time <= datetime(?, 'unixepoch', '+8 hours') AND
      Reservations.end_time > datetime(?, 'unixepoch', '+8 hours')";

  // 取得每個座位的狀態，回傳為vector包含(座位號碼, 狀態)
  let result: Vec<(u16, String)> = handle_sqlx(
    sqlx::query_as::<_, (u16, String)>(sql)
      .bind(time)
      .bind(time)
      .fetch_all(pool)
      .await,
    "Selecting all seats status",
  )?;

  let mut seats = Vec::new();

  // 將狀態由String轉換為seat::Status
  for (seat_id, status_str) in result {
    let status = match status_str.as_str() {
      "Available" => seat::Status::Available,
      "Borrowed" => seat::Status::Borrowed,
      "Unavailable" => seat::Status::Unavailable,
      _ => return Err(Status::InternalServerError),
    };
    seats.push(seat::SeatStatus { seat_id, status });
  }

  // 建立seat::AllSeatsStatus回傳
  let all_seats_status = seat::AllSeatsStatus { seats: seats };

  Ok(all_seats_status)
}

pub async fn get_seats_status_in_specific_timeslots(
  pool: &Pool<Sqlite>,
  start_time: i64,
  end_time: i64,
) -> Result<seat::AllSeatsStatus, Status> {
  /*
  查詢特定時間段中位置是否被借用
   */
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
      (MAX(datetime(?, 'unixepoch', '+8 hours'), start_time) < MIN(datetime(?, 'unixepoch', '+8 hours'), end_time))";

  let result: Vec<(u16, String)> = handle_sqlx(
    sqlx::query_as::<_, (u16, String)>(sql)
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
  start_time: i64,
  end_time: i64,
  seat_id: u16,
) -> Result<Vec<(i64, i64)>, Status> {
  let sql = "
    SELECT
      CAST(strftime('%s', start_time, '-8 hours') AS INTEGER) as start_time, 
      CAST(strftime('%s', end_time, '-8 hours') AS INTEGER) as end_time
    FROM 
      Reservations
    WHERE
      seat_id = ? AND 
      start_time >= datetime(?, 'unixepoch', '+8 hours') AND 
      end_time <= datetime(?, 'unixepoch', '+8 hours')";

  let timeslots: Vec<(i64, i64)> = handle_sqlx(
    sqlx::query_as::<_, (i64, i64)>(sql)
      .bind(seat_id)
      .bind(start_time)
      .bind(end_time)
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

pub async fn update_seat_availability(
  pool: &Pool<Sqlite>,
  seat_id: u16,
  available: bool,
) -> Result<(), Status> {
  let affected_rows = handle_sqlx(
    query!(
      "UPDATE Seats
      SET
        available = ?
      WHERE 
        seat_id = ? ",
      available,
      seat_id,
    )
    .execute(pool)
    .await,
    "Updating seat",
  )?
  .rows_affected();

  // 檢查是否有成功更新
  // affected_rows == 0，此次操作無作用到任何資料
  if affected_rows == 0 {
    log::warn!("No seat found for updation");

    return Err(Status::NotFound);
  }

  Ok(())
}
