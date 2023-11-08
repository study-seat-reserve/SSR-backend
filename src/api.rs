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
  if database::check_if_user_name_exists(&data.user_name)? {
    return Err(Status::Conflict);
  }

  /* if let true = database::check_if_email_exists(&data.user_name)? {
    return Err(Status::Conflict);
  } */

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
  let time = now.time();

  log::info!("Show current seats status");

  let all_seats_status = database::get_all_seats_status(date, time, time)?;

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show current seats status successfully");

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
#[post("/api/reserve", format = "json", data = "<reservation>")]
pub async fn reserve_seat(reservation: Json<reservation::Reservation>) -> Result<(), Status> {
  handle_validator(reservation.validate())?;
  let reservation_data: reservation::Reservation = reservation.into_inner();

  let user_id = reservation_data.user_id;
  let seat_id = reservation_data.seat_id;
  let date = reservation_data.date;
  let start_time = reservation_data.start_time;
  let end_time = reservation_data.end_time;

  log::info!("Reserving a seat :{} for user: {}", seat_id, user_id);

  if database::is_overlapping_with_other_reservation(seat_id, date, start_time, end_time)? {
    return Err(Status::Conflict);
  };
  if database::is_overlapping_with_unavailable_timeslot(date, start_time, end_time)? {
    return Err(Status::Conflict);
  };

  database::reserve_seat(&user_id, seat_id, date, start_time, end_time)?;

  log::info!(
    "Seat: {} reserved successfully for user_id: {}",
    seat_id,
    user_id
  );

  Ok(())
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

/*
todo
1.u32->naivetime
2.overlap
3.不能預約比現在要早的時間
*/
