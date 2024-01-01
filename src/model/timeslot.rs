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
  use super::*;
  use crate::utils::{get_now, naive_date_to_timestamp};
  use chrono::Timelike;

  #[test]
  fn test_validate_timeslot() {
    //結束時間晚於開始時間
    let start_time = 1658401200;
    let end_time = 1658424000;

    let timeslot = TimeSlot {
      start_time,
      end_time,
    };

    assert!(validate_timeslot(&timeslot).is_ok());
    //結束時間早於開始時間
    let start_time = 1658424000;
    let end_time = 1658401200;

    let timeslot = TimeSlot {
      start_time,
      end_time,
    };

    assert!(validate_timeslot(&timeslot).is_err());

    // 開始時間早於現在
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
