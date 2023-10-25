use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct TimeSlot {
  pub date: NaiveDate,
  pub start_time: NaiveTime,
  pub end_time: NaiveTime,
  pub Availability: bool,
}
