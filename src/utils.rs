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
  use chrono::{Duration, NaiveDate, NaiveTime, Utc};

  #[test]
  fn test_get_now() {
    let now = get_now();
    assert!(now > 0);
  }

  #[test]
  fn test_get_today() {
    let today = Local::now().date_naive();
    assert_eq!(get_today(), today);
  }

  #[test]
  fn test_get_datetime() {
    let datetime = get_datetime();
    assert!(datetime.timestamp() > 0);
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
    let date_str = "2023-02-15";
    let expected = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();

    assert_eq!(date_from_string(date_str).unwrap(), expected);
  }

  #[test]
  fn test_parse_time() {
    let time = NaiveTime::from_hms_opt(12, 34, 56).expect("Invalid time values");
    assert_eq!(parse_time(time).unwrap(), 123456);
  }

  #[test]
  fn test_validate_seat_id() {
    validate_seat_id(100).unwrap();

    assert_eq!(
      validate_seat_id(0).unwrap_err(),
      Status::UnprocessableEntity
    );
  }

  #[test]
  fn test_validate_date() {
    let today = Local::now().date_naive();
    let valid_date = today + Duration::days(1);

    validate_date(valid_date).unwrap();

    let invalid_date = today - Duration::days(1);

    assert_eq!(
      validate_date(invalid_date).unwrap_err(),
      Status::UnprocessableEntity
    );
  }

  #[test]
  fn test_validate_datetime() {
    let tomorrow = Utc::now().date_naive() + Duration::days(1);

    validate_datetime(tomorrow, 800, 1000).unwrap();

    let now = Utc::now().date_naive();
    assert_eq!(
      validate_datetime(now, 700, 800).unwrap_err(),
      Status::UnprocessableEntity
    );

    let invalid_order = validate_datetime(tomorrow, 800, 700);
    assert_eq!(invalid_order.unwrap_err(), Status::UnprocessableEntity);
  }

  #[test]
  fn test_send_verification_email() {
    std::env::set_var("EMAIL_ADDRESS", "test@example.com");
    std::env::set_var("EMAIL_PASSWORD", "test_password");
    std::env::set_var("EMAIL_DOMAIN", "example.com");

    let test_email = "test@example.com";
    let test_url = "https://test.com/test";

    let _result = send_verification_email(test_email, test_url);

    assert!(result.is_ok());
  }
}
