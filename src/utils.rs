use crate::model::*;
use chrono::{Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use lettre::{
  message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport,
};
use sqlx::Error as SqlxError;
use std::{
  env,
  io::{Error as IoError, ErrorKind},
};
use validator::ValidationErrorsKind;

pub use rocket::http::Status;

pub fn handle<T, E>(result: Result<T, E>, prefix: &str) -> Result<T, Status>
where
  E: std::error::Error + 'static,
{
  result.map_err(|err| {
    log::error!("{} failed with error: {:?}", prefix, err);

    let dyn_error: &dyn std::error::Error = &err;

    if let Some(e) = dyn_error.downcast_ref::<IoError>() {
      match e.kind() {
        ErrorKind::NotFound => Status::NotFound,
        ErrorKind::PermissionDenied => Status::Forbidden,
        ErrorKind::ConnectionRefused => Status::ServiceUnavailable,
        _ => Status::InternalServerError,
      }
    } else {
      Status::InternalServerError
    }
  })
}

pub fn handle_sqlx<T>(result: Result<T, SqlxError>, prefix: &str) -> Result<T, Status> {
  result.map_err(|err| {
    log::error!("{} failed with error: {:?}", prefix, err);

    match &err {
      SqlxError::RowNotFound => Status::NotFound,
      SqlxError::ColumnNotFound(_) => Status::BadRequest,
      SqlxError::ColumnIndexOutOfBounds { .. } => Status::BadRequest,
      _ => Status::InternalServerError,
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

pub fn get_now() -> NaiveDateTime {
  Local::now().naive_local()
}

pub fn time_to_string(timestamp: i64) -> Result<String, Status> {
  let naive_datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).ok_or_else(|| {
    log::error!("Invalid timestamp");
    Status::InternalServerError
  })?;

  Ok(naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub fn naive_date_to_timestamp(
  date: NaiveDate,
  hour: u32,
  min: u32,
  sec: u32,
) -> Result<i64, Status> {
  let time = NaiveTime::from_hms_opt(hour, min, sec).ok_or_else(|| {
    log::error!("Invalid NaiveTime");
    Status::InternalServerError
  })?;

  // 本地日期時間 GMT++8
  let datetime_local = NaiveDateTime::new(date, time);
  // GNT 0 日期時間
  let datetime = datetime_local - chrono::Duration::hours(8);

  let timestamp = datetime.timestamp();

  Ok(timestamp)
}

/// -> get now timestamp
pub fn naive_datetime_to_timestamp(datetime: NaiveDateTime) -> Result<i64, Status> {
  let datetime = datetime - chrono::Duration::hours(8);
  let timestamp = datetime.timestamp();

  Ok(timestamp)
}

pub fn timestamp_to_naive_datetime(timestamp: i64) -> Result<NaiveDateTime, Status> {
  // GMT+8
  let offset = FixedOffset::east_opt(8 * 3600).ok_or_else(|| {
    log::error!("Invalid offset");
    Status::InternalServerError
  })?;

  // GMT 0 日期時間
  let datetime = Utc.timestamp_opt(timestamp, 0).single().ok_or_else(|| {
    log::error!("Invalid timestamp");
    Status::InternalServerError
  })?;

  // 本地日期時間
  let datetime_local = datetime.with_timezone(&offset).naive_local();

  Ok(datetime_local)
}

pub fn validate_seat_id(seat_id: u16) -> Result<(), Status> {
  validate_utils::validate_seat_id(seat_id).map_err(|e| {
    let message = e.code.as_ref();
    log::error!("seat_id: {}, Failed with error: {}", seat_id, message);
    Status::UnprocessableEntity
  })?;

  Ok(())
}

pub fn validate_datetime(start_time: i64, end_time: i64) -> Result<(), Status> {
  validate_utils::validate_datetime(start_time, end_time).map_err(|e| {
    let message = e.code.as_ref();
    log::error!(
      "start_time: {}, end_time: {}, Failed with error: {}",
      start_time,
      end_time,
      message
    );
    Status::UnprocessableEntity
  })?;

  Ok(())
}

pub fn get_root() -> String {
  env::var("ROOT").expect("Failed to get root path")
}

pub fn get_base_url() -> String {
  env::var("BASE_URL").expect("Failed to get base url")
}

pub fn send_verification_email(user_email: &str, verification_token: &str) -> Result<(), Status> {
  let email_address_str = env::var("EMAIL_ADDRESS").expect("Failed to get email address");
  let email_password = env::var("EMAIL_PASSWORD").expect("Failed to get email password");
  let email_domain = env::var("EMAIL_DOMAIN").expect("Failed to get email domain");

  let email_address = handle(
    email_address_str.parse::<Mailbox>(),
    "Parsing email address",
  )?;

  let user_email = handle(user_email.parse::<Mailbox>(), "Parsing user email")?;
  let url = format!(
    "{}/api/verify?verification_token={}",
    get_base_url(),
    verification_token
  );

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

pub fn create_userinfo_token(user_name: &str, user_role: user::UserRole) -> Result<String, Status> {
  let duration: Duration = match user_role {
    user::UserRole::Admin => Duration::hours(24), // 1 天後過期
    user::UserRole::RegularUser => Duration::hours(1), // 1 小時後過期
  };

  let exp = Utc::now()
    .checked_add_signed(duration)
    .expect("valid timestamp")
    .timestamp() as usize;

  let claim = token::UserInfoClaim {
    user: user_name.to_string(),
    role: user_role,
    exp: exp,
  };

  let header = Header::new(Algorithm::HS256);
  let key = env::var("SECRET_KEY").expect("Failed to get secret key");

  let encoding_key = EncodingKey::from_secret(key.as_ref());

  let token = handle(encode(&header, &claim, &encoding_key), "Encoding JWT")?;

  Ok(token)
}

pub fn create_resend_verification_token(
  email: &str,
  verification_token: &str,
  is_resend: bool,
) -> Result<String, Status> {
  let exp = Utc::now()
    .checked_add_signed(Duration::hours(1)) // 1 小時後過期
    .expect("valid timestamp")
    .timestamp() as usize; // or u64

  let mut expiration = 0;
  if is_resend {
    expiration = Utc::now()
      .checked_add_signed(Duration::minutes(1))
      .expect("valid timestamp")
      .timestamp() as i64;
  }

  let claim = token::ResendVerificationClaim {
    email: email.to_string(),
    verification_token: verification_token.to_string(),
    expiration: expiration,
    exp: exp,
  };

  let header = Header::new(Algorithm::HS256);
  let key = env::var("SECRET_KEY").expect("Failed to get secret key");

  let encoding_key = EncodingKey::from_secret(key.as_ref());

  let token = handle(encode(&header, &claim, &encoding_key), "Encoding JWT")?;

  Ok(token)
}

#[cfg(test)]
mod tests {

  use crate::model::token::{Claim, ResendVerificationClaim, UserInfoClaim};

  use super::*;

  // test_handle

  #[test]
  fn test_handle_not_found_error() {
    let result = Err::<i32, IoError>(IoError::new(ErrorKind::NotFound, "Not found"));
    let prefix = "Test";
    let handled_result = handle(result, prefix);

    assert_eq!(handled_result.unwrap_err(), Status::NotFound);

    // permission denied error
    let result = Err::<i32, IoError>(IoError::new(
      ErrorKind::PermissionDenied,
      "Permission denied",
    ));
    let prefix = "Test";
    let handled_result = handle(result, prefix);

    assert_eq!(handled_result.unwrap_err(), Status::Forbidden);

    //connection refused error
    let result = Err::<i32, IoError>(IoError::new(
      ErrorKind::ConnectionRefused,
      "Connection refused",
    ));
    let prefix = "Test";
    let handled_result = handle(result, prefix);

    assert_eq!(handled_result.unwrap_err(), Status::ServiceUnavailable);
  }

  #[test]
  fn test_handle_other_error() {
    let result = Err::<i32, IoError>(IoError::new(ErrorKind::Other, "Other"));
    let prefix = "Test";
    let handled_result = handle(result, prefix);

    assert_eq!(handled_result.unwrap_err(), Status::InternalServerError);
  }

  #[test]
  fn test_get_root() {
    env::set_var("ROOT", "/app");
    assert_eq!(get_root(), "/app");
  }

  #[test]
  #[should_panic(expected = "Failed to get root path")]
  fn test_get_root_miss() {
    env::remove_var("ROOT");
    let _ = get_root();
  }

  #[test]
  fn test_get_base_url() {
    env::set_var("BASE_URL", "https://api.example.com");
    assert_eq!(get_base_url(), "https://api.example.com");
  }

  #[test]
  #[should_panic(expected = "Failed to get base url")]
  fn test_get_base_url_miss() {
    env::remove_var("BASE_URL");
    let _ = get_base_url();
  }

  #[test]
  fn test_send_verification_email() {
    let user_email = "test@example.com";
    let verification_token = "token123";

    env::set_var("EMAIL_ADDRESS", "mybot@example.com");
    env::set_var("EMAIL_PASSWORD", "123456");
    env::set_var("EMAIL_DOMAIN", "example.com");

    let result = send_verification_email(user_email, verification_token);
    assert!(result.is_ok());

    env::remove_var("EMAIL_ADDRESS");
    let result = send_verification_email(user_email, verification_token);
    assert!(result.is_err());
  }

  #[test]
  fn test_create_userinfo_token() {
    let user_name = "testuser";
    let user_role = user::UserRole::RegularUser;

    env::set_var("SECRET_KEY", "mysecretkey");

    let token = create_userinfo_token(&user_name, user_role).unwrap();
    assert!(!token.is_empty());

    let claims = UserInfoClaim::verify_jwt(&token).unwrap();
    assert_eq!(claims.user, user_name);
    assert_eq!(claims.role, user_role);

    env::remove_var("SECRET_KEY");
    let result = create_userinfo_token(&user_name, user_role);
    assert!(result.is_err());
  }

  #[test]
  fn test_create_resend_verification_token() {
    let email = "test@example.com";
    let verification_token = "token123";

    env::set_var("SECRET_KEY", "mysecretkey");

    let token = create_resend_verification_token(&email, &verification_token, true).unwrap();

    let claims = ResendVerificationClaim::verify_jwt(&token).unwrap();
    assert_eq!(claims.email, email);
    assert_eq!(claims.verification_token, verification_token);

    env::remove_var("SECRET_KEY");
    let result = create_resend_verification_token(&email, &verification_token, true);
    assert!(result.is_err());
  }
}
