use regex::Regex;
use serde::{Deserialize, Serialize};

use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct User {
  #[validate(length(min = 1, max = 20), custom = "validate_username")]
  pub user_name: String,
  #[validate(length(min = 8, max = 20))]
  pub password: String,
  #[validate(email, contains = "@mail.ntou.edu.tw")]
  pub email: String,
}

fn validate_username(user_name: &str) -> Result<(), ValidationError> {
  let regex = Regex::new(r"^[a-zA-Z0-9]*$").unwrap();

  if !regex.is_match(user_name) {
    return Err(ValidationError::new(
      "Username must only contain letters and numbers",
    ));
  }

  Ok(())
}
