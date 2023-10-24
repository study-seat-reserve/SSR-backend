// use crate::model::constant::*;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use rand::Rng;
use reqwest;
pub use std::io::{Error, ErrorKind};
use std::{
  collections::HashMap,
  env,
  fs::{self, File},
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

pub fn handle<T, E>(result: Result<T, E>, msg: &str) -> Result<T, Error>
where
  E: std::error::Error,
{
  result.map_err(|e| {
    log::error!("{} failed with error: {:?}", msg, e);

    let err_msg = format!("{} failed", msg);
    Error::new(ErrorKind::Other, err_msg)
  })
}

pub fn get_date() -> NaiveDate {
  Local::now().date_naive()
}

pub fn get_datetime() -> NaiveDateTime {
  Local::now().naive_local()
}

pub fn get_tomorrow_midnight() -> NaiveDateTime {
  let now = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
  let get_tomorrow_midnight = now + Duration::days(1);
  get_tomorrow_midnight
}

pub fn get_last_week() -> NaiveDate {
  let now = Local::now().date_naive();
  let seven_days_ago = now - Duration::days(7);
  seven_days_ago
}

pub fn date_from_string(date: &str) -> Result<NaiveDate, Error> {
  handle(
    NaiveDate::parse_from_str(date, "%Y-%m-%d"),
    &format!("Parsing date from str '{}'", date),
  )
}
