use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Reservation {
  pub user_id: String,
  pub seat_id: u16,
  pub date: NaiveDate,
  pub start_time: NaiveTime,
  pub end_time: NaiveTime,
}
