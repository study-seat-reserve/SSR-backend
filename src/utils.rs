use crate::model::{constant::*, *};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lettre::{
  message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport,
};
pub use rocket::http::Status;
use rusqlite::{Error as SqliteError, ErrorCode};
use std::{env, io::ErrorKind};
use validator::ValidationErrorsKind;

pub fn handle<T, E>(result: Result<T, E>, prefix: &str) -> Result<T, Status>
where
  E: std::error::Error + 'static,
{
  result.map_err(|err| {
    log::error!("{} failed with error: {:?}", prefix, err);

    let dyn_error: &dyn std::error::Error = &err;

    if let Some(e) = dyn_error.downcast_ref::<std::io::Error>() {
      match e.kind() {
        ErrorKind::NotFound => Status::NotFound,
        ErrorKind::PermissionDenied => Status::Forbidden,
        ErrorKind::ConnectionRefused => Status::ServiceUnavailable,
        _ => Status::InternalServerError,
      }
    } else if let Some(e) = dyn_error.downcast_ref::<rusqlite::Error>() {
      match e {
        SqliteError::SqliteFailure(se, _) => match se.code {
          ErrorCode::ConstraintViolation | ErrorCode::TypeMismatch => Status::UnprocessableEntity,
          ErrorCode::PermissionDenied => Status::Forbidden,
          ErrorCode::NotFound => Status::NotFound,
          ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => Status::ServiceUnavailable,
          _ => Status::InternalServerError,
        },
        _ => Status::InternalServerError,
      }
    } else {
      Status::InternalServerError
    }
  })
}

pub fn handle_validator(result: Result<(), validator::ValidationErrors>) -> Result<(), Status> {
  result.map_err(|validation_errors| {
    let error_message: Vec<String> = validation_errors
      .errors()
      .into_iter()
      .map(|(field, validation_error_kind)| {
        let msg: Vec<String> =
          if let ValidationErrorsKind::Field(validation_error) = validation_error_kind {
            validation_error
              .into_iter()
              .map(|error| format!("Invalid {} Error: {}", field, error.code))
              .collect()
          } else {
            Vec::new()
          };
        return msg.join(", ");
      })
      .collect();

    log::error!("{}", error_message.join(", "));

    Status::UnprocessableEntity
  })
}

pub fn get_today() -> NaiveDate {
  Local::now().date_naive()
}

pub fn get_now() -> u32 {
  parse_time(Local::now().naive_local().time()).expect("Parse time 'now' error!")
}

pub fn get_datetime() -> NaiveDateTime {
  Local::now().naive_local()
}

pub fn get_tomorrow_midnight() -> NaiveDateTime {
  let now = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
  let get_tomorrow_midnight = now + Duration::days(1);
  get_tomorrow_midnight
}

pub fn get_last_week() -> NaiveDate {
  let now = Local::now().date_naive();
  let seven_days_ago = now - Duration::days(7);
  seven_days_ago
}

pub fn date_from_string(date: &str) -> Result<NaiveDate, Status> {
  handle(
    NaiveDate::parse_from_str(date, "%Y-%m-%d"),
    &format!("Parsing date from str '{}'", date),
  )
}

pub fn parse_time(time: NaiveTime) -> Result<u32, Status> {
  let time_string = format!("{:02}{:02}{:02}", time.hour(), time.minute(), time.second());
  log::debug!("Parsing time {:?} to string: {}", time, time_string);

  handle(time_string.parse::<u32>(), "Parsing time")
}

pub fn validate_seat_id(seat_id: u16) -> Result<(), Status> {
  if seat_id < 1 || seat_id > NUMBER_OF_SEATS {
    log::error!("Invalid seat_id Error: Seat id out of range");
    return Err(Status::UnprocessableEntity);
  }

  Ok(())
}

pub fn validate_date(date: NaiveDate) -> Result<(), Status> {
  let today = get_today();
  let three_days_later = today + Duration::days(3);

  if date < today || date > three_days_later {
    log::error!("Invalid date Error: Invalid reservation date");
    return Err(Status::UnprocessableEntity);
  }

  Ok(())
}

