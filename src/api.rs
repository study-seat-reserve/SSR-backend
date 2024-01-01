use crate::{
  database,
  model::{constant::*, *},
  utils::*,
};

use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{get, http::Status, post, serde::json::Json, State};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use validator::Validate;

// 註冊
#[post("/api/register", format = "json", data = "<user>")]
pub async fn register(
  pool: &State<Pool<Sqlite>>,
  user: Json<user::RegisterRequest>,
) -> Result<String, Status> {
  handle_validator(user.validate())?;

  let data: user::RegisterRequest = user.into_inner();
  let user_name = data.user_name;
  let password = data.password;
  let email = data.email;

  log::info!("Handling registration for user: {}", user_name);

  let password_hash = handle(hash(password, DEFAULT_COST), "Hashing password")?;
  let verification_token = Uuid::new_v4().to_string();
  let user_role = user::UserRole::RegularUser;
  let verified = false;

  let user_info = user::UserInfo {
    user_name: user_name.clone(),
    email: email.clone(),
    password_hash: password_hash,
    user_role: user_role,
    verified: verified,
    verification_token: verification_token.clone(),
  };

  database::user::insert_new_user_info(pool.inner(), user_info).await?;

  send_verification_email(&email, &verification_token)?;

  let token = create_resend_verification_token(&email, &verification_token, false)
    .map_err(|_| Status::InternalServerError)?;

  log::info!("Finished registration for user: {}", user_name);

  Ok(token)
}

#[get("/api/resend_verification")]
pub async fn resend_verification_email(
  claims: token::ResendVerificationClaim,
) -> Result<String, Status> {
  log::info!("Starting to process the resend verification email request");

  let email = claims.email;
  let verification_token = claims.verification_token;
  let expiration = claims.expiration;
  let now: i64 = get_now().timestamp();

  // 超過規定時間才可以重發驗證郵件
  if now <= expiration {
    log::warn!("Not yet reached the time limit for resending verification email, request denied");
    return Err(Status::BadRequest);
  }

  // 重發驗證信
  send_verification_email(&email, &verification_token)?;

  // 重新生成JWT token(is_resend設定為true) -> 重新計算重發驗證信的冷卻時間
  let token = create_resend_verification_token(&email, &verification_token, true)
    .map_err(|_| Status::InternalServerError)?;

  log::info!("Resend verification email to address: {}", &email);
  Ok(token)
}

#[get("/api/verify?<verification_token>")]
pub async fn email_verify(
  pool: &State<Pool<Sqlite>>,
  verification_token: String,
) -> Result<String, Status> {
  log::info!("Verifying email for token: {}", verification_token);

  database::user::update_user_verified_by_token(pool.inner(), &verification_token).await?;

  log::info!(
    "Email verification completed for token: {}",
    verification_token
  );

  Ok("Your email has been successfully verified.".to_string())
}

// 登入
#[post("/api/login", data = "<creds>")]
pub async fn login(
  pool: &State<Pool<Sqlite>>,
  creds: Json<user::LoginRequest>,
) -> Result<String, Status> {
  handle_validator(creds.validate())?;
  let user_info = database::user::get_user_info(pool.inner(), &creds.user_name).await?;

  log::info!(
    "Processing the login request for user: {}",
    &user_info.user_name
  );

  let password_matches =
    verify(&creds.password, &user_info.password_hash).map_err(|_| Status::InternalServerError)?;

  if !password_matches {
    log::warn!("Password is incorrect");
    return Err(Status::Unauthorized);
  }

  if !&user_info.verified {
    log::warn!("The user's email has not been verified");
    return Err(Status::BadRequest);
  }

  let is_in_blacklist =
    database::user::is_user_in_blacklist(pool.inner(), &user_info.user_name).await?;

  if is_in_blacklist {
    log::warn!(
      "User '{}' is currently in the blacklist.",
      &user_info.user_name
    );
    return Err(Status::Forbidden);
  }

  let token = create_userinfo_token(&user_info.user_name, user_info.user_role)
    .map_err(|_| Status::InternalServerError)?;

  log::info!("User: {} login successful", user_info.user_name);

  Ok(token)
}

