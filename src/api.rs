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
#[post("/api/register", format = "json", data = "<data>")]
pub async fn register(pool: &State<Pool<Sqlite>>, data: Json<user::User>) -> Result<(), Status> {
  handle_validator(data.validate())?;

  let user_name = &data.user_name;
  let password = &data.password;
  let email = &data.email;

  log::info!("Handling registration for user: {}", user_name);

  let password_hash = handle(hash(password, DEFAULT_COST), "Hashing password")?;
  let verification_token = Uuid::new_v4().to_string();

  database::insert_new_user_info(
    pool.inner(),
    user_name,
    &password_hash,
    email,
    &verification_token,
  )
  .await?;

  let url = format!(
    "{}/api/verify?verification_token={}",
    get_base_url(),
    verification_token
  );

  send_verification_email(&email, &url)?;

  log::info!("Finished registration for user: {}", user_name);

  Ok(())
}

// #[get("/api/resend_verification_email/<user_name>")]
// pub fn resend_verification_email(user_name: String) -> Result<(), Status> {
//   //log

//   let url = format!(
//     "{}/api/verify?verification_token={}",
//     get_base_url(),
//     verification_token
//   );

//   send_verification_email(&email, &url)?;

//   log::info!("Finished registration for user: {}", user_name);

//   Ok(())
// }

#[get("/api/verify?<verification_token>")]
pub async fn email_verify(
  pool: &State<Pool<Sqlite>>,
  verification_token: String,
) -> Result<String, Status> {
  log::info!("Verifying email for token: {}", verification_token);

  database::update_user_verified_by_token(pool.inner(), &verification_token).await?;

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
  creds: Json<user::LoginCreds>,
) -> Result<String, Status> {
  handle_validator(creds.validate())?;
  log::info!("Processing the login request");

  let user_info = database::get_user_info(pool.inner(), &creds.user_name).await?;
  let username = user_info.user_name.clone();
  let password_hash = &user_info.password_hash;
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
pub async fn show_current_seats_status(pool: &State<Pool<Sqlite>>) -> Result<String, Status> {
  log::info!("Show current seats status");

  let date = get_today();
  let now = get_now();
  let all_seats_status: seat::AllSeatsStatus;

  if database::is_within_unavailable_timeslot(pool, date, now).await? {
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
    all_seats_status = database::get_all_seats_status(pool.inner(), date, now).await?;
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
  pool: &State<Pool<Sqlite>>,
  date: &str,
  start_time: u32,
  end_time: u32,
) -> Result<String, Status> {
  log::info!("Show seats status by time");

  let date = date_from_string(date)?;
  validate_datetime(date, start_time, end_time)?;

  let all_seats_status =
    database::get_seats_status_by_time(pool, date, start_time, end_time).await?;

  let json = handle(
    serde_json::to_string(&all_seats_status),
    "Serialize the data as a String of JSON",
  )?;

  log::info!("Show seats status by time successfully");

  Ok(json)
}

// 查詢當前特定位置預約狀態
#[get("/api/show_reservations/<date>/<seat_id>")]
pub async fn show_seat_reservations(
  pool: &State<Pool<Sqlite>>,
  date: &str,
  seat_id: u16,
) -> Result<String, Status> {
  log::info!("Show reservations of seat: {}", seat_id);

  let date = date_from_string(date)?;
  validate_date(date)?;
  validate_seat_id(seat_id)?;

  let timeslots = database::get_seat_reservations(pool, date, seat_id).await?;

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
  pool: &State<Pool<Sqlite>>,
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

  if !database::is_seat_available(pool, seat_id).await? {
    log::warn!("The seat: {} is unavailable", seat_id);
    return Err(Status::UnprocessableEntity);
  }

  if database::is_overlapping_with_unavailable_timeslot(pool, date, start_time, end_time).await? {
    log::warn!(
      "The date: {} start_time: {} end_time: {} is overlapping unavailable time slot",
      date,
      start_time,
      end_time
    );
    return Err(Status::Conflict);
  }

  database::reserve_seat(pool, &user_name, seat_id, date, start_time, end_time).await?;

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

  if database::is_overlapping_with_unavailable_timeslot(pool, date, new_start_time, new_end_time)
    .await?
  {
    return Err(Status::Conflict);
  }

  database::update_reservation_time(pool, &user_name, date, new_start_time, new_end_time).await?;

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
  claims: token::Claims,
  delete_reservation: Json<reservation::DeleteReservation>,
) -> Result<(), Status> {
  handle_validator(delete_reservation.validate())?;
  let delete_data: reservation::DeleteReservation = delete_reservation.into_inner();
  let date = delete_data.date;
  let user_name = claims.user;

  log::info!("Deleting reservation for user: {}", user_name);

  database::delete_reservation_time(pool, &user_name, date).await?;

  log::info!("Reservation for user: {} deleted successfully", user_name);

  Ok(())
}

// 顯示使用者預約時段
#[get("/api/user_reservations")]
pub async fn display_user_reservations(
  pool: &State<Pool<Sqlite>>,
  claims: token::Claims,
) -> Result<Json<Vec<reservation::Reservation>>, Status> {
  let user_name = claims.user;

  log::info!("Displaying the user's reservations");

  let reservations = database::get_user_reservations(pool, &user_name).await?;

  log::info!("Displaying the user's reservations successfully");

  Ok(Json(reservations))
}

#[post("/api/set_timeslots")]
pub async fn set_unavailable_timeslots() {}
/*
管理者設定unavaliable time slot
管理者設定座位avaliable
黑名單
使用者改名
*/
