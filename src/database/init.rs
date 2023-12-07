use crate::model::*;

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
