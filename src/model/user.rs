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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

//TEST
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_user() {
    // valid user
    let user = User {
      user_name: "istestuser".to_string(),
      password: "istestpassword".to_string(),
      email: "istestemail@mail.ntou.edu.tw".to_string(),
    };
    assert!(
      user.validate().is_ok(),
      "Error: {:?}",
      user.validate().err()
    );
    // invalid user
    let user = User {
      user_name: "".to_string(),
      password: "testpw".to_string(),
      email: "istestemail".to_string(),
    };
    assert!(user.validate().is_err(), "Error");
    if let Err(errors) = user.validate() {
      assert_eq!(errors.field_errors().len(), 3); // error for username field
    }

    let user = User {
      user_name: "istestuseristestuseristestuser".to_string(),
      password: "istestpasswordistestpasswordistestpassword".to_string(),
      email: "istestemail".to_string(),
    };
    assert!(user.validate().is_err(), "Error");
    if let Err(errors) = user.validate() {
      assert_eq!(errors.field_errors().len(), 3); // error for username field
    }
  }

  #[test]
  fn test_user_info() {
    let user = UserInfo {
      user_name: "istestuser".to_string(),
      password: "istestpassword".to_string(),
      email: "istestemail@mail.ntou.edu.tw".to_string(),
      user_role: UserRole::RegularUser,
      verified: true,
    };

    assert_eq!(user.user_name, "istestuser");
    assert_eq!(user.password, "istestpassword");
    assert_eq!(user.email, "istestemail@mail.ntou.edu.tw");
    assert_eq!(user.user_role, UserRole::RegularUser);
    assert_eq!(user.verified, true);

    let user = UserInfo {
      user_name: "istestuser".to_string(),
      password: "istestpassword".to_string(),
      email: "istestemail@mail.ntou.edu.tw".to_string(),
      user_role: UserRole::RegularUser,
      verified: false,
    };

    assert_eq!(user.user_name, "istestuser");
    assert_eq!(user.password, "istestpassword");
    assert_eq!(user.email, "istestemail@mail.ntou.edu.tw");
    assert_eq!(user.user_role, UserRole::RegularUser);
    assert_eq!(user.verified, false);
  }

  #[test]
  fn test_login_creds() {
    // valid
    let login_creds = LoginCreds {
      user_name: "istestuser".to_string(),
      password: "istestpassword".to_string(),
    };
    assert!(login_creds.validate().is_ok());

    // invalid
    let login_creds = LoginCreds {
      user_name: "".to_string(),      // Invalid username
      password: "testpw".to_string(), // Invalid password
    };

    assert!(login_creds.validate().is_err());
    if let Err(errors) = login_creds.validate() {
      assert_eq!(errors.field_errors().len(), 2); // Expecting errors for both fields
    }
  }

  #[test]
  fn test_column_result() {
    let result = UserRole::column_result(ValueRef::Text("RegularUser".as_bytes()));
    assert_eq!(result, Ok(UserRole::RegularUser));

    let result = UserRole::column_result(ValueRef::Text("Admin".as_bytes()));
    assert_eq!(result, Ok(UserRole::Admin));

    let result = UserRole::column_result(ValueRef::Text("Invalid".as_bytes()));
    assert!(result.is_err());
  }

  #[test]
  fn test_to_string() {
    assert_eq!(UserRole::RegularUser.to_string(), "RegularUser".to_owned());

    assert_eq!(UserRole::Admin.to_string(), "Admin".to_owned());
  }

  #[test]
  fn test_from_str() {
    assert_eq!(
      UserRole::from_str("RegularUser").unwrap(),
      UserRole::RegularUser
    );

    assert_eq!(UserRole::from_str("Admin").unwrap(), UserRole::Admin);

    assert!(UserRole::from_str("Invalid").is_err());
  }

  #[test]
  fn test_validate_username() {
    // Valid username
    let result = validate_username("istestuser000");
    assert!(result.is_ok());

    // Invalid username
    let result = validate_username("istestuser 000");
    assert!(result.is_err());
    if let Err(validation_error) = result {
      assert_eq!(
        validation_error.to_string(),
        "Username must only contain letters and numbers"
      );
    }

    let result = validate_username("istestuser@000");
    assert!(result.is_err());
    if let Err(validation_error) = result {
      assert_eq!(
        validation_error.to_string(),
        "Username must only contain letters and numbers"
      );
    }
  }
}
