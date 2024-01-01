use super::{common::*, user};
use crate::utils::handle;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rocket::{
  http::Status,
  request::{FromRequest, Outcome, Request},
};
use std::env;

pub trait Claim: Sized {
  fn verify_jwt(token: &str) -> Result<Self, Status>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoClaim {
  pub user: String,
  pub role: user::UserRole,
  pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResendVerificationClaim {
  pub email: String,
  pub verification_token: String,
  pub expiration: i64,
  pub exp: usize,
}

impl Claim for UserInfoClaim {
  fn verify_jwt(token: &str) -> Result<UserInfoClaim, Status> {
    let key = env::var("SECRET_KEY").expect("Failed to get secret key");

    let token = handle(
      decode::<UserInfoClaim>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::new(Algorithm::HS256),
      ),
      "Decoding JWT",
    )?;

    Ok(token.claims)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserInfoClaim {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let headers = request.headers().get_one("Authorization");
    match headers {
      Some(header) => {
        let token = header.replace("Bearer ", "");

        match Self::verify_jwt(&token) {
          Ok(claims) => Outcome::Success(claims),
          Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
      }
      None => Outcome::Failure((Status::BadRequest, ())),
    }
  }
}

impl Claim for ResendVerificationClaim {
  fn verify_jwt(token: &str) -> Result<ResendVerificationClaim, Status> {
    let key = env::var("SECRET_KEY").expect("Failed to get secret key");

    let token = handle(
      decode::<ResendVerificationClaim>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::new(Algorithm::HS256),
      ),
      "Decoding JWT",
    )?;

    Ok(token.claims)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ResendVerificationClaim {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let headers = request.headers().get_one("Authorization");
    match headers {
      Some(header) => {
        let token = header.replace("Bearer ", "");

        match Self::verify_jwt(&token) {
          Ok(claims) => Outcome::Success(claims),
          Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
      }
      None => Outcome::Failure((Status::BadRequest, ())),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::model::user::UserRole;

  use super::*;

  #[test]
  fn test_verify_jwt_user_info_claim() {
    env::set_var("SECRET_KEY", "your_secret_key");

    let user_info_claim = UserInfoClaim {
      user: "testuser".to_string(),
      role: UserRole::RegularUser,
      exp: 1234567890,
    };

    let token = jsonwebtoken::encode(
      &jsonwebtoken::Header::default(),
      &user_info_claim,
      &jsonwebtoken::EncodingKey::from_secret("your_secret_key".as_ref()),
    )
    .expect("Failed to encode JWT");

    let result = UserInfoClaim::verify_jwt(&token);
    assert!(result.is_ok());

    let invalid_token = "invalid_token";
    let result = UserInfoClaim::verify_jwt(invalid_token);
    assert!(result.is_err());
  }

  #[test]
  fn test_verify_jwt_resend_verification_claim() {
    env::set_var("SECRET_KEY", "your_secret_key");

    let resend_verification_claim = ResendVerificationClaim {
      email: "test@email.ntou.edu.tw".to_string(),
      verification_token: "verificationtoken".to_string(),
      expiration: 1234567890,
      exp: 1234567890,
    };

    let token = jsonwebtoken::encode(
      &jsonwebtoken::Header::default(),
      &resend_verification_claim,
      &jsonwebtoken::EncodingKey::from_secret("your_secret_key".as_ref()),
    )
    .expect("Failed to encode JWT");

    let result = ResendVerificationClaim::verify_jwt(&token);
    assert!(result.is_ok());

    let invalid_token = "invalid_token";
    let result = ResendVerificationClaim::verify_jwt(invalid_token);
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_from_request_user_info_claim() {
    env::set_var("SECRET_KEY", "your_secret_key");

    let user_info_claim = UserInfoClaim {
      user: "testuser".to_string(),
      role: UserRole::RegularUser,
      exp: 1234567890,
    };

    let token = jsonwebtoken::encode(
      &jsonwebtoken::Header::default(),
      &user_info_claim,
      &jsonwebtoken::EncodingKey::from_secret("your_secret_key".as_ref()),
    )
    .expect("Failed to encode JWT");

    env::set_var("SECRET_KEY", "your_secret_key");

    let resend_verification_claim = ResendVerificationClaim {
      email: "test@email.ntou.edu.tw".to_string(),
      verification_token: "verificationtoken".to_string(),
      expiration: 1234567890,
      exp: 1234567890,
    };

    let token = jsonwebtoken::encode(
      &jsonwebtoken::Header::default(),
      &resend_verification_claim,
      &jsonwebtoken::EncodingKey::from_secret("your_secret_key".as_ref()),
    )
    .expect("Failed to encode JWT");
  }
}
