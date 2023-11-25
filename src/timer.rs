use crate::{database, utils::*};
use std::{env, fs};
use tokio::time::sleep;

pub async fn start() {
  log::info!("Starting timmer!");
  delete_logfile();
  //   delete_data();
  loop {
    let tomorrow_midnight = get_tomorrow_midnight();
    let duration = tomorrow_midnight - get_datetime();
    let std_duration = duration.to_std().unwrap();
    // let std_duration = std::time::Duration::from_secs(3);

    sleep(std_duration).await;
    delete_logfile();
    // delete_data();
  }
}

fn delete_logfile() {
  log::info!("Deleting logfiles");

  let root = env::var("ROOT").expect("Failed to get root path");
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

// pub fn delete_data() -> Result<(), Status> {}
