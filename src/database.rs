use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};

use crate::{model::*, utils::*};

fn connect_to_db() -> Result<Connection> {
  log::debug!("Connecting to db");

  let path = format!("{}/SSR.db3", get_root());
  log::debug!("path={}", path);

  Connection::open(path)
}

// 初始化
pub fn init_db() {
  log::info!("Initializing db");

  let path: String = format!("{}/SSR.db3", get_root());
  log::debug!("path={}", path);

  let conn = Connection::open(path).unwrap_or_else(|e| {
    log::error!("Failed to open SQLite connection: {}", e);
    panic!("Failed to open SQLite connection");
  });

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS Seats (
         seat_id INTEGER PRIMARY KEY,
         available BOOLEAN NOT NULL,
         other_info TEXT
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create Seats table: {}", e);
      panic!("Failed to create Seats table");
    });

  init_seat_info(&conn);

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS Users (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         user_name TEXT NOT NULL UNIQUE,
         password_hash TEXT NOT NULL,
         email TEXT NOT NULL UNIQUE,
         user_role TEXT NOT NULL,
         verified BOOLEAN NOT NULL,
         verification_token TEXT
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create Users table: {}", e);
      panic!("Failed to create Users table");
    });

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS Reservations (
         user_id TEXT NOT NULL,
         seat_id INTEGER NOT NULL,
         date TEXT NOT NULL,
         start_time INTEGER NOT NULL,
         end_time INTEGER NOT NULL,
         PRIMARY KEY (user_id, Date),
         FOREIGN KEY(user_id) REFERENCES Users(user_id),
         FOREIGN KEY(seat_id) REFERENCES Seats(seat_id)
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create Reservations table: {}", e);
      panic!("Failed to create Reservations table");
    });

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS UnavailableTimeSlots (
         date TEXT NOT NULL,
         start_time INTEGER NOT NULL,
         end_time INTEGER NOT NULL,
         PRIMARY KEY (date, start_time, end_time)
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create UnavailableTimeSlots table: {}", e);
      panic!("Failed to create UnavailableTimeSlots table");
    });

  log::info!("successfully initialized db");
}

fn init_seat_info(conn: &Connection) {
  let count: u16 = conn
    .query_row("SELECT COUNT(*) FROM Seats", [], |row| row.get(0))
    .unwrap_or_else(|e| {
      log::error!("Failed to create TimeSlots table: {}", e);
      panic!("Failed to create TimeSlots table: {}", e);
    });

  if count <= constant::NUMBER_OF_SEATS {
    log::info!("Initializing Seats table");

    for i in (count + 1)..=constant::NUMBER_OF_SEATS {
      conn
        .execute(
          "INSERT INTO Seats (seat_id, available, other_info) VALUES (?1, ?2, ?3)",
          params![i, true, ""],
        )
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
pub fn insert_new_user_info(
  user_name: &str,
  password_hash: &str,
  email: &str,
  verification_token: &str,
) -> Result<(), Status> {
  log::info!("Inserting new user information");

  let conn = handle(connect_to_db(), "Connecting to db")?;

  let user_role = user::UserRole::RegularUser.to_string();
  handle(
    conn.execute(
      "INSERT INTO Users (user_name, password_hash, email, user_role, verified, verification_token) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      params![user_name, password_hash, email, user_role, false, verification_token],
    ),
    "Inserting new user information",
  )?;

  log::info!("Successfully inserted new user information");
  Ok(())
}

pub fn update_user_verified_by_token(verification_token: &str) -> Result<(), Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let affected_rows = handle(
    conn.execute(
      "UPDATE Users SET verified = true WHERE verification_token = ?1",
      params![verification_token],
    ),
    "Updating user verified by verification_token",
  )?;

  if affected_rows == 0 {
    log::warn!("No Users found for updation");
    return Err(Status::NotFound);
  }

  Ok(())
}

// 檢查用戶名
pub fn check_if_user_name_exists(user_name: &str) -> Result<bool, Status> {
  log::info!(
    "Checking if username: '{}' exists in the database",
    user_name
  );

  let conn = handle(connect_to_db(), "Connecting to db")?;

  let count: u64 = handle(
    conn.query_row(
      "SELECT COUNT(*) FROM Users WHERE user_name = ?1",
      params![user_name],
      |row| row.get(0),
    ),
    "Querying select operation",
  )?;

  if count > 0 {
    log::debug!("username: {} exists", user_name);

    Ok(true)
  } else {
    log::debug!("username: {} does not exist", user_name);

    Ok(false)
  }
}

// 登入
pub fn login() {}

// 查詢所有位置在特定時間點狀態
pub fn get_all_seats_status(date: NaiveDate, time: u32) -> Result<seat::AllSeatsStatus, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT 
      Seats.seat_id,
      CASE
        WHEN Seats.available = 0 THEN 'Unavailable'
        WHEN Reservations.seat_id IS NULL THEN 'Available'
        ELSE 'Borrowed'
      END as ReservationStatus
    FROM 
      Seats
    LEFT JOIN Reservations ON 
      Seats.seat_id = Reservations.seat_id AND
      Reservations.date = ? AND
      Reservations.start_time <= ? AND
      Reservations.end_time > ?
      ",
    ),
    "Selecting seat_id",
  )?;

  let seat_status_iter = handle(
    stmt.query_map(params![date, time, time], |row| {
      let seat_id: u16 = row.get(0)?;
      let status_str: String = row.get(1)?;
      let status = match status_str.as_str() {
        "Available" => seat::Status::Available,
        "Borrowed" => seat::Status::Borrowed,
        "Unavailable" => seat::Status::Unavailable,
        _ => return Err(rusqlite::Error::InvalidQuery),
      };

      Ok(seat::SeatStatus {
        seat_id: seat_id,
        status: status,
      })
    }),
    "Query mapping",
  )?;

  let seats_vec: Vec<seat::SeatStatus> =
    handle(seat_status_iter.collect(), "Collecting seats status")?;

  let all_seats_status = seat::AllSeatsStatus { seats: seats_vec };

  Ok(all_seats_status)
}

