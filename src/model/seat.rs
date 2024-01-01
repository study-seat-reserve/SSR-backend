use super::{common::*, validate_utils::*};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Seat {
  pub seat_id: u16,
  pub available: bool,
  pub other_info: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SeatStatus {
  pub seat_id: u16,
  pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub struct AllSeatsStatus {
  pub seats: Vec<SeatStatus>,
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub enum Status {
  Available,
  Unavailable,
  Borrowed,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SeatAvailabilityRequest {
  #[validate(custom = "validate_seat_id")]
  pub seat_id: u16,
  pub available: bool,
}

impl ToString for Status {
  fn to_string(&self) -> String {
    match *self {
      Status::Available => "Available".to_owned(),
      Status::Unavailable => "Unavailable".to_owned(),
      Status::Borrowed => "Borrowed".to_owned(),
    }
  }
}

impl FromStr for Status {
  type Err = std::io::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Available" => Ok(Status::Available),
      "Unavailable" => Ok(Status::Unavailable),
      "Borrowed" => Ok(Status::Borrowed),
      _ => Err(std::io::Error::new(
        ErrorKind::InvalidInput,
        "Provided string does not match any Status variant",
      )),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_seat_serial_deserial() {
    let seat = Seat {
      seat_id: 109, // Maximum u16 value
      available: true,
      other_info: Some("Extra information".to_string()),
    };

    let json = serde_json::to_string(&seat).unwrap();
    let deserialized_seat: Seat = serde_json::from_str(&json).unwrap();

    assert_eq!(seat, deserialized_seat);
  }

  #[test]
  fn test_seat() {
    let seat = Seat {
      seat_id: 109,
      available: true,
      other_info: None,
    };

    assert_eq!(seat.other_info, None);
  }

  #[test]
  fn test_to_string() {
    let status = Status::Available;
    assert_eq!(status.to_string(), "Available");

    let status = Status::Unavailable;
    assert_eq!(status.to_string(), "Unavailable");

    let status = Status::Borrowed;
    assert_eq!(status.to_string(), "Borrowed");
  }

  #[test]
  fn test_from_str() {
    let status = Status::from_str("Available").unwrap();
    assert_eq!(status, Status::Available);

    let status = Status::from_str("Unavailable").unwrap();
    assert_eq!(status, Status::Unavailable);

    let status = Status::from_str("Borrowed").unwrap();
    assert_eq!(status, Status::Borrowed);

    let result = Status::from_str("Invalid");
    assert!(result.is_err());
  }

  #[test]
  fn test_seat_availability() {
    let request = SeatAvailabilityRequest {
      seat_id: 109,
      available: true,
    };
    assert!((&request).validate().is_ok());

    let request = SeatAvailabilityRequest {
      seat_id: 0,
      available: true,
    };
    assert!((&request).validate().is_err());

    let request = SeatAvailabilityRequest {
      seat_id: 218,
      available: true,
    };
    assert!((&request).validate().is_err());
  }
}
