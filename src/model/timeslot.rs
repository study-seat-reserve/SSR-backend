use chrono::NaiveDate;

#[derive(Debug)]
pub struct TimeSlot {
  pub date: NaiveDate,
  pub start_time: u32,
  pub end_time: u32,
}
