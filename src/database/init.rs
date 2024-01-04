use std::env;

use super::{
  common::*,
  timeslot::{insert_unavailable_timeslot, is_overlapping_with_unavailable_timeslot},
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Datelike;

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
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            PRIMARY KEY (user_name, start_time, end_time),
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
      start_time TEXT NOT NULL,
      end_time TEXT NOT NULL,
      PRIMARY KEY (start_time, end_time)
    )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create UnavailableTimeSlots table: {}", e);
    panic!("Failed to create UnavailableTimeSlots table");
  });

  sqlx::query(
    "CREATE TABLE IF NOT EXISTS BlackList (
      user_name TEXT NOT NULL,
      start_time TEXT NOT NULL,
      end_time TEXT NOT NULL,
      PRIMARY KEY (user_name),
      FOREIGN KEY(user_name) REFERENCES Users(user_name)
    )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create BlackList table: {}", e);
    panic!("Failed to create BlackList table");
  });

  init_seat_info(&pool).await;

  init_unavailable_timeslots(&pool).await;

  insert_admin(&pool).await;

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

async fn init_unavailable_timeslots(pool: &Pool<Sqlite>) {
  log::info!("Setting unavailable timeslots");

  let today = get_today();
  let mut time_slots: Vec<(i64, i64)> = Vec::new();

  for i in 0..=3 {
    let future_date = today + chrono::Duration::days(i);
    let weekday = future_date.weekday();
    let is_holiday = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;

    if is_holiday {
      let start_time: i64 =
        naive_date_to_timestamp(future_date, 0, 0, 0).expect("Invalid timestamp");
      let end_time: i64 = naive_date_to_timestamp(future_date, 9, 0, 0).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));

      let start_time: i64 =
        naive_date_to_timestamp(future_date, 17, 0, 0).expect("Invalid timestamp");
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));
    } else {
      let start_time: i64 =
        naive_date_to_timestamp(future_date, 0, 0, 0).expect("Invalid timestamp");
      let end_time: i64 = naive_date_to_timestamp(future_date, 8, 0, 0).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));

      let start_time: i64 =
        naive_date_to_timestamp(future_date, 22, 0, 0).expect("Invalid timestamp");
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));
    }
  }

  for (start_time, end_time) in time_slots.into_iter() {
    let overlapping = is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time)
      .await
      .unwrap_or_else(|e| {
        log::error!(
          "Failed to check overlapping with unavailable timeslot: {}",
          e
        );
        panic!(
          "Failed to check overlapping with unavailable timeslot: {}",
          e
        );
      });

    if !overlapping {
      insert_unavailable_timeslot(pool, start_time, end_time)
        .await
        .unwrap_or_else(|e| {
          log::error!("Failed to insert unavailable timeslots: {}", e);
          panic!("Failed to insert unavailable timeslots: {}", e);
        });
    }
  }
}

pub async fn clear_table(pool: &Pool<Sqlite>) {
  let table_names = [
    "BlackList",
    "Reservations",
    "Seats",
    "Users",
    "UnavailableTimeSlots",
  ];
  for table_name in table_names {
    let sql = format!("DELETE FROM {}", table_name);
    query(&sql)
      .execute(pool)
      .await
      .expect("Failed to clear table");
  }
}

async fn insert_admin(pool: &Pool<Sqlite>) {
  log::info!("Inserting admin");

  let admin_password = env::var("ADMIN_PASSWORD").expect("Failed to get admin password");

  let password_hash = hash(admin_password, DEFAULT_COST).expect("Hashing password failed");

  let admin_email = env::var("ADMIN_EMAIL").expect("Failed to get admin email");

  query!(
    "INSERT OR IGNORE INTO Users 
        (user_name, password_hash, email, user_role, verified, verification_token) 
      VALUES 
        (?1, ?2, ?3, ?4, ?5, ?6)",
    "admin",
    password_hash,
    admin_email,
    user::UserRole::Admin,
    true,
    ""
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to insert admin: {}", e);
    panic!("Failed to insert admin: {}", e);
  });
}

#[cfg(test)]
mod test {
  use sqlx::Row;

  use super::*;
  use crate::utils;

  #[tokio::test]
  async fn test_init_db() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("DROP TABLE IF EXISTS Seats")
      .execute(&pool)
      .await
      .unwrap();
    sqlx::query("DROP TABLE IF EXISTS Users")
      .execute(&pool)
      .await
      .unwrap();
    sqlx::query("DROP TABLE IF EXISTS Reservations")
      .execute(&pool)
      .await
      .unwrap();
    sqlx::query("DROP TABLE IF EXISTS UnavailableTimeSlots")
      .execute(&pool)
      .await
      .unwrap();

    // 呼叫測試函數
    super::init_db(&pool).await;

