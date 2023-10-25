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

/*

fn is_overlapping(slot1: &str, slot2: &str) -> bool {
    let (start1, end1) = parse_slot(slot1);
    let (start2, end2) = parse_slot(slot2);

    max(start1, start2) < min(end1, end2)
}

fn parse_slot(slot: &str) -> (i32, i32) {
    let times: Vec<&str> = slot.split('-').collect();
    let start = times[0].replace(":", "").parse::<i32>().unwrap();
    let end = times[1].replace(":", "").parse::<i32>().unwrap();

    (start, end)
}

*/