pub fn get_seats_status_by_time(
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<seat::AllSeatsStatus, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT DISTINCT 
      Seats.seat_id,
      CASE
        WHEN Seats.available = 0 THEN 'Unavailable'
        WHEN Reservations.seat_id IS NULL THEN 'Available'
        ELSE 'Borrowed'
      END as ReservationStatus
    FROM 
      Seats
    LEFT JOIN Reservations ON 
      Seats.seat_id = Reservations.seat_id AND
      Reservations.date = ? AND
      (MAX(?, start_time) < MIN(?, end_time))
      ",
    ),
    "Selecting seat_id",
  )?;

  let seat_status_iter = handle(
    stmt.query_map(params![date, start_time, end_time], |row| {
      let seat_id: u16 = row.get(0)?;
      let status_str: String = row.get(1)?;
      let status = match status_str.as_str() {
        "Available" => seat::Status::Available,
        "Borrowed" => seat::Status::Borrowed,
        "Unavailable" => seat::Status::Unavailable,
        _ => return Err(rusqlite::Error::InvalidQuery),
      };

      Ok(seat::SeatStatus {
        seat_id: seat_id,
        status: status,
      })
    }),
    "Query mapping",
  )?;

  let seats_vec: Vec<seat::SeatStatus> =
    handle(seat_status_iter.collect(), "Collecting seats status")?;

  let all_seats_status = seat::AllSeatsStatus { seats: seats_vec };

  Ok(all_seats_status)
}

// 查詢特定位置狀態
pub fn get_seat_reservations(date: NaiveDate, seat_id: u16) -> Result<Vec<(u32, u32)>, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT
      start_time, end_time
    FROM 
      Reservations
    Where
      date = ? AND seat_id = ?
      ",
    ),
    "Selecting date, start_time, end_time",
  )?;

  let timeslots_iter = handle(
    stmt.query_map(params![date, seat_id], |row| {
      let start_time: u32 = row.get(0)?;
      let end_time: u32 = row.get(1)?;

      Ok((start_time, end_time))
    }),
    "Query mapping",
  )?;

  let timeslots: Vec<(u32, u32)> = handle(
    timeslots_iter.collect(),
    "Collecting time slots in reservations",
  )?;

  Ok(timeslots)
}

// 預約座位
pub fn reserve_seat(
  user_id: &str,
  seat_id: u16,
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<(), Status> {
  let mut conn = handle(connect_to_db(), "Connecting to db")?;
  let tx = handle(conn.transaction(), "Starting new transaction")?;
  let is_overlapping: bool;

  {
    let mut stmt = handle(
      tx.prepare(
        "SELECT EXISTS(
        SELECT 1 FROM Reservations
        WHERE date = ?
        AND  (MAX(?, start_time) < MIN(?, end_time))
      )",
      ),
      "Selecting overlaping reservations",
    )?;

    is_overlapping = handle(
      stmt.query_row(params![date, start_time, end_time], |row| row.get(0)),
      "Query mapping",
    )?;
  }

  if is_overlapping {
    log::warn!("Found overlapping reservation");

    handle(tx.rollback(), "Rolling back")?;
    return Err(Status::Conflict);
  }

  handle(
    tx.execute(
      "INSERT INTO Reservations (user_id, seat_id, date, start_time, end_time) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![user_id, seat_id, date, start_time, end_time],
    ),
    "Inserting new Reservation information",
  )?;

  handle(tx.commit(), "Commiting transcation")?;

  Ok(())
}

