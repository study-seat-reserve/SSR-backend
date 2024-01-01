use super::{common::*, validate_utils::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct Seat {
  pub seat_id: u16,
  pub available: bool,
  pub other_info: Option<String>,
}

#[derive(Serialize)]
pub struct SeatStatus {
  pub seat_id: u16,
  pub status: Status,
}

#[derive(Serialize)]
pub struct AllSeatsStatus {
  pub seats: Vec<SeatStatus>,
}

#[derive(Serialize)]
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
