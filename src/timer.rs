use crate::{database, utils::*};
use chrono::Datelike;
use rusqlite::params;
use std::fs;
use tokio::time::sleep;

pub async fn start() {
  log::info!("Starting timmer!");
  delete_logfile();
  set_unavailable_timeslots();
  loop {
    let tomorrow_midnight = get_tomorrow_midnight();
    let duration = tomorrow_midnight - get_datetime();
    let std_duration = duration.to_std().unwrap();
    // let std_duration = std::time::Duration::from_secs(3);

    sleep(std_duration).await;
    delete_logfile();
    set_unavailable_timeslots();
  }
}

fn delete_logfile() {
  log::info!("Deleting logfiles");

  let root = get_root();
  let path = root + "/logfiles";
  log::debug!("path={}", path);

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
            if date <= get_last_week() {
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

pub fn set_unavailable_timeslots() {
  log::info!("Setting unavailable timeslots");

  let today = get_today();

  for i in 0..=3 {
    let future_date = today + chrono::Duration::days(i);
    let weekday = future_date.weekday();
    let is_holiday = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;

    let mut time_slots = Vec::new();

    if is_holiday {
      time_slots.push((000000, 090000));
      time_slots.push((170000, 240000));
    } else {
      time_slots.push((000000, 080000));
      time_slots.push((220000, 240000));
    }

    database::insert_unavailable_timeslots(future_date, time_slots)
      .expect("Inserting unavailable timeslots failed");
  }
}

// TEST
