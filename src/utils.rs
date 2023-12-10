use crate::model::{constant::*, *};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
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
  let exp = Utc::now()
    .checked_add_signed(Duration::hours(1)) // 1 小時後過期
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
      .timestamp() as u64;
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