pub fn validate_datetime(date: NaiveDate, start_time: u32, end_time: u32) -> Result<(), Status> {
  let today = get_today();
  let now = get_now();

  validate_date(date)?;

  if date == today && start_time < now {
    log::error!("Invalid start_time: start_time < current time");
    return Err(Status::UnprocessableEntity);
  }

  if end_time < start_time {
    log::error!("Invalid start_time: start time > end time");
    return Err(Status::UnprocessableEntity);
  }

  Ok(())
}

pub fn get_root() -> String {
  env::var("ROOT").expect("Failed to get root path")
}

pub fn get_base_url() -> String {
  env::var("BASE_URL").expect("Failed to get base url")
}

pub fn send_verification_email(email: &str, url: &str) -> Result<(), Status> {
  let email_address_str = env::var("EMAIL_ADDRESS").expect("Failed to get email address");
  let email_password = env::var("EMAIL_PASSWORD").expect("Failed to get email password");
  let email_domain = env::var("EMAIL_DOMAIN").expect("Failed to get email domain");

  let email_address = handle(
    email_address_str.parse::<Mailbox>(),
    "Parsing email address",
  )?;
  let user_email = handle(email.parse::<Mailbox>(), "Parsing user email")?;

  let email = handle(
    Message::builder()
      .to(user_email)
      .from(email_address)
      .subject("Verify your email")
      .body(format!(
        "Please click on the link to verify your email: {}",
        url
      )),
    "Building email",
  )?;

  let creds = Credentials::new(email_address_str, email_password);

  let mailer = handle(
    SmtpTransport::relay(&format!("smtp.{}.com", email_domain)),
    "Setting SMTP server address",
  )?
  .credentials(creds)
  .build();

  handle(mailer.send(&email), "Sending email")?;

  Ok(())
}

pub fn create_token(userinfo: user::UserInfo) -> Result<String, Status> {
  let expiration = Utc::now()
    .checked_add_signed(Duration::hours(1)) // 1 小時後過期
    .expect("valid timestamp")
    .timestamp() as usize;

  let claims = token::Claims {
    user: userinfo.user_name,
    role: userinfo.user_role,
    exp: expiration,
  };

  let header = Header::new(Algorithm::HS256);
  let key = env::var("SECRET_KEY").expect("Failed to get secret key");

  let encoding_key = EncodingKey::from_secret(key.as_ref());

  let token = handle(encode(&header, &claims, &encoding_key), "Encoding JWT")?;

  Ok(token)
}

pub fn verify_jwt(token: &str) -> Result<token::Claims, Status> {
  let key = env::var("SECRET_KEY").expect("Failed to get secret key");

  let token = handle(
    decode::<token::Claims>(
      token,
      &DecodingKey::from_secret(key.as_ref()),
      &Validation::new(Algorithm::HS256),
    ),
    "Decoding JWT",
  )?;

  Ok(token.claims)
}

// TEST

#[cfg(test)]
mod tests {

  use super::*;
  use chrono::{Duration, NaiveDate, NaiveTime};
  use rocket::http::Status;
  use validator::{ValidationError, ValidationErrors};

  #[test]
  fn test_get_today() {
    assert_eq!(get_today(), Local::now().date_naive());
  }

  #[test]
  fn test_get_now() {
    assert_eq!(
      get_now(),
      parse_time(Local::now().naive_local().time()).unwrap()
    );
  }

  #[test]
  fn test_get_datetime() {
    assert_eq!(get_datetime(), Local::now().naive_local());
  }

  #[test]
  fn test_get_tomorrow_midnight() {
    let tomorrow = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap() + Duration::days(1);
    assert_eq!(get_tomorrow_midnight(), tomorrow);
  }

  #[test]
  fn test_get_last_week() {
    let last_week = Local::now().date_naive() - Duration::days(7);
    assert_eq!(get_last_week(), last_week);
  }

  #[test]
  fn test_date_from_string() {
    let date_str = "2023-12-27";
    let expected = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
    assert_eq!(date_from_string(date_str).unwrap(), expected);
  }

  #[test]
  fn test_parse_time() {
    let time = NaiveTime::from_hms_opt(10, 08, 29).expect("Invalid time values");
    assert_eq!(parse_time(time).unwrap(), 100829);
  }

  #[test]
  fn test_validate_seat_id() {
    let seat_id = 109;
    assert!(validate_seat_id(seat_id).is_ok());

    assert_eq!(
      validate_seat_id(0).unwrap_err(),
      Status::UnprocessableEntity
    );

    assert_eq!(
      validate_seat_id(218).unwrap_err(),
      Status::UnprocessableEntity
    );
  }