// 查詢當前所有位置狀態
/*
如果座位(Seats)不可用，則該座位的狀態為Unavailable
如果特定時間被包含在某筆預約中，則則該座位的狀態為Borrowed
否則為Available
*/
#[get("/api/show_status")]
pub async fn show_current_seats_status(pool: &State<Pool<Sqlite>>) -> Result<String, Status> {
  log::info!("Show current seats status");

  let now: i64 = get_now().timestamp();
  let all_seats_status: seat::AllSeatsStatus;

  if database::timeslot::is_within_unavailable_timeslot(pool.inner(), now).await? {
    log::warn!(
      "The time: {} is within an unavailable timeslot",
      time_to_string(now)?
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
    all_seats_status = database::seat::get_all_seats_status(pool.inner(), now).await?;
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
#[get("/api/show_status/<start_time>/<end_time>")]
pub async fn show_seats_status_in_specific_timeslots(
  pool: &State<Pool<Sqlite>>,
  start_time: i64,
  end_time: i64,
) -> Result<String, Status> {
  log::info!("Show seats status by time");
  validate_datetime(start_time, end_time)?;

  let all_seats_status =
    database::seat::get_seats_status_in_specific_timeslots(pool.inner(), start_time, end_time)
      .await?;

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show seats status by time successfully");

  Ok(json)
}

// 查詢當前特定位置預約狀態
#[get("/api/show_reservations/<seat_id>/<start_time>/<end_time>")]
pub async fn show_seat_reservations(
  pool: &State<Pool<Sqlite>>,
  seat_id: u16,
  start_time: i64,
  end_time: i64,
) -> Result<String, Status> {
  log::info!("Show reservations of seat: {}", seat_id);

  validate_seat_id(seat_id)?;

  let timeslots =
    database::seat::get_seat_reservations(pool.inner(), start_time, end_time, seat_id).await?;

  let json = handle(
    serde_json::to_string(&timeslots),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show reservations of seat: {} successfully", seat_id);

  Ok(json)
}

// 預約座位
#[post("/api/reserve", format = "json", data = "<insert_reservation>")]
pub async fn reserve_seat(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  insert_reservation: Json<reservation::InsertReservationRequest>,
) -> Result<(), Status> {
  handle_validator(insert_reservation.validate())?;

  let data: reservation::InsertReservationRequest = insert_reservation.into_inner();
  let seat_id = data.seat_id;
  let start_time = data.start_time;
  let end_time = data.end_time;
  let user_name = claims.user;

  log::info!("Reserving a seat :{} for user: {}", seat_id, user_name);

  if !database::seat::is_seat_available(pool.inner(), seat_id).await? {
    log::warn!("The seat: {} is unavailable", seat_id);
    return Err(Status::BadRequest);
  }

  if database::timeslot::is_overlapping_with_unavailable_timeslot(
    pool.inner(),
    start_time,
    end_time,
  )
  .await?
  {
    log::warn!(
      "The start_time: {} end_time: {} is overlapping unavailable timeslot",
      start_time,
      end_time
    );
    return Err(Status::BadRequest);
  }

  let date = timestamp_to_naive_datetime(start_time)?.date();

  if database::reservation::check_unfinished_reservations(pool.inner(), &user_name, date).await? {
    log::warn!("Found ongoing reservation not yet completed");
    return Err(Status::BadRequest);
  }

  database::reservation::reserve_seat(pool.inner(), &user_name, seat_id, start_time, end_time)
    .await?;

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
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  update_reservation: Json<reservation::UpdateReservationRequest>,
) -> Result<(), Status> {
  handle_validator(update_reservation.validate())?;

  let data: reservation::UpdateReservationRequest = update_reservation.into_inner();
  let start_time = data.start_time;
  let end_time = data.end_time;
  let new_start_time = data.new_start_time;
  let new_end_time = data.new_end_time;
  let user_name = claims.user;

  log::info!("Updating reservation for user: {}", user_name);

  if database::timeslot::is_overlapping_with_unavailable_timeslot(
    pool.inner(),
    new_start_time,
    new_end_time,
  )
  .await?
  {
    return Err(Status::BadRequest);
  }

  database::reservation::update_reservation_time(
    pool.inner(),
    &user_name,
    start_time,
    end_time,
    new_start_time,
    new_end_time,
  )
  .await?;

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
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  delete_reservation: Json<reservation::DeleteReservationRequest>,
) -> Result<(), Status> {
  handle_validator(delete_reservation.validate())?;

  let data: reservation::DeleteReservationRequest = delete_reservation.into_inner();
  let start_time = data.start_time;
  let end_time = data.end_time;
  let user_name = claims.user;

  log::info!("Deleting reservation for user: {}", user_name);

  database::reservation::delete_reservation_time(pool.inner(), &user_name, start_time, end_time)
    .await?;

  log::info!("Reservation for user: {} deleted successfully", user_name);

  Ok(())
}

// 顯示使用者預約時段
#[get("/api/user_reservations")]
pub async fn display_user_reservations(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
) -> Result<Json<Vec<reservation::Reservation>>, Status> {
  let user_name = claims.user;

  log::info!("Displaying the user's reservations");

  let reservations = database::reservation::get_user_reservations(pool.inner(), &user_name).await?;

  log::info!("Displaying the user's reservations successfully");

  Ok(Json(reservations))
}

// 設定不可預約時間
#[post("/api/set_timeslots", format = "json", data = "<time_slot>")]
pub async fn set_unavailable_timeslots(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  time_slot: Json<timeslot::TimeSlot>,
) -> Result<(), Status> {
  handle_validator(time_slot.validate())?;

  let user_name = claims.user;
  if claims.role != user::UserRole::Admin {
    log::warn!(
      "Unauthorized attempt to set unavailable timeslots by user: {}",
      &user_name
    );
    return Err(Status::Unauthorized);
  }

  let start_time = time_slot.start_time;
  let end_time = time_slot.end_time;

  log::info!(
    "Setting unavailable timeslot start_time: {:?}, end_time: {:?}",
    start_time,
    end_time
  );

  database::timeslot::insert_unavailable_timeslot(pool.inner(), start_time, end_time).await?;

  log::info!("Unavailable timeslot set successfully");
  Ok(())
}

// 設定不可使用座位
#[post(
  "/api/set_seat_availability",
  format = "json",
  data = "<seat_availability>"
)]
pub async fn set_seat_availability(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  seat_availability: Json<seat::SeatAvailabilityRequest>,
) -> Result<(), Status> {
  handle_validator(seat_availability.validate())?;

  let user_name = claims.user;
  if claims.role != user::UserRole::Admin {
    log::warn!(
      "Unauthorized attempt to set seat availability by user: {}",
      &user_name
    );
    return Err(Status::Unauthorized);
  }

  let seat_id = seat_availability.seat_id;
  let available = seat_availability.available;

  log::info!(
    "Setting seat availability seat_id: {:?}, available: {:?}",
    seat_id,
    available
  );

  database::seat::update_seat_availability(pool.inner(), seat_id, available).await?;

  log::info!("Seat availability set successfully");
  Ok(())
}

#[post("/api/set_blacklist", format = "json", data = "<ban_request>")]
pub async fn add_user_to_blacklist(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  ban_request: Json<user::BanRequest>,
) -> Result<(), Status> {
  handle_validator(ban_request.validate())?;

  let user_name = claims.user;
  if claims.role != user::UserRole::Admin {
    log::warn!(
      "Unauthorized attempt to add user to blacklist by user: {}",
      &user_name
    );
    return Err(Status::Unauthorized);
  }

  let user_name_to_ban = &ban_request.user_name;
  let start_time = ban_request.start_time;
  let end_time = ban_request.end_time;

  log::info!(
    "Adding user to blacklist with start_time: {:?}, end_time: {:?}",
    start_time,
    end_time
  );

  let is_user_in_blacklist =
    database::user::is_user_in_blacklist(pool.inner(), user_name_to_ban).await?;
  if is_user_in_blacklist {
    database::user::delete_user_from_blacklist(pool.inner(), user_name_to_ban).await?;
  }

  database::user::insert_user_to_blacklist(pool.inner(), user_name_to_ban, start_time, end_time)
    .await?;

  log::info!("Add user to blacklist successfully");
  Ok(())
}

// 將使用者移除黑名單
#[post("/api/remove_blacklist", format = "json", data = "<unban_request>")]
pub async fn remove_user_from_blacklist(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  unban_request: Json<user::UnBanRequest>,
) -> Result<(), Status> {
  let user_name = claims.user;
  if claims.role != user::UserRole::Admin {
    log::warn!(
      "Unauthorized attempt to remove user from blacklist by user: {}",
      &user_name
    );
    return Err(Status::Unauthorized);
  }

  let user_name_to_unban = &unban_request.user_name;

  log::info!("Removing user from blacklist");

  database::user::delete_user_from_blacklist(pool.inner(), user_name_to_unban).await?;

  log::info!("Remove user from blacklist successfully");
  Ok(())
}
/*
管理者設定unavaliable time slot
管理者設定座位avaliable
黑名單
*/

// fn register -> O
// fn resend_verification_email
// fn email_verify
// fn login -> O
// fn show_current_seats_status
// fn show_seats_status_in_specific_timeslots
// fn show_seat_reservations
// fn reserve_seat
// fn update_reservation
// fn delete_reservation_time
// fn display_user_reservations
// fn set_unavailable_timeslots
// fn set_seat_availability

#[cfg(test)]
mod tests {

  use crate::utils;

  use super::*;
  use crate::model::reservation;
  use rocket::http::ContentType;
  use rocket::http::Status as HttpStatus;
  use rocket::http::Status;
  use rocket::local::blocking::Client;
  use rocket::serde::json::json;
  use rocket::uri;
  use sqlx::SqlitePool;

  #[test]
  fn test_register() {
    // Valid
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

    assert_eq!(response.status(), Status::Ok);

    // Invalid
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let user_data = json!({
        "user_name": "istestuser",
        "password": "test",
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

  #[tokio::test]
  async fn test_resend_verification_email() {
    let email = "test@email.ntou.edu.tw".to_string();
    let verification_token = "testtoken".to_string();
    let expiration = utils::get_now().timestamp() + 60;

    let claims = token::ResendVerificationClaim {
      email: email.clone(),
      verification_token: verification_token.clone(),
      expiration: expiration,
      exp: 12345678,
    };
    // let req = LocalRequest::with_header("Authorization", "Bearer dummytoken").to_request();

    // let res = claims.from_request(&req).await;
    // assert!(res.is_success());
    // let claims = res.unwrap();

    // let result = resend_verification_email(claims).await;

    // assert!(result.is_ok());

    // let email_address = std::env::var("EMAIL_ADDRESS").unwrap();
    // assert_eq!(email_address, "nick20030829@gmail.com");
    // assert_eq!(1, mails_sent()); // 寄信數量應為1

    // let token = result.unwrap();
    // let new_claims = token::UserInfoClaim::verify_jwt(&token).unwrap();

    // assert_eq!(new_claims.email, email);
    // assert_eq!(new_claims.verification_token, verification_token);
    // assert!(new_claims.expiration > expiration); //冷卻時間應該增加
  }

  #[test]
  fn test_email_verify() {
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

    // 遺失 token
    let client = Client::tracked(rocket::build()).expect("valid rocket instance");
    let base_url = get_base_url();

    let response = client.get(format!("{}/api/verify", base_url)).dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    // 驗證失敗
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

  #[tokio::test]
  async fn test_login() {
    let client = Client::tracked(rocket::build()).unwrap();

    // 正確登入
    let user = json!({
      "user_name": "testuser",
      "password": "password123"
    });

    let response = client
      .post("/api/login")
      .header(ContentType::JSON)
      .json(&user)
      .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // 錯誤密碼
    let wrong_password = json!({
       "user_name": "testuser",
       "password": "wrongpassword"
    });

    let response = client
      .post("/api/login")
      .header(ContentType::JSON)
      .json(&wrong_password)
      .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    // 未驗證用戶
    let unverified = json!({
      "user_name": "unverified",
      "password": "password123"
    });

    let response = client
      .post("/api/login")
      .header(ContentType::JSON)
      .json(&unverified)
      .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    // Error
    let client = Client::tracked(rocket::build()).unwrap();

    let no_username = json!({
       "user_name": "",
       "password": "password"
    });

    let response = client
      .post("/api/login")
      .header(ContentType::JSON)
      .json(&no_username)
      .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);

    let no_password = json!({
       "user_name": "testuser",
       "password": ""
    });

    let response = client
      .post("/api/login")
      .header(ContentType::JSON)
      .json(&no_password)
      .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
  }

  #[tokio::test]
  async fn test_show_current_seats_status() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let rocket = rocket::build().manage(pool.clone());
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/api/show_status").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: seat::AllSeatsStatus =
      serde_json::from_str(&response.into_string().unwrap()).unwrap();

    assert_eq!(body.seats.len(), (NUMBER_OF_SEATS - 1) as usize);

    for seat_status in body.seats {
      match seat_status.status {
        seat::Status::Available => {
          assert!(seat_status.seat_id > 0);
        }
        seat::Status::Borrowed => {
          assert!(seat_status.seat_id > 0);
        }
        seat::Status::Unavailable => {
          assert!(seat_status.seat_id > 0);
        }
      }
    }
  }

  #[tokio::test]
  async fn test_show_seats_status_in_specific_timeslots() {
    // let rocket = rocket::build();
    // let client = Client::tracked(rocket).expect("valid rocket instance");

    // let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // let pool = rocket.manage(pool);

    // let start_time = 1672531200;
    // let end_time = 1672534800;

    // let response = client
    //   .get(uri!(show_seats_status_in_specific_timeslots: pool.inner(), start_time, end_time))
    //   .dispatch();

    // assert_eq!(response.status(), Status::Ok);

    // let result: seat::AllSeatsStatus =
    //   serde_json::from_str(&response.into_string().unwrap()).unwrap();

    // assert_eq!(
    //   result.seats.len(),
    //   usize::from(constant::NUMBER_OF_SEATS - 1)
    // );

    // for seat in result.seats {
    //   assert!(seat.seat_id > 0 && seat.seat_id <= NUMBER_OF_SEATS); // 座位號碼正確性

    //   match seat.status {
    //     seat::Status::Available => (),
    //     seat::Status::Borrowed => (),
    //     seat::Status::Unavailable => (),
    //     _ => panic!("Invalid seat status"),
    //   }
    // }
  }

  #[tokio::test]
  async fn test_show_seat_reservations() {
    // let rocket = rocket::build();
    // let client = Client::tracked(rocket).expect("valid rocket instance");
    // let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("in-memory database");

    // let mut conn = pool.acquire().await.expect("acquire connection");
    // sqlx::query("INSERT INTO Seats (seat_id, available) VALUES (1, 1)")
    //   .execute(&mut *conn)
    //   .await
    //   .expect("insert seat");

    // sqlx::query(
    //   "INSERT INTO Reservations (seat_id, start_time, end_time) VALUES (1, 1672531200, 1672534800)",
    // )
    // .execute(&mut *conn)
    // .await
    // .expect("insert reservation");

    // let seat_id = 1;
    // let start_time = 1672530000;
    // let end_time = 1672536000;

    // let response = client
    //   .get(uri!(show_seat_reservations: *pool, seat_id, start_time, end_time))
    //   .dispatch();

    // assert_eq!(response.status(), Status::Ok);

    // let body: Vec<(i64, i64)> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    // assert_eq!(body, vec![(1672531200, 1672534800)]);
  }

  #[tokio::test]
  async fn test_reserve_seat() {}

  #[tokio::test]
  async fn test_update_reservation() {
    let rocket = rocket::build();
    let client = Client::tracked(rocket).expect("valid rocket instance");

    // 初始化資料庫連線池
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let pool = rocket.manage(pool);

    // 準備測試資料
    let user_name = "testuser";
    let user_role = user::UserRole::RegularUser;
    let exp = 1234567890;

    let start_time = 1234567890;
    let end_time = 1234567899;
    let new_start_time = 1234567901;
    let new_end_time = 1234567999;

    let update_reservation = reservation::UpdateReservationRequest {
      start_time,
      end_time,
      new_start_time,
      new_end_time,
    };

    let user_claims = token::UserInfoClaim {
      user: user_name.to_string(),
      role: user_role,
      exp,
    };

    sqlx::query!(
      "INSERT INTO Reservations (user_name, seat_id, start_time, end_time)
     VALUES (?, 1, datetime(?, '+8 hours'), datetime(?, '+8 hours'))",
      user_name,
      start_time,
      end_time
    )
    .execute(&pool)
    .await
    .unwrap();

    // 呼叫測試函數
    let response = client
      .post("/api/update_reservation")
      .header(ContentType::JSON)
      .json(&update_reservation)
      .dispatch();

    // 檢查返回結果
    assert_eq!(response.status(), Status::Ok);

    // 檢查資料庫資料是否正確更新
    let rows = sqlx::query!(
      "SELECT start_time, end_time 
     FROM Reservations 
     WHERE user_name = ?",
      user_name
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].start_time, new_start_time.to_string());
    assert_eq!(rows[0].end_time, new_end_time.to_string());
  }

  #[tokio::test]
  async fn test_delete_reservation_time() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // 準備測試資料
    let user_name = "test_user";
    let user_role = user::UserRole::RegularUser;
    let exp = 1234567890;

    let start_time = 1234567890;
    let end_time = 1234567899;

    let delete_reservation = reservation::DeleteReservationRequest {
      start_time,
      end_time,
    };

    let user_claims = token::UserInfoClaim {
      user: user_name.to_string(),
      role: user_role,
      exp,
    };

    sqlx::query!(
      "INSERT INTO Reservations (user_name, seat_id, start_time, end_time)
     VALUES (?, 1, datetime(?, '+8 hours'), datetime(?, '+8 hours'))",
      user_name,
      start_time,
      end_time
    )
    .execute(&pool)
    .await
    .unwrap();

    // let result = delete_reservation_time(
    //   &rocket::State::from(pool.clone()),
    //   user_claims,
    //   Json(delete_reservation),
    // )
    // .await;

    // // 檢查返回結果
    // assert!(result.is_ok());

    let rows = sqlx::query!(
    "SELECT * FROM Reservations WHERE user_name = ? AND start_time = datetime(?, '+8 hours') AND end_time = datetime(?, '+8 hours')",
    user_name, start_time, end_time
  )
  .fetch_all(&pool)
  .await
  .unwrap();

    assert_eq!(rows.len(), 0);
  }

  #[rocket::async_test]
  async fn test_display_user_reservations() {
    // let database_url = ":memory:";
    // let pool = SqlitePool::connect(&database_url).await;
    // std::env::set_var("DATABASE_URL", database_url);
    // rocket::custom(rocket::Config::from_env("test"))
    //   .manage(pool)
    //   .launch();

    // let client = Client::tracked(rocket()).unwrap();

    // let user_name = "test_user";
    // let start_time = 1672531200;
    // let end_time = 1672534800;

    // let insert_request = InsertReservationRequest {
    //   seat_id: 1,
    //   start_time,
    //   end_time,
    // };

    // reserve_seat(&client.rocket().state(), user_name, insert_request)
    //   .await
    //   .unwrap();

    // let response = client
    //   .get(format!("/api/user_reservations"))
    //   .header(ContentType::JSON)
    //   .dispatch();

    // assert_eq!(response.status(), Status::Ok);

    // let reservations: Vec<Reservation> =
    //   serde_json::from_str(&response.into_string().unwrap()).unwrap();

    // assert!(!reservations.is_empty());

    // let retrieved_reservation = &reservations[0];
    // assert_eq!(retrieved_reservation.seat_id, insert_request.seat_id);
  }

  #[tokio::test]
  async fn test_set_unavailable_timeslots() {}

  #[tokio::test]
  async fn test_set_seat_availability() {}
}

/////////////////////////////////////////////////////////////
// async fn test_show_seats_status_by_time_db_error() {
//   database::init_db();
//   let db_path = format!("{}/SSR.db3", utils::get_root());
//   std::fs::remove_file(db_path).unwrap();

//   let date = "2023-12-28";
//   let start_time = 10000;
//   let end_time = 12000;

//   let result = show_seats_status_by_time(date, start_time, end_time).await;

//   assert_eq!(result.unwrap_err(), Status::InternalServerError);
// }

// #[tokio::test]
// async fn test_show_seat_reservations_success() {
//   let date = "2023-12-28";
//   let seat_id = 125;

//   let result = show_seat_reservations(&date, seat_id).await;

//   assert!(result.is_ok());

//   let result_str = result.unwrap();
//   let timeslots: Vec<(u32, u32)> =
//     serde_json::from_str(&result_str).expect("Result should be timeslots JSON");

//   // 檢查 seat_id 相等
//   assert!(timeslots.iter().all(|(start, end)| start < end));
// }

// #[tokio::test]
// async fn test_show_seat_reservations_invalid_seat() {
//   let date = "2023-12-28";
//   let seat_id = 0; // 不存在的座位

//   let result = show_seat_reservations(&date, seat_id).await;

//   assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
// }

// #[tokio::test]
// async fn test_show_seat_reservations_invalid_date() {
//   let date = "2023-15-32"; // 不存在的日期
//   let seat_id = 120;

//   let result = show_seat_reservations(&date, seat_id).await;

//   assert_eq!(result.unwrap_err(), Status::UnprocessableEntity);
// }

// #[tokio::test]
// async fn test_show_seat_reservations_db_error() {
//   database::init_db();
//   let db_path = format!("{}/SSR.db3", utils::get_root());
//   std::fs::remove_file(db_path).unwrap();

//   let date = "2023-12-28";
//   let seat_id = 120;

//   let result = show_seat_reservations(&date, seat_id).await;

//   assert_eq!(result.unwrap_err(), Status::InternalServerError);
// }}
