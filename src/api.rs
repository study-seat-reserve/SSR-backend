use crate::{
  database,
  model::{constant::*, *},
  utils::*,
};

use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{get, http::Status, post, serde::json::Json};
use uuid::Uuid;
use validator::Validate;

// 註冊
#[post("/api/register", format = "json", data = "<data>")]
pub async fn register(data: Json<user::User>) -> Result<(), Status> {
  handle_validator(data.validate())?;

  let user_name = &data.user_name;
  let password = &data.password;
  let email = &data.email;

  log::info!("Handling registration for user: {}", user_name);

  let password_hash = handle(hash(password, DEFAULT_COST), "Hashing password")?;
  let verification_token = Uuid::new_v4().to_string();

  database::insert_new_user_info(user_name, &password_hash, email, &verification_token)?;

  let url = format!(
    "{}/api/verify?verification_token={}",
    get_base_url(),
    verification_token
  );

  send_verification_email(&email, &url)?;

  log::info!("Finished registration for user: {}", user_name);

  Ok(())
}

#[get("/api/verify?<verification_token>")]
pub async fn email_verify(verification_token: String) -> Result<String, Status> {
  log::info!("Verifying email for token: {}", verification_token);

  database::update_user_verified_by_token(&verification_token)?;

  log::info!(
    "Email verification completed for token: {}",
    verification_token
  );

  Ok("Your email has been successfully verified.".to_string())
}

// 登入
#[post("/api/login", data = "<creds>")]
pub fn login(creds: Json<user::LoginCreds>) -> Result<String, Status> {
  handle_validator(creds.validate())?;
  log::info!("Processing the login request");

  let user_info = database::get_user_info(&creds.user_name)?;
  let username = user_info.user_name.clone();
  let password_hash = &user_info.password;
  let verified = user_info.verified;

  let password_matches =
    verify(&creds.password, &password_hash).map_err(|_| Status::InternalServerError)?;

  if !password_matches {
    log::warn!("Password is incorrect");
    return Err(Status::Unauthorized);
  }

  if !verified {
    log::warn!("The user's email has not been verified.");
    return Err(Status::BadRequest);
  }

  let token = create_token(user_info).map_err(|_| Status::InternalServerError)?;

  log::info!("User: {} login successful", username);

  Ok(token)
}

// 查詢當前所有位置狀態
/*
如果now在unavailable timeslot中回傳
Unavailable
否則若是在Reservations中回傳
Borrowed
否則是
Available
*/
#[get("/api/show_status")]
pub async fn show_current_seats_status() -> Result<String, Status> {
  log::info!("Show current seats status");

  let date = get_today();
  let now = get_now();
  let all_seats_status: seat::AllSeatsStatus;

  if database::is_within_unavailable_timeslot(date, now)? {
    log::warn!(
      "The date: {} time: {} is within an unavailable timeslot",
      date,
      now
    );

    let mut seats_vec = Vec::<seat::SeatStatus>::new();

    for seat in 1..NUMBER_OF_SEATS {
      seats_vec.push(seat::SeatStatus {
        seat_id: seat,
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

  Ok(json)
}

// 查詢當前所有位置狀態 + filter
/*
如果給定的時間段中有重疊到被預約的時段則回傳
Borrowed
否則回傳
Available
*/
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
pub async fn reserve_seat(
  claims: token::Claims,
  reservation: Json<reservation::Reservation>,
) -> Result<(), Status> {
  handle_validator(reservation.validate())?;
  let reservation_data: reservation::Reservation = reservation.into_inner();
  let seat_id = reservation_data.seat_id;
  let date = reservation_data.date;
  let start_time = reservation_data.start_time;
  let end_time = reservation_data.end_time;
  let user_name = claims.user;

  log::info!("Reserving a seat :{} for user: {}", seat_id, user_name);

  if !database::is_seat_available(seat_id)? {
    log::warn!("The seat: {} is unavailable", seat_id);
    return Err(Status::UnprocessableEntity);
  }

  if database::is_overlapping_with_unavailable_timeslot(date, start_time, end_time)? {
    log::warn!(
      "The date: {} start_time: {} end_time: {} is overlapping unavailable time slot",
      date,
      start_time,
      end_time
    );
    return Err(Status::Conflict);
  }

  database::reserve_seat(&user_name, seat_id, date, start_time, end_time)?;

  log::info!(
    "Seat: {} reserved successfully for user: {}",
    seat_id,
    user_name
  );

  Ok(())
}

// 修改預約時段
#[post(
  "/api/update_reservation",
  format = "json",
  data = "<update_reservation>"
)]
pub async fn update_reservation(
  claims: token::Claims,
  update_reservation: Json<reservation::UpdateReservation>,
) -> Result<(), Status> {
  handle_validator(update_reservation.validate())?;
  let update_data: reservation::UpdateReservation = update_reservation.into_inner();
  let date = update_data.date;
  let new_start_time = update_data.new_start_time;
  let new_end_time = update_data.new_end_time;
  let user_name = claims.user;

  log::info!("Updating reservation for user: {}", user_name);

  if database::is_overlapping_with_unavailable_timeslot(date, new_start_time, new_end_time)? {
    return Err(Status::Conflict);
  }

  database::update_reservation_time(&user_name, date, new_start_time, new_end_time)?;

  log::info!("Reservation for user: {} updated successfully", user_name);

  Ok(())
}

// 刪除預約時段
#[post(
  "/api/delete_reservation",
  format = "json",
  data = "<delete_reservation>"
)]
pub async fn delete_reservation_time(
  claims: token::Claims,
  delete_reservation: Json<reservation::DeleteReservation>,
) -> Result<(), Status> {
  handle_validator(delete_reservation.validate())?;
  let update_data: reservation::DeleteReservation = delete_reservation.into_inner();
  let date = update_data.date;
  let user_name = claims.user;

  log::info!("Deleting reservation for user: {}", user_name);

  database::delete_reservation_time(&user_name, date)?;

  log::info!("Reservation for user: {} deleted successfully", user_name);

  Ok(())
}

