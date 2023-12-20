use super::{common::*, validate_utils::*};

#[derive(Debug, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_timeslot", skip_on_field_errors = false))]
pub struct TimeSlot {
  pub start_time: i64,
  pub end_time: i64,
}

fn validate_timeslot(timeslot: &TimeSlot) -> Result<(), ValidationError> {
  let start_time = timeslot.start_time;
  let end_time = timeslot.end_time;

  validate_datetime(start_time, end_time)
}
