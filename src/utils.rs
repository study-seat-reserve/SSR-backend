// use crate::model::constant::*;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use reqwest;
pub use rocket::http::Status;
use rusqlite::{Error as SqliteError, ErrorCode};
use validator::ValidationErrorsKind;

use std::{
  cmp::{max, min},
  collections::HashMap,
  env,
  fs::{self, File},
  io::{Error, ErrorKind},
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

pub fn handle<T, E>(result: Result<T, E>, prefix: &str) -> Result<T, Status>
where
  E: std::error::Error + 'static,
{
  result.map_err(|err| {
    log::error!("{} failed with error: {:?}", prefix, err);

    let dyn_error: &dyn std::error::Error = &err;

    if let Some(e) = dyn_error.downcast_ref::<std::io::Error>() {
      match e.kind() {
        ErrorKind::NotFound => Status::NotFound,
        ErrorKind::PermissionDenied => Status::Forbidden,
        ErrorKind::ConnectionRefused => Status::ServiceUnavailable,
        _ => Status::InternalServerError,
      }
    } else if let Some(e) = dyn_error.downcast_ref::<rusqlite::Error>() {
      match e {
        SqliteError::SqliteFailure(se, _) => match se.code {
          ErrorCode::ConstraintViolation | ErrorCode::TypeMismatch => Status::UnprocessableEntity,
          ErrorCode::PermissionDenied => Status::Forbidden,
          ErrorCode::NotFound => Status::NotFound,
          ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => Status::ServiceUnavailable,
          _ => Status::InternalServerError,
        },
        _ => Status::InternalServerError,
      }
    } else {
      Status::InternalServerError
    }
  })
}

pub fn handle_validator(result: Result<(), validator::ValidationErrors>) -> Result<(), Status> {
  result.map_err(|validation_errors| {
    let error_message: Vec<String> = validation_errors
      .errors()
      .into_iter()
      .map(|(field, validation_error_kind)| {
        let msg: Vec<String> =
          if let ValidationErrorsKind::Field(validation_error) = validation_error_kind {
            validation_error
              .into_iter()
              .map(|error| format!("Invalid {} Error: {}", field, error.code))
              .collect()
          } else {
            Vec::new()
          };
        return msg.join(", ");
      })
      .collect();

    log::error!("{}", error_message.join(", "));

    Status::UnprocessableEntity
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

pub fn date_from_string(date: &str) -> Result<NaiveDate, Status> {
  handle(
    NaiveDate::parse_from_str(date, "%Y-%m-%d"),
    &format!("Parsing date from str '{}'", date),
  )
}

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
