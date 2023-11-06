use crate::{database, model::*, utils::*};

use bcrypt::verify;
use chrono::{Local, NaiveDate, NaiveTime};
use rocket::{form::Form, fs::NamedFile, get, http::Status, post, serde::json::Json, State};
use uuid::Uuid;
use validator::Validate;

// 註冊
#[post("/api/register", format = "json", data = "<data>")]
pub async fn register(data: Json<user::User>) -> Result<(), Status> {
  handle_validator(data.validate())?;
  if let true = database::check_if_user_name_exists(&data.user_name)? {
    return Err(Status::Conflict);
  }

  let user_id = Uuid::new_v4().to_string();

  log::info!("Handling registration for user: {}", &user_id);

  let user = data.into_inner();

  database::insert_new_user_info(user, &user_id)?;

  //TODO: 信箱驗證

  log::info!("Finished registration for user: {}", &user_id);
  Ok(())
}

// 登入
pub async fn login() {
  // let valid = verify("hunter2", &hashed).unwrap();

  //TODO: SQL injection
}

// 查詢當前所有位置狀態
#[get("/api/show_status")]
pub async fn show_current_seats_status() -> Result<String, Status> {
  let now = Local::now();
  let date = now.date_naive();
  let time = now.time().format("%H:%M:%S");
  println!("date: {}, time: {}", date, time);

  let all_seats_status =
    database::get_all_seats_status(&date.to_string(), &time.to_string(), &time.to_string())?;

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  Ok(json)
}

// 查詢當前所有位置狀態 + filter
#[get("/api/show_status/<date>/<start_time>/<end_time>")]
pub async fn show_all_seats_status_by_time(
  date: &str,
  start_time: &str,
  end_time: &str,
) -> Result<(), Status> {
  let date = handle(NaiveDate::parse_from_str(date, "%Y-%m-%d"), "Parsing date")?;
  let start_time = handle(
    NaiveTime::parse_from_str(start_time, "%H:%M:%S"),
    "Parsing time",
  )?;
  let end_time = handle(
    NaiveTime::parse_from_str(end_time, "%H:%M:%S"),
    "Parsing time",
  )?;

  Ok(())
}

// 查詢當前特定位置預約狀態
pub async fn show_specific_seat_status() {}

// 預約座位
pub async fn reserve_seat() {
  // return status
}

// 修改預約時段
pub async fn modify_reservation_time() {
  // return status
}

// 刪除預約時段
pub async fn delete_reservation_time() {
  // return status
}

// 顯示使用者預約時段
pub async fn get_user_reservation_times() {
  // return time
}
