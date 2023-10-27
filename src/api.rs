use crate::{database, model::*, utils::*};

use bcrypt::verify;
use rocket::{form::Form, fs::NamedFile, get, http::Status, post, serde::json::Json, State};
use uuid::Uuid;
use validator::Validate;

// 註冊
#[post("/api/register", format = "json", data = "<data>")]
pub async fn register(data: Json<user::User>) -> Result<(), Status> {
  handle_validator(data.validate())?;
  let user_id = Uuid::new_v4().to_string();

  log::info!("Handling registration for user: {}", &user_id);

  let user = data.into_inner();

  //
  // let user_name_exists = handle(
  //   database::check_if_user_name_exists(&user_name),
  //   "Checking if username already exists",
  // )
  // .map_err(|e| error_to_status(e))?;

  // if user_name_exists == true {
  //   log::warn!("Username '{}' already exists", &user_name);
  //   return Err(Status::Conflict);
  // }
  //

  database::insert_new_user_info(user, &user_id)?;

  //TODO: 信箱驗證

  log::info!("Finished registration for user: {}", &user_id);
  Ok(())
}

// 登入
pub async fn login() {
  // let valid = verify("hunter2", &hashed).unwrap();
}

// 查詢當前所有位置狀態 + filiter
pub async fn show_status_of_all_seats() {}

// 查詢當前特定位置預約狀態
pub async fn show_status_of_the_seat() {}

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
