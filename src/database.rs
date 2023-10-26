/*
座位表 (Seat)
seat_id (主鍵)
OtherInfo

用戶表 (Users)
user_id (主鍵)
user_name
Password_hash
blacklist
email

預約表 (Reservations)
ReservationID (主鍵)
user_id (外鍵)
seat_id (外鍵)
Date (考慮到日期)
StartTime
EndTime


時段表 (TimeSlots)
Date (主鍵)
StartTime (例如: 09:00)
EndTime (例如: 09:30)
Availability (bool)

trigger?
*/

use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Connection, Result};
use std::env;
use uuid::Uuid;

use crate::{
  model::{constant, seat},
  utils::*,
};

fn connect_to_db() -> Result<Connection, Error> {
  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/SSR.db3", root);
  log::debug!("path={}", path);

  handle(Connection::open(path), "DB Connect")
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
         Password_hash TEXT NOT NULL,
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
         Date TEXT NOT NULL,
         StartTime TEXT NOT NULL,
         EndTime TEXT NOT NULL,
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
         Date TEXT PRIMARY KEY,
         StartTime TEXT NOT NULL,
         EndTime TEXT NOT NULL,
         Availability INTEGER DEFAULT 1
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
pub fn insert_new_user_info() {}

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
