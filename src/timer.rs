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
    // let std_duration = std::time::Duration::from_secs(3);

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
  use mockito;
  use sqlx::sqlite::SqlitePool;
  use tempdir::TempDir;

  #[tokio::test]
  async fn test_start() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let delete_logfile_mock = mockito::mock("DELETE_LOGFILE_MOCK").expect(1);

    let set_unavailable_timeslots_mock = mockito::mock("SET_UNAVAILABLE_TIMESLOTS_MOCK").expect(1);

    mockito::expect!(delete_logfile_mock);
    mockito::expect!(set_unavailable_timeslots_mock);

    start(&pool).await;

    delete_logfile_mock.assert();
    set_unavailable_timeslots_mock.assert();
  }

  #[test]
  fn test_delete_logfile() {
    let temp_dir = TempDir::new("logs").unwrap();

    let old_log = temp_dir.path().join("old.log");
    fs::write(&old_log, "old log content").unwrap();

    let new_log = temp_dir.path().join("new.log");
    fs::write(&new_log, "new log content").unwrap();

    std::env::set_var("ROOT", temp_dir.path());

    delete_logfile();

    assert!(!old_log.exists());
    assert!(new_log.exists());
  }

  #[tokio::test]
  async fn test_set_unavailable_timeslots() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let insert_mock = mockito::mock("INSERT");

    mockito::expect!(insert_mock);

    set_unavailable_timeslots(&pool).await;

    mockito::verify!(insert_mock);
  }

  #[test]
  fn test_date_from_string() {
    let valid_date = "2022-01-01";
    let invalid_date = "abc";

    assert!(date_from_string(valid_date).is_ok());
    assert!(date_from_string(invalid_date).is_err());
  }
}
