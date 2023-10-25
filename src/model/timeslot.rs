use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSlot {
  pub date: NaiveDate,
  pub start_time: NaiveTime,
  pub end_time: NaiveTime,
  pub Availability: bool,
}
