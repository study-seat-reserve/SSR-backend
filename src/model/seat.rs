use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr, string::ToString};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Seat {
  pub seat_id: u16,
  pub available: bool,
  pub other_info: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SeatStatus {
  pub seat_id: u16,
  pub status: Status,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct AllSeatsStatus {
  pub seats: Vec<SeatStatus>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Status {
  Available,
  Unavailable,
  Borrowed,
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

//TEST

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::ErrorKind;

  #[test]
  fn test_status_to_string() {
    let status = Status::Available;
    assert_eq!(status.to_string(), "Available");

    let status = Status::Unavailable;
    assert_eq!(status.to_string(), "Unavailable");

    let status = Status::Borrowed;
    assert_eq!(status.to_string(), "Borrowed");
  }

  #[test]
  fn test_status_from_str() {
    let result = Status::from_str("Available");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Status::Available);

    let result = Status::from_str("Unavailable");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Status::Unavailable);

    let result = Status::from_str("Borrowed");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Status::Borrowed);

    let result = Status::from_str("Invalid");
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().kind(), ErrorKind::InvalidInput);
  }

  #[test]
  fn test_seat_serialization() {
    let seat = Seat {
      seat_id: 5,
      available: true,
      other_info: None,
    };

    let serialized = serde_json::to_string(&seat).unwrap();
    let deserialized: Seat = serde_json::from_str(&serialized).unwrap();

    assert_eq!(seat, deserialized);
  }

  #[test]
  fn test_seat_status_serialization() {
    let status = Status::Available;
    let seat_status = SeatStatus {
      seat_id: 1,
      status: status.clone(),
    };

    let serialized = serde_json::to_string(&seat_status).unwrap();
    let deserialized: SeatStatus = serde_json::from_str(&serialized).unwrap();

    assert_eq!(seat_status.seat_id, deserialized.seat_id);
    assert_eq!(seat_status.status, deserialized.status);
  }

  #[test]
  fn test_all_seats_status_serialization() {
    let seats = vec![
      SeatStatus {
        seat_id: 1,
        status: Status::Available,
      },
      SeatStatus {
        seat_id: 2,
        status: Status::Unavailable,
      },
    ];

    let all_seats_status = AllSeatsStatus { seats };

    let serialized = serde_json::to_string(&all_seats_status).unwrap();
    let deserialized: AllSeatsStatus = serde_json::from_str(&serialized).unwrap();

    assert_eq!(all_seats_status.seats, deserialized.seats);
  }
}