// 顯示使用者預約時段
#[get("/api/user_reservations")]
pub async fn display_user_reservations(
  claims: token::Claims,
) -> Result<Json<Vec<reservation::Reservation>>, Status> {
  let user_name = claims.user;

  log::info!("Displaying the user's reservations");

  let reservations = database::get_user_reservations(&user_name)?;

  log::info!("Displaying the user's reservations successfully");

  Ok(Json(reservations))
}

/*
管理者設定unavaliable time slot
管理者設定座位avaliable
黑名單
使用者改名
*/

#[cfg(test)]
mod tests {

  use crate::utils;

  use super::*;
  use crate::model::user;
  use rocket::http::Status as HttpStatus;
  use rocket::http::Status;
  use rocket::local::blocking::Client;
  use rocket::serde::json::json;
  use rusqlite::params;

  #[test]
  fn test_register_success() {
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let user_data = json!({
        "user_name": "istestuser",
        "password": "istestpassword123",
        "email": "istest@email.ntou.edu.tw",
    });

    let response = client
      .post(format!("{}/api/register", base_url))
      .header(rocket::http::ContentType::JSON)
      .body(user_data.to_string())
      .dispatch();

    assert_eq!(response.status(), Status::Ok); // HttpStatus::Ok
  }

  #[test]
  fn test_register_failure() {
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let user_data = json!({
        "user_name": "istestuser",
        "password": "istestpassword123",
        "email": "istest@email.ntou.edu.tw",
    });

    let response = client
      .post(format!("{}/api/register", base_url))
      .header(rocket::http::ContentType::JSON)
      .body(user_data.to_string())
      .dispatch();

    assert_eq!(response.status(), Status::InternalServerError); // HttpStatus::Ok

    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    // Invalid data with empty user_name
    let invalid_user_data = json!({
        "user_name": "",
        "password": "istestpassword123",
        "email": "istest@email.ntou.edu.tw",
    });

    let response = client
      .post(format!("{}/api/register", base_url))
      .header(rocket::http::ContentType::JSON)
      .body(invalid_user_data.to_string())
      .dispatch();

    assert_eq!(response.status(), HttpStatus::UnprocessableEntity);
  }