  #[test]
  fn test_validate_date() {
    let today = Local::now().date_naive();
    let valid_date = today + Duration::days(1);
    assert!(validate_date(valid_date).is_ok());

    let invalid_date = today - Duration::days(1);
    assert_eq!(
      validate_date(invalid_date).unwrap_err(),
      Status::UnprocessableEntity
    );

    let invalid_date = today + Duration::days(4);
    assert_eq!(
      validate_date(invalid_date).unwrap_err(),
      Status::UnprocessableEntity
    );
  }

  #[test]
  fn test_validate_datetime() {
    let date = get_today() + Duration::days(1);
    let start_time = get_now() + 3600;
    let end_time = start_time + 3600;
    assert!(validate_datetime(date, start_time, end_time).is_ok());

    let date = get_today();
    let start_time = get_now() - 3600;
    let end_time = start_time + 3600;
    assert!(validate_datetime(date, start_time, end_time).is_err());

    let date = get_today() + Duration::days(1);
    let start_time = get_now();
    let end_time = start_time - 3600;
    assert!(validate_datetime(date, start_time, end_time).is_err());
  }

  #[test]
  fn test_handle_ok() {
    let result: Result<&str, rocket::http::Status> =
      handle::<_, std::io::Error>(Ok("Success"), "Test");
    assert_eq!(result, Ok("Success"));
  }

  #[test]
  fn test_handle_not_found_error() {
    let result: Result<(), rocket::http::Status> =
      handle(Err(std::io::Error::from(ErrorKind::NotFound)), "Test");
    assert_eq!(result, Err(Status::NotFound));
  }

  #[test]
  fn test_handle_permission_denied_error() {
    let result: Result<(), rocket::http::Status> = handle(
      Err(std::io::Error::from(ErrorKind::PermissionDenied)),
      "Test",
    );
    assert_eq!(result, Err(Status::Forbidden));
  }

  #[test]
  fn test_handle_connection_refused_error() {
    let result: Result<(), rocket::http::Status> = handle(
      Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Connection refused",
      )),
      "Test",
    );
    assert_eq!(result, Err(Status::ServiceUnavailable));
  }

  #[test]
  fn test_handle_other_io_error() {
    let result: Result<(), rocket::http::Status> = handle::<_, std::io::Error>(
      Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Other error",
      )),
      "Test",
    );
    assert_eq!(result, Err(Status::InternalServerError));
  }

  #[test]
  fn test_handle_rusqlite_constraint_violation() {
    let result: Result<u32, rusqlite::Error> = Err(rusqlite::Error::SqliteFailure(
      rusqlite::ffi::Error {
        code: rusqlite::ffi::ErrorCode::ConstraintViolation,
        extended_code: 0,
      },
      None,
    ));
    let prefix = "Test";
    assert_eq!(handle(result, prefix), Err(Status::UnprocessableEntity));
  }

  #[test]
  fn test_handle_rusqlite_permission_denied() {
    let result: Result<(), rocket::http::Status> = handle(
      Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error {
          code: rusqlite::ffi::ErrorCode::PermissionDenied,
          extended_code: 0,
        },
        None,
      )),
      "Test",
    );
    assert_eq!(result, Err(Status::Forbidden));
  }

  fn create_validation_errors() -> ValidationErrors {
    let mut validation_errors = ValidationErrors::new();

    validation_errors.add("field1", ValidationError::new("error1"));

    validation_errors.add("field2", ValidationError::new("error2"));

    validation_errors
  }

  // #[test]
  // fn test_handle_validator() {
  //   let result = Ok(());
  //   assert_eq!(handle_validator(result), Ok(()));

  //   let validation_errors = create_validation_errors();
  //   let result = Err(validation_errors);
  //   let status_result = handle_validator(result);
  //   assert!(status_result.is_err());

  //   match status_result {
  //     Err(Status::UnprocessableEntity) => {
  //       assert!(status_result
  //         .unwrap_err()
  //         .to_string()
  //         .contains("Invalid field1 Error: error1"));
  //       assert!(status_result
  //         .unwrap_err()
  //         .to_string()
  //         .contains("Invalid field2 Error: error2"));
  //     }
  //     _ => panic!("Unexpected result variant"),
  //   }
  // }
}
