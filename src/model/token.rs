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
