use crate::{
  database::timeslot::{insert_unavailable_timeslot, is_overlapping_with_unavailable_timeslot},
  model::*,
  utils::{get_today, naive_date_to_timestamp},
};

use chrono::Datelike;
use sqlx::{query, query_as, Pool, Sqlite};

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
            start_time INTEGER NOT NULL,
            end_time INTEGER NOT NULL,
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
      start_time INTEGER NOT NULL,
      end_time INTEGER NOT NULL,
      PRIMARY KEY (start_time, end_time)
    )",
  )
  .execute(pool)
  .await
  .unwrap_or_else(|e| {
    log::error!("Failed to create UnavailableTimeSlots table: {}", e);
    panic!("Failed to create UnavailableTimeSlots table");
  });

  init_seat_info(&pool).await;

  init_unavailable_timeslots(&pool).await;

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
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 9, 0, 0).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));

      let start_time: i64 =
        naive_date_to_timestamp(future_date, 17, 0, 0).expect("Invalid timestamp");
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));
    } else {
      let start_time: i64 =
        naive_date_to_timestamp(future_date, 0, 0, 0).expect("Invalid timestamp");
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 8, 0, 0).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));

      let start_time: i64 =
        naive_date_to_timestamp(future_date, 22, 0, 0).expect("Invalid timestamp");
      let end_time: i64 =
        naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

      time_slots.push((start_time, end_time));
    }
  }

  for (start_time, end_time) in time_slots.into_iter() {
    let is_overlapping = is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time)
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

    if !is_overlapping {
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
  let table_names = ["Reservations", "Seats", "Users", "UnavailableTimeSlots"];
  for table_name in table_names {
    let query = format!("DELETE FROM {}", table_name);
    sqlx::query(&query)
      .execute(pool)
      .await
      .expect("Failed to clear table");
  }
}
