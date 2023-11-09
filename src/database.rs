/*
座位表 (Seat)
seat_id (pk)
other_info

用戶表 (Users)
id (pk)
user_id
user_name
password_hash
blacklist
email

預約表 (Reservations)
user_id (pk、fk)
seat_id (pk、fk)
date (pk)
start_time
end_time


時段表 (TimeSlots)
date (pk)
start_time
end_time
availability (bool)

trigger?
*/

use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use std::env;

use bcrypt::{hash, DEFAULT_COST};

use crate::{
  model::{constant::*, *},
  utils::*,
};

fn connect_to_db() -> Result<Connection> {
  log::info!("Connecting to db");

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/SSR.db3", root);
  log::debug!("path={}", path);

  Connection::open(path)
}

// 初始化
pub fn init_db() {
  log::info!("Initializing db");

  let root = env::var("ROOT").expect("Failed to get root path");
  let path: String = format!("{}/SSR.db3", root);
  log::debug!("path={}", path);

  let conn = Connection::open(path).unwrap_or_else(|e| {
    log::error!("Failed to open SQLite connection: {}", e);
    panic!("Failed to open SQLite connection");
  });

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS Seat (
         seat_id INTEGER PRIMARY KEY,
         other_info TEXT
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create Seat table: {}", e);
      panic!("Failed to create Seat table");
    });

  init_seat_info(&conn);

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS Users (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         user_id TEXT NOT NULL UNIQUE,
         user_name TEXT NOT NULL UNIQUE,
         password_hash TEXT NOT NULL,
         blacklist INTEGER DEFAULT 0,
         email TEXT NOT NULL UNIQUE
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
         FOREIGN KEY(seat_id) REFERENCES Seat(seat_id)
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
    .query_row("SELECT COUNT(*) FROM Seat", [], |row| row.get(0))
    .unwrap_or_else(|e| {
      log::error!("Failed to create TimeSlots table: {}", e);
      panic!("Failed to create TimeSlots table");
    });

  if count != constant::NUMBER_OF_SEATS {
    for i in 1..=constant::NUMBER_OF_SEATS {
      conn
        .execute(
          "INSERT INTO Seat (seat_id, other_info) VALUES (?1, ?2)",
          params![i, ""],
        )
        .unwrap_or_else(|e| {
          log::error!("Failed to init Seat table: {}", e);
          panic!("Failed to init Seat table");
        });
    }
  }
}

// 註冊
pub fn insert_new_user_info(user: user::User, user_id: &str) -> Result<(), Status> {
  log::info!("Inserting new user information");

  let password_hash = handle(hash(user.password, DEFAULT_COST), "Hashing password")?;

  let conn = handle(connect_to_db(), "Connecting to db")?;

  handle(
    conn.execute(
      "INSERT INTO Users (user_id, user_name, password_hash, blacklist, email) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![user_id, user.user_name, password_hash, 0, user.email],
    ),
    "Inserting new user information",
  )?;

  log::info!("Successfully inserted new user information");
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
  log::info!("Getting current seats status");

  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT 
      Seat.seat_id,
      CASE
        WHEN Reservations.seat_id IS NULL THEN 'Available'
        ELSE 'Borrowed'
      END as ReservationStatus
    FROM 
      Seat
    LEFT JOIN Reservations ON 
      Seat.seat_id = Reservations.seat_id AND
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

  log::info!("Successfully got current seats status");

  Ok(all_seats_status)
}

pub fn get_seats_status_by_time(
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<seat::AllSeatsStatus, Status> {
  log::info!("Getting seats status by time");

  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT DISTINCT 
      Seat.seat_id,
      CASE
        WHEN Reservations.seat_id IS NULL THEN 'Available'
        ELSE 'Borrowed'
      END as ReservationStatus
    FROM 
      Seat
    LEFT JOIN Reservations ON 
      Seat.seat_id = Reservations.seat_id AND
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

  log::info!("Successfully got seats status by time");

  Ok(all_seats_status)
}

// 查詢特定位置狀態
pub fn get_seat_reservations(date: NaiveDate, seat_id: u16) -> Result<Vec<(u32, u32)>, Status> {
  log::info!("Getting seat: {} status", seat_id);

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

  log::info!("Successfully got seats: {} status", seat_id);

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
  log::info!("Reserving a seat for user_id: {}", user_id);

  let conn = handle(connect_to_db(), "Connecting to db")?;

  handle(
    conn.execute(
      "INSERT INTO Reservations (user_id, seat_id, date, start_time, end_time) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![user_id, seat_id, date, start_time, end_time],
    ),
    "Inserting new Reservation information",
  )?;

  log::info!("Successfully reserved a seat for user_id: {}", user_id);
  Ok(())
}

pub fn is_overlapping_with_other_reservation(
  seat_id: u16,
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<bool, Status> {
  log::info!(
    "Checking for overlapping reservations for seat_id: {}, date: {}, start_time: {}, end_time: {}",
    seat_id,
    date,
    start_time,
    end_time
  );

  let conn = handle(connect_to_db(), "Connecting to db")?;

  let mut stmt = handle(
    conn.prepare(
      "SELECT EXISTS(
        SELECT 1 FROM Reservations
        WHERE date = ?
        AND  (MAX(?, start_time) < MIN(?, end_time))
      )",
    ),
    "Selecting overlaping reservations",
  )?;

  let is_overlapping: bool = handle(
    stmt.query_row(params![date, start_time, end_time], |row| row.get(0)),
    "Query mapping",
  )?;

  if is_overlapping {
    log::warn!("Found overlapping reservation");
    return Ok(true);
  }

  Ok(false)
}

pub fn is_overlapping_with_unavailable_timeslot(
  date: NaiveDate,
  start_time: u32,
  end_time: u32,
) -> Result<bool, Status> {
  log::info!(
    "Checking for overlapping unavailable time slots for date: {}, start_time: {}, end_time: {}",
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

  if is_overlapping {
    log::warn!("Found overlapping unavailable time slot");
    return Ok(true);
  }

  Ok(false)
}