pub fn update_reservation_time(
  user_id: &str,
  date: NaiveDate,
  new_start_time: u32,
  new_end_time: u32,
) -> Result<(), Status> {
  let mut conn = handle(connect_to_db(), "Connecting to db")?;
  let tx = handle(conn.transaction(), "Starting new transaction")?;
  let is_overlapping: bool;

  {
    let mut stmt = handle(
      tx.prepare(
        "SELECT EXISTS(
            SELECT 1 FROM Reservations
            WHERE user_id != ?
            AND date = ?
            AND  (MAX(?, start_time) < MIN(?, end_time))
        )",
      ),
      "Preparing to check for overlapping reservations",
    )?;

    is_overlapping = handle(
      stmt.query_row(
        params![user_id, date, new_start_time, new_end_time],
        |row| row.get(0),
      ),
      "Query mapping",
    )?;
  }

  if is_overlapping {
    log::warn!("Found overlapping reservation");

    handle(tx.rollback(), "Rolling back")?;
    return Err(Status::Conflict);
  }

  let affected_rows = handle(
    tx.execute(
      "UPDATE Reservations SET start_time = ?, end_time = ? WHERE user_id = ? AND date = ?",
      params![new_start_time, new_end_time, user_id, date],
    ),
    "Updating reservation",
  )?;

  if affected_rows == 0 {
    log::warn!("No reservation found for updation");

    handle(tx.rollback(), "Rolling back")?;
    return Err(Status::NotFound);
  } else {
    handle(tx.commit(), "Commiting transcation")?;
  }

  Ok(())
}

pub fn delete_reservation_time(user_id: &str, date: NaiveDate) -> Result<(), Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let affected_rows = handle(
    conn.execute(
      "DELETE FROM Reservations WHERE user_id = ?1 AND date = ?2",
      params![user_id, date],
    ),
    "Deleting reservation",
  )?;

  if affected_rows == 0 {
    log::warn!("No reservation found for deletion");
    return Err(Status::NotFound);
  }

  Ok(())
}

pub fn get_user_reservations(user_id: &str) -> Result<Vec<reservation::Reservation>, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let date = get_today();

  let mut stmt = handle(
    conn
    .prepare("SELECT seat_id, date, start_time, end_time FROM Reservations WHERE user_id = ?1 AND date >= ?")
    ,"Selecting reservations")?;

  let reservations_iter = handle(
    stmt.query_map(params![user_id, date], |row| {
      Ok(reservation::Reservation {
        user_id: user_id.to_owned(),
        seat_id: row.get(0)?,
        date: row.get(1)?,
        start_time: row.get(2)?,
        end_time: row.get(3)?,
      })
    }),
    "Query mapping",
  )?;

  let reservations: Vec<reservation::Reservation> =
    handle(reservations_iter.collect(), "Collecting reservations")?;

  Ok(reservations)
}

pub fn is_overlapping_with_unavailable_timeslot(
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<bool, Status> {
  log::info!(
    "Checking for overlapping timeslots on date: {}, start_time: {}, end_time: {}",
    date,
    start_time,
    end_time
  );
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT EXISTS(
        SELECT 1 FROM UnavailableTimeSlots
    WHERE date = ?
    AND (MAX(?, start_time) < MIN(?, end_time))
      )",
    ),
    "Selecting overlaping unavailable time slots",
  )?;

  let is_overlapping: bool = handle(
    stmt.query_row(params![date, end_time, start_time], |row| row.get(0)),
    "Query mapping",
  )?;

  Ok(is_overlapping)
}

pub fn is_within_unavailable_timeslot(date: NaiveDate, time: u32) -> Result<bool, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT EXISTS(
        SELECT 1 FROM UnavailableTimeSlots
    WHERE date = ?
    AND start_time <= ? AND end_time > ?
      )",
    ),
    "Selecting overlaping unavailable time slots",
  )?;

  let is_within_timeslot: bool = handle(
    stmt.query_row(params![date, time, time], |row| row.get(0)),
    "Query mapping",
  )?;

  Ok(is_within_timeslot)
}

pub fn is_seat_available(seat_id: u16) -> Result<bool, Status> {
  let conn = handle(connect_to_db(), "Connecting to db")?;

  let available = handle(
    conn.query_row(
      "SELECT available FROM Seats WHERE seat_id = ?1",
      params![seat_id],
      |row| row.get(0),
    ),
    "Selecting available from Seats",
  )?;

  Ok(available)
}
