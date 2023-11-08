use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, Serialize, Deserialize)]
pub struct Seat {
  pub seat_id: u16,
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
        "provided string does not match any Status variant",
      )),
    }
  }
}

// // API 路由處理函數
// #[get("/locations_status")]
// fn locations_status() -> Json<LocationsStatus> {
//   // 假設這是從某個資料源獲取的數據
//   let location_statuses = vec![
//     LocationStatus {
//       location_id: 1,
//       status: "Available".to_string(),
//     },
//     LocationStatus {
//       location_id: 2,
//       status: "Occupied".to_string(),
//     },
//     // 其他位置狀態...
//   ];

//   // 建構 LocationsStatus 結構並封裝為 JSON
//   Json(LocationsStatus {
//     locations: location_statuses,
//   })
// }