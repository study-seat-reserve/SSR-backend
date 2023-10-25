use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Seat {
  pub seat_id: u16,
  pub other_info: Option<String>,
}
