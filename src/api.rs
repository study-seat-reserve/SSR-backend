use crate::{
  database,
  model::{constant::NUMBER_OF_SEATS, *},
  utils::*,
};

use bcrypt::verify;
use chrono::NaiveDate;
use rocket::{get, http::Status, post, serde::json::Json, State};
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
  log::info!("Show current seats status");

  let date = get_today();
  let now = get_now();
  let all_seats_status: seat::AllSeatsStatus;

  if database::is_within_unavailable_timeslot(date, now)? {
    let mut seats_vec = Vec::<seat::SeatStatus>::new();
    for s in 1..NUMBER_OF_SEATS {
      seats_vec.push(seat::SeatStatus {
        seat_id: s,
        status: seat::Status::Unavailable,
      })
    }

    all_seats_status = seat::AllSeatsStatus { seats: seats_vec };
  } else {
    all_seats_status = database::get_all_seats_status(date, now)?;
  }

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show current seats status successfully");

  /*
  如果now在unavailable timeslot中回傳
  Unavailable
  否則若是在Reservations中回傳
  Borrowed
  否則是
  Available
  */

  Ok(json)
}

/////
// 查詢當前所有位置狀態 + filter
#[get("/api/show_status/<date>/<start_time>/<end_time>")]
pub async fn show_seats_status_by_time(
  date: &str,
  start_time: u32,
  end_time: u32,
) -> Result<String, Status> {
  log::info!("Show seats status by time");

  let date = date_from_string(date)?;
  validate_datetime(date, start_time, end_time)?;

  let all_seats_status = database::get_seats_status_by_time(date, start_time, end_time)?;

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show seats status by time successfully");

  /*
  如果給定的時間段中有重疊到被預約的時段則回傳
  Borrowed
  否則回傳
  Available
  */

  Ok(json)
}

// 查詢當前特定位置預約狀態
#[get("/api/show_reservations/<date>/<seat_id>")]
pub async fn show_seat_reservations(date: &str, seat_id: u16) -> Result<String, Status> {
  log::info!("Show reservations of seat: {}", seat_id);

  let date = date_from_string(date)?;
  validate_date(date)?;
  validate_seat_id(seat_id)?;

  let timeslots = database::get_seat_reservations(date, seat_id)?;

  let json = handle(
    serde_json::to_string(&timeslots),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show reservations of seat: {} successfully", seat_id);

  Ok(json)
}

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
