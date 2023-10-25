use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub user_id: Uuid,     // gen
  pub user_name: String, // len < 20 uni
  pub password_hash: String,
  pub blacklist: bool,
  pub email: String,
}
