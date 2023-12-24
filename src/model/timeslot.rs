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

#[cfg(test)]
mod tests {

  use chrono::Timelike;

  use crate::utils::{get_now, naive_date_to_timestamp};

  use super::*;

  #[test]
  fn test_validate_timeslot_success() {
    let start_time = 1658401200; // 2023-07-20 10:00:00
    let end_time = 1658424000; // 2023-07-20 14:00:00

    let timeslot = TimeSlot {
      start_time,
      end_time,
    };

    assert!(validate_timeslot(&timeslot).is_ok());

    let start_time = 1658424000; // 2023-07-20 14:00:00
    let end_time = 1658401200; // 2023-07-20 10:00:00

    let timeslot = TimeSlot {
      start_time,
      end_time,
    };

    assert!(validate_timeslot(&timeslot).is_err());

    let now = get_now();
    let start_time =
      naive_date_to_timestamp(now.date(), now.hour(), now.minute() - 10, now.second()).unwrap();
    let end_time = start_time + 3600;

    let timeslot = TimeSlot {
      start_time,
      end_time,
    };

    assert!(validate_timeslot(&timeslot).is_err());
  }
}
