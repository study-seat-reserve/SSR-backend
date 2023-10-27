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

use chrono::{NaiveDate, NaiveTime};
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
         start_time TEXT NOT NULL,
         end_time TEXT NOT NULL,
         PRIMARY KEY (user_id, seat_id, Date),
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
      "CREATE TABLE IF NOT EXISTS TimeSlots (
         date TEXT PRIMARY KEY,
         start_time TEXT NOT NULL,
         end_time TEXT NOT NULL,
         availability INTEGER DEFAULT 1
     )",
      [],
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create TimeSlots table: {}", e);
      panic!("Failed to create TimeSlots table");
    });

  log::info!("Initialization completed successfully");
}

fn init_seat_info(conn: &Connection) {
  let count: u16 = conn
    .query_row("SELECT COUNT(*) FROM Seat", [], |row| row.get(0))
    .unwrap_or_else(|e| {
      log::error!("Failed to create TimeSlots table: {}", e);
      panic!("Failed to create TimeSlots table");
    });

  if count != constant::NUMBER_OF_SEAT {
    for i in 1..=constant::NUMBER_OF_SEAT {
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

  log::info!("Insertion completed successfully");
  Ok(())
}

// 檢查用戶名
pub fn check_if_user_name_exists(user_name: &str) -> Result<bool, Status> {
  log::info!(
    "Checking if username: '{}' exists in the database.",
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

// 查詢特定所有位置在特定時間點狀態
pub fn show_status_of_all_seats_by_time(start: NaiveTime, end: NaiveTime) {}

// 查詢特定位置狀態
pub fn show_status_of_the_seat(id: u16) {}

// 預約座位
pub async fn reserve_seat(
  user_id: String,
  seat_id: u16,
  date: NaiveDate,
  start: NaiveTime,
  end: NaiveTime,
) {
  // return status
}
