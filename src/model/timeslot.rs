use super::validate_utils::{validate_date, validate_datetime};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_timeslot", skip_on_field_errors = false))]
pub struct TimeSlot {
  #[validate(custom = "validate_date")]
  pub date: NaiveDate,
  pub start_time: u32,
  pub end_time: u32,
}

fn validate_timeslot(timeslot: &TimeSlot) -> Result<(), ValidationError> {
  let date: NaiveDate = timeslot.date;
  let start_time: u32 = timeslot.start_time;
  let end_time: u32 = timeslot.end_time;

  validate_datetime(date, start_time, end_time)
}