  #[test]
  fn test_email_verify_success() {
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let verification_token = "248959e3-04be-4f97-bae0-95c7aeb5279d".to_string();

    let response = client
      .get(format!(
        "{}/api/verify?verification_token={}",
        base_url, verification_token
      ))
      .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response
      .into_string()
      .unwrap()
      .contains("Your email has been successfully verified"));
  }

  #[test]
  fn test_email_verify_missing_token() {
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let response = client.get(format!("{}/api/verify", base_url)).dispatch();

    assert_eq!(response.status(), Status::BadRequest);
  }

  #[test]
  fn test_email_verify_failure() {
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let invalid_token = "invalid-token";

    let response = client
      .get(format!(
        "{}/api/verify?verification_token={}",
        base_url, invalid_token
      ))
      .dispatch();

    assert_eq!(response.status(), Status::NotFound);
  }

  #[test]
  fn test_login_success() {
    let creds = user::LoginCreds {
      user_name: "istestuser".to_string(),
      password: "ispassword123".to_string(),
    };

    let result = login(Json(creds));

    assert_eq!(result.is_ok(), true);

    // unverified
    let creds = user::LoginCreds {
      user_name: "isunverifieduser".to_string(),
      password: "ispassword123".to_string(),
    };

    let result = login(Json(creds));

    assert_eq!(result.unwrap_err(), Status::BadRequest);

    // no username
    let creds = user::LoginCreds {
      user_name: "".to_string(),
      password: "ispassword123".to_string(),
    };

    let result = login(Json(creds));

    assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);

    // no password
    let creds = user::LoginCreds {
      user_name: "istestuser".to_string(),
      password: "".to_string(),
    };

    let result = login(Json(creds));

    assert_eq!(result.unwrap_err(), Status::Unauthorized);
  }

  #[test]
  fn test_login_validator() {
    let creds = user::LoginCreds {
      user_name: "".to_string(),
      password: "istest".to_string(),
    };
    let result = login(Json(creds));
    assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
  }

  #[tokio::test]
  async fn test_show_current_seats_status_success() {
    let result = show_current_seats_status().await;

    assert!(result.is_ok());
    let result_str = result.unwrap();

    let _: seat::AllSeatsStatus =
      serde_json::from_str(&result_str).expect("Result should be AllSeatsStatus JSON");

    // unvailable time slot
    database::init_db(); // 初始化 DB

    let mut conn = database::connect_to_db().unwrap();
    let date = utils::get_today();
    let time = utils::get_now();

    // 插入 unavailable timeslot 包含當前時間
    let mut tx = conn.transaction().unwrap();
    tx.execute(
      "INSERT INTO UnavailableTimeSlots (date, start_time, end_time) VALUES (?1, 0, ?2)",
      params![date, time + 3600],
    )
    .unwrap();
    tx.commit().unwrap();

    let result = show_current_seats_status().await;

    assert_eq!(result.unwrap(), "Unavailable");

    // DB error
    database::init_db();
    let db_path = format!("{}/SSR.db3", utils::get_root());
    std::fs::remove_file(db_path).unwrap();

    let result = show_current_seats_status().await;

    assert_eq!(result.unwrap_err(), Status::InternalServerError);
  }

  #[tokio::test]
  async fn test_show_seats_status_by_time_success() {
    let date = "2023-12-28";
    let start_time = 10000;
    let end_time = 12000;

    let result = show_seats_status_by_time(date, start_time, end_time).await;

    assert!(result.is_ok());

    let result_str = result.unwrap();
    let status: seat::AllSeatsStatus =
      serde_json::from_str(&result_str).expect("Result should be AllSeatsStatus JSON");

    assert_eq!(status.seats.len(), 217); // 應該有 217 個座位
  }

  #[tokio::test]
  async fn test_show_seats_status_by_time_invalid_datetime() {
    let date = "2023-12-28";
    let start_time = 22000; // 超出時間範圍
    let end_time = 25000;

    let result = show_seats_status_by_time(date, start_time, end_time).await;

    assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
  }

  #[tokio::test]
  async fn test_show_seats_status_by_time_db_error() {
    database::init_db();
    let db_path = format!("{}/SSR.db3", utils::get_root());
    std::fs::remove_file(db_path).unwrap();

    let date = "2023-12-28";
    let start_time = 10000;
    let end_time = 12000;

    let result = show_seats_status_by_time(date, start_time, end_time).await;

    assert_eq!(result.unwrap_err(), Status::InternalServerError);
  }

  #[tokio::test]
  async fn test_show_seat_reservations_success() {
    let date = "2023-12-28";
    let seat_id = 125;

    let result = show_seat_reservations(&date, seat_id).await;

    assert!(result.is_ok());

    let result_str = result.unwrap();
    let timeslots: Vec<(u32, u32)> =
      serde_json::from_str(&result_str).expect("Result should be timeslots JSON");

    // 檢查 seat_id 相等
    assert!(timeslots.iter().all(|(start, end)| start < end));
  }

  #[tokio::test]
  async fn test_show_seat_reservations_invalid_seat() {
    let date = "2023-12-28";
    let seat_id = 0; // 不存在的座位

    let result = show_seat_reservations(&date, seat_id).await;

    assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
  }

  #[tokio::test]
  async fn test_show_seat_reservations_invalid_date() {
    let date = "2023-15-32"; // 不存在的日期
    let seat_id = 120;

    let result = show_seat_reservations(&date, seat_id).await;

    assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
  }

  #[tokio::test]
  async fn test_show_seat_reservations_db_error() {
    database::init_db();
    let db_path = format!("{}/SSR.db3", utils::get_root());
    std::fs::remove_file(db_path).unwrap();

    let date = "2023-12-28";
    let seat_id = 120;

    let result = show_seat_reservations(&date, seat_id).await;

    assert_eq!(result.unwrap_err(), Status::InternalServerError);
  }
}
