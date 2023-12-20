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
    log::warn!("The user's email has not been verified.");
    return Err(Status::BadRequest);
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
#[post("/api/set_timeslots", format = "json", data = "<timeslot>")]
pub async fn set_unavailable_timeslots(
  pool: &State<Pool<Sqlite>>,
  claims: token::UserInfoClaim,
  timeslot: Json<timeslot::TimeSlot>,
) -> Result<(), Status> {
  handle_validator(timeslot.validate())?;

  if claims.role != user::UserRole::Admin {
    log::warn!("");
    // return;
  }

  let user_name = claims.user;
  log::info!("Handling registration for user: {}", user_name);

  // database::insert_unavailable_timeslots()
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
  if claims.role != user::UserRole::Admin {
    log::warn!("");
    // return;
  }

  let user_name = claims.user;
  log::info!("Handling registration for user: {}", user_name);

  // database::update_seat_availability()
  Ok(())
}

/*
管理者設定unavaliable time slot
管理者設定座位avaliable
黑名單
使用者改名
*/
