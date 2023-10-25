#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub user_id: String,   // gen
  pub user_name: String, // len < 20 uni
  pub password_hash: String,
  pub email: String,
}
