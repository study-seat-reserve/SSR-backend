use crate::utils::*;

use rocket::{form::Form, fs::NamedFile, get, http::Status, post, serde::json::Json, State};

// 查詢當前所有位置狀態 + filiter
pub async fn show_status_of_all_seats() {}

// 註冊
pub async fn register() {}

// 登入
pub async fn login() {}

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