    // 檢查 Seats 表是否正確創建
    let seats_exist =
      sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Seats'")
        .fetch_optional(&pool)
        .await
        .unwrap()
        .is_some();
    assert!(seats_exist);

    // 檢查其他表是否也正確創建
    let users_exist =
      sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Users'")
        .fetch_optional(&pool)
        .await
        .unwrap()
        .is_some();
    assert!(users_exist);

    let reservations_exist =
      sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Reservations'")
        .fetch_optional(&pool)
        .await
        .unwrap()
        .is_some();
    assert!(reservations_exist);

    let unavailable_timeslots_exist = sqlx::query(
      "SELECT name FROM sqlite_master WHERE type='table' AND name='UnavailableTimeSlots'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();
    assert!(unavailable_timeslots_exist);

    let admin_count = sqlx::query!("SELECT COUNT(*) FROM Users WHERE user_name = 'admin'")
      .fetch_one(&pool)
      .await
      .unwrap()
      .0;
    assert_eq!(admin_count, 1);
  }

  #[tokio::test]
  async fn test_insert_admin() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    // 清空 Users 表
    sqlx::query("DELETE FROM Users")
      .execute(&pool)
      .await
      .unwrap();

    // 呼叫函式
    insert_admin(&pool).await;

    // 檢查 Admin 使用者是否已新增
    let admin_user = sqlx::query_as!(
      user::UserInfo,
      "SELECT * FROM Users WHERE user_name = 'admin'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch admin user");

    assert_eq!(admin_user.user_name, "admin");
    assert_ne!(admin_user.password_hash, "");
    assert_eq!(admin_user.user_role, user::UserRole::Admin);
    assert!(admin_user.verified);
  }

  #[tokio::test]
  async fn test_clear_table() {
    // 連線到一個記憶體資料庫
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE TestTable1 (id INTEGER PRIMARY KEY)")
      .execute(&pool)
      .await
      .unwrap();

    sqlx::query("INSERT INTO TestTable1 (id) VALUES (1)")
      .execute(&pool)
      .await
      .unwrap();

    sqlx::query("CREATE TABLE TestTable2 (id INTEGER PRIMARY KEY)")
      .execute(&pool)
      .await
      .unwrap();

    sqlx::query("INSERT INTO TestTable2 (id) VALUES (1)")
      .execute(&pool)
      .await
      .unwrap();

    // 呼叫要測試的函式
    super::clear_table(&pool).await;

    // 檢查資料表是否已清空
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM TestTable1")
      .fetch_one(&pool)
      .await
      .unwrap()
      .get(0);

    assert_eq!(0, count);

    let count: i64 = sqlx::query("SELECT COUNT(*) FROM TestTable2")
      .fetch_one(&pool)
      .await
      .unwrap()
      .get(0);

    assert_eq!(0, count);
  }

  #[tokio::test]
  async fn test_init_unavailable_timeslots() {
    // 連線測試資料庫
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    // 建立 UnavailableTimeSlots 表格
    sqlx::query(
      "CREATE TABLE UnavailableTimeSlots (
            start_time INTEGER NOT NULL,
            end_time INTEGER NOT NULL,
            PRIMARY KEY (start_time, end_time)
          )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("DELETE FROM UnavailableTimeSlots")
      .execute(&pool)
      .await
      .unwrap();

    let today = utils::get_today();

    super::init_unavailable_timeslots(&pool).await;

    let result =
      sqlx::query_as::<_, (i64, i64)>("SELECT start_time, end_time FROM UnavailableTimeSlots")
        .fetch_all(&pool)
        .await
        .unwrap();

    let mut expected = Vec::new();

    let start_time = utils::naive_date_to_timestamp(today, 0, 0, 0).unwrap();
    let end_time = utils::naive_date_to_timestamp(today, 9, 0, 0).unwrap();
    expected.push((start_time, end_time));

    let start_time = utils::naive_date_to_timestamp(today, 17, 0, 0).unwrap();
    let end_time = utils::naive_date_to_timestamp(today, 23, 59, 59).unwrap();
    expected.push((start_time, end_time));

    assert_eq!(result, expected);
  }

  #[tokio::test]
  async fn test_init_seat_info() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("DELETE FROM Seats")
      .execute(&pool)
      .await
      .unwrap();

    super::init_seat_info(&pool).await;

    let count: u16 = sqlx::query("SELECT COUNT(*) FROM Seats")
      .fetch_one(&pool)
      .await
      .unwrap()
      .get(0);

    assert_eq!(count, constant::NUMBER_OF_SEATS);

    let available: Vec<bool> = sqlx::query("SELECT available FROM Seats")
      .fetch_all(&pool)
      .await
      .unwrap()
      .into_iter()
      .map(|r| r.get(0))
      .collect();

    assert!(available.into_iter().all(|a| a));
  }
}
