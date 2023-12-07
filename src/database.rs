use crate::{model::*, utils::*};
use chrono::NaiveDate;
use sqlx::{query, query_as, query_scalar, Pool, Sqlite};

pub async fn init_db(pool: &Pool<Sqlite>) {
  log::info!("Initializing db");

  sqlx::query(
    "CREATE TABLE IF NOT EXISTS Seats (
          seat_id INTEGER PRIMARY KEY,
          available BOOLEAN NOT NULL,
          other_info TEXT
      )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create Seats table: {}", e);
    panic!("Failed to create Seats table");
  });

  init_seat_info(&pool).await;

  sqlx::query(
    "CREATE TABLE IF NOT EXISTS Users (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          user_name TEXT NOT NULL UNIQUE,
          password_hash TEXT NOT NULL,
          email TEXT NOT NULL UNIQUE,
          user_role TEXT NOT NULL,
          verified BOOLEAN NOT NULL,
          verification_token TEXT
      )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create Users table: {}", e);
    panic!("Failed to create Users table");
  });

  sqlx::query(
    "CREATE TABLE IF NOT EXISTS Reservations (
          user_name TEXT NOT NULL,
          seat_id INTEGER NOT NULL,
          date TEXT NOT NULL,
          start_time INTEGER NOT NULL,
          end_time INTEGER NOT NULL,
          PRIMARY KEY (user_name, Date),
          FOREIGN KEY(user_name) REFERENCES Users(user_name),
          FOREIGN KEY(seat_id) REFERENCES Seats(seat_id)
      )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create Reservations table: {}", e);
    panic!("Failed to create Reservations table");
  });

  sqlx::query(
    "CREATE TABLE IF NOT EXISTS UnavailableTimeSlots (
          date TEXT NOT NULL,
          start_time INTEGER NOT NULL,
          end_time INTEGER NOT NULL,
          PRIMARY KEY (date, start_time, end_time)
      )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create UnavailableTimeSlots table: {}", e);
    panic!("Failed to create UnavailableTimeSlots table");
  });

  log::info!("Successfully initialized db");
}

async fn init_seat_info(pool: &Pool<Sqlite>) {
  let count: u16 = query_as::<_, (u16,)>("SELECT COUNT(*) FROM Seats")
    .fetch_one(pool)
    .await
    .map(|row| row.0)
    .unwrap_or_else(|e| {
      log::error!("Failed to query Seats table: {}", e);
      panic!("Failed to query Seats table: {}", e);
    });

  if count <= constant::NUMBER_OF_SEATS {
    log::info!("Initializing Seats table");

    for i in (count + 1)..=constant::NUMBER_OF_SEATS {
      query("INSERT INTO Seats (seat_id, available, other_info) VALUES (?1, ?2, ?3)")
        .bind(i)
        .bind(true)
        .bind("")
        .execute(pool)
        .await
        .unwrap_or_else(|e| {
          log::error!("Failed to initialize Seats table: {}", e);
          panic!("Failed to initialize Seats table: {}", e);
        });
    }
  } else {
    log::error!("Failed to initialize Seats table: number of seat in table > NUMBER_OF_SEATS");
    panic!("Failed to initialize Seats table: number of seat in table > NUMBER_OF_SEATS");
  }
}

// 註冊
pub async fn insert_new_user_info(
  pool: &Pool<Sqlite>,
  user_name: &str,
  password_hash: &str,
  email: &str,
  verification_token: &str,
) -> Result<(), Status> {
  log::info!("Inserting new user information");

  let user_role = user::UserRole::RegularUser.to_string();

  handle_sqlx(query("INSERT INTO Users (user_name, password_hash, email, user_role, verified, verification_token) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
      .bind(user_name)
      .bind(password_hash)
      .bind(email)
      .bind(user_role)
      .bind(false)
      .bind(verification_token)
      .execute(pool)
      .await,
"Inserting new user information")?;

  log::info!("Successfully inserted new user information");
  Ok(())
}

pub async fn get_user_info(pool: &Pool<Sqlite>, user_name: &str) -> Result<user::UserInfo, Status> {
  let user_info = handle_sqlx(
    query_as::<_, user::UserInfo>(
      "SELECT user_name, password_hash, email, user_role, verified
       FROM Users
       WHERE user_name = ?",
    )
    .bind(user_name)
    .fetch_one(pool)
    .await,
    "Selecting user info",
  )?;

  Ok(user_info)
}

pub async fn update_user_verified_by_token(
  pool: &Pool<Sqlite>,
  verification_token: &str,
) -> Result<(), Status> {
  let result = handle_sqlx(
    query("UPDATE Users SET verified = true WHERE verification_token = ?")
      .bind(verification_token)
      .execute(pool)
      .await,
    "Updating user verified by verification_token",
  )?;

  let affected_rows = result.rows_affected();

  if affected_rows == 0 {
    log::warn!("No Users found for updation");
    return Err(Status::NotFound);
  }

  Ok(())
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

pub async fn check_if_user_name_exists(
  pool: &Pool<Sqlite>,
  user_name: &str,
) -> Result<bool, Status> {
  let result: Option<(i64,)> = handle_sqlx(
    query_as::<_, (i64,)>("SELECT COUNT(*) FROM Users WHERE user_name = ?")
      .bind(user_name)
      .fetch_optional(pool)
      .await,
    "Querying select operation",
  )?;

  match result {
    Some(_) => {
      log::debug!("username: {} exists", user_name);
      Ok(true)
    }
    None => {
      log::debug!("username: {} does not exist", user_name);
      Ok(false)
    }
  }
}

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

pub async fn is_seat_available(pool: &Pool<Sqlite>, seat_id: u16) -> Result<bool, Status> {
  let available: bool = handle_sqlx(
    query_scalar!("SELECT available FROM Seats WHERE seat_id = ?", seat_id)
      .fetch_one(pool)
      .await,
    "Selecting available from Seats",
  )?;

  Ok(available)
}
