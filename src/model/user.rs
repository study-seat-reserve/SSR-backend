use regex::Regex;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, str::FromStr};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct User {
  #[validate(length(min = 1, max = 20), custom = "validate_username")]
  pub user_name: String,
  #[validate(length(min = 8, max = 20))]
  pub password: String,
  // #[validate(email, contains = "@mail.ntou.edu.tw")]
  #[validate(email)]
  pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
  pub user_name: String,
  pub password: String,
  pub email: String,
  pub user_role: UserRole,
  pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginCreds {
  #[validate(length(min = 1, max = 20), custom = "validate_username")]
  pub user_name: String,
  #[validate(length(min = 8, max = 20))]
  pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
  RegularUser,
  Admin,
}

impl FromSql for UserRole {
  fn column_result(value: ValueRef) -> Result<UserRole, FromSqlError> {
    match value.as_str() {
      Ok("RegularUser") => Ok(UserRole::RegularUser),
      Ok("Admin") => Ok(UserRole::Admin),
      _ => Err(FromSqlError::InvalidType),
    }
  }
}

impl ToString for UserRole {
  fn to_string(&self) -> String {
    match *self {
      UserRole::RegularUser => "RegularUser".to_owned(),
      UserRole::Admin => "Admin".to_owned(),
    }
  }
}

impl FromStr for UserRole {
  type Err = std::io::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "RegularUser" => Ok(UserRole::RegularUser),
      "Admin" => Ok(UserRole::Admin),
      _ => Err(std::io::Error::new(
        ErrorKind::InvalidInput,
        "Provided string does not match any UserRole variant",
      )),
    }
  }
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
