use crate::{database, utils::*};
use chrono::{Datelike, Duration, NaiveDate};
use sqlx::{Pool, Sqlite};
use std::fs;
use tokio::time::sleep;

pub async fn start(pool: &Pool<Sqlite>) {
  log::info!("Starting timmer!");
  delete_logfile();
  set_unavailable_timeslots(pool).await;
  loop {
    let today = get_today().and_hms_opt(0, 0, 0).unwrap();
    let tomorrow_midnight = today + Duration::days(1);
    let now = get_now();

    let duration = tomorrow_midnight - now;
    let std_duration = duration.to_std().unwrap();
    // let std_duration = std::time::Duration::from_secs(3)

    sleep(std_duration).await;
    delete_logfile();
    set_unavailable_timeslots(pool).await;
  }
}

fn delete_logfile() {
  log::info!("Deleting logfiles");

  let root = get_root();
  let path = root + "/logfiles";
  let last_week = get_today() - Duration::days(7);

  let entries = fs::read_dir(&path).unwrap_or_else(|e| {
    log::error!("Reading directory '{}' failed with err: {:?}", path, e);
    panic!("Reading directory '{}' failed with err: {:?}", path, e);
  });

  for entry in entries {
    match entry {
      Ok(entry) => {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(date_str) = file_name.split('.').next() {
          if let Ok(date) = date_from_string(date_str) {
            // 刪除7天前的logfile
            if date <= last_week {
              fs::remove_file(entry.path()).unwrap_or_else(|e| {
                log::warn!(
                  "Removing file '{:?}' failed with err: {:?}",
                  entry.path(),
                  e
                );
              });
            }
          }
        }
      }
      Err(e) => {
        log::warn!("Failed to read directory entry: {:?}", e);
      }
    }
  }
}

async fn set_unavailable_timeslots(pool: &Pool<Sqlite>) {
  log::info!("Setting unavailable timeslots");

  let today = get_today();

  let future_date = today + chrono::Duration::days(3);
  let weekday = future_date.weekday();
  let is_holiday = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;

  let mut time_slots: Vec<(i64, i64)> = Vec::new();

  if is_holiday {
    let start_time: i64 = naive_date_to_timestamp(future_date, 0, 0, 0).expect("Invalid timestamp");
    let end_time: i64 = naive_date_to_timestamp(future_date, 9, 0, 0).expect("Invalid timestamp");

    time_slots.push((start_time, end_time));

    let start_time: i64 =
      naive_date_to_timestamp(future_date, 17, 0, 0).expect("Invalid timestamp");
    let end_time: i64 =
      naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

    time_slots.push((start_time, end_time));
  } else {
    let start_time: i64 = naive_date_to_timestamp(future_date, 0, 0, 0).expect("Invalid timestamp");
    let end_time: i64 = naive_date_to_timestamp(future_date, 8, 0, 0).expect("Invalid timestamp");

    time_slots.push((start_time, end_time));

    let start_time: i64 =
      naive_date_to_timestamp(future_date, 22, 0, 0).expect("Invalid timestamp");
    let end_time: i64 =
      naive_date_to_timestamp(future_date, 23, 59, 59).expect("Invalid timestamp");

    time_slots.push((start_time, end_time));
  }

  for (start_time, end_time) in time_slots.into_iter() {
    let is_overlapping =
      database::timeslot::is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time)
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
      database::timeslot::insert_unavailable_timeslot(pool, start_time, end_time)
        .await
        .unwrap_or_else(|e| {
          log::error!("Failed to insert unavailable timeslots: {}", e);
          panic!("Failed to insert unavailable timeslots: {}", e);
        });
    }
  }
}

fn date_from_string(date: &str) -> Result<NaiveDate, Status> {
  handle(
    NaiveDate::parse_from_str(date, "%Y-%m-%d"),
    &format!("Parsing date from str '{}'", date),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::database::init::init_db;
  use sqlx::sqlite::SqlitePool;
  use std::{fs, time::Duration};

  #[tokio::test]
  async fn test_start() {
    let pool = create_pool().await;

    start(&pool);

    tokio::time::sleep(Duration::from_secs(5)).await;
  }

  #[tokio::test]
  async fn test_delete_logfile() {
    let pool = create_pool().await;
    let root = get_root();
    let logfiles_path = format!("{}/logfiles", root);

    let test_logfile_path = format!("{}/test_logfile.txt", logfiles_path);
    fs::write(&test_logfile_path, "Test log content").expect("Failed to create test log file");

    delete_logfile();

    assert!(!fs::metadata(&test_logfile_path).is_ok());
  }

  #[tokio::test]
  async fn test_set_unavailable_timeslots() {
    let pool = create_pool().await;

    set_unavailable_timeslots(&pool).await;

    let today = get_today();
    let future_date = today + chrono::Duration::days(3);
    let time_slots = vec![
      (
        naive_date_to_timestamp(future_date, 0, 0, 0).unwrap(),
        naive_date_to_timestamp(future_date, 9, 0, 0).unwrap(),
      ),
      (
        naive_date_to_timestamp(future_date, 17, 0, 0).unwrap(),
        naive_date_to_timestamp(future_date, 23, 59, 59).unwrap(),
      ),
    ];

    for (start_time, end_time) in time_slots {
      assert!(
        database::timeslot::is_overlapping_with_unavailable_timeslot(&pool, start_time, end_time)
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
          })
      );
    }
  }

  #[test]
  fn test_date_from_string() {
    let valid_date = "2023-12-30";
    let invalid_date = "testdate";

    assert!(date_from_string(valid_date).is_ok());
    assert!(date_from_string(invalid_date).is_err());
  }

  async fn create_pool() -> SqlitePool {
    let database_url = "sqlite::memory:";
    let pool = Pool::connect(database_url)
      .await
      .expect("Failed to create in-memory database");
    init_db(&pool).await;
    pool
  }
}
