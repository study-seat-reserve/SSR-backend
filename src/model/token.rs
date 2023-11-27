use super::user;
use crate::utils::*;
use chrono::{Duration, Utc};
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
  pub user: String,
  pub role: user::UserRole,
  pub exp: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let headers = request.headers().get_one("Authorization");
    match headers {
      Some(header) => {
        let token = header.replace("Bearer ", "");
        log::debug!("token: {:?}", token);

        match verify_jwt(&token) {
          Ok(claims) => Outcome::Success(claims),
          Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
      }
      None => Outcome::Failure((Status::BadRequest, ())),
    }
  }
}

// TEST

use super::*;
use crate::model::user::{UserInfo, UserRole};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;

const TEST_USERNAME: &str = "test_user";
const TEST_KEY: &str = "test_key";

#[test]
fn test_claims_serialization() {
  let claims = Claims {
    user: "test_user".to_string(),
    role: UserRole::RegularUser,
    exp: 1234567,
  };

  let serialized = serde_json::to_string(&claims).unwrap();
  let deserialized: Claims = serde_json::from_str(&serialized).unwrap();

  assert_eq!(claims, deserialized);
}

#[test]
fn test_token_encoding_decoding() {
  // Test token encoding and decoding

  let test_userinfo = UserInfo {
    user_name: TEST_USERNAME.to_string(),
    password: "test_password".to_string(),
    email: "test@example.com".to_string(),
    user_role: UserRole::RegularUser,
    verified: true,
  };

  let test_key = TEST_KEY;
  let encoding_key = EncodingKey::from_secret(test_key.as_bytes());

  let header = Header::default();

  let expiration = Utc::now()
    .checked_add_signed(Duration::hours(1))
    .expect("valid timestamp")
    .timestamp() as usize;

  let claims = Claims {
    user: test_userinfo.user_name.clone(),
    role: test_userinfo.user_role,
    exp: expiration,
  };

  let token = encode(&header, &claims, &encoding_key).unwrap();

  let decoding_key = DecodingKey::from_secret(test_key.as_bytes());
  let validation = Validation::new(Algorithm::HS256);

  let decoded_claims = decode::<Claims>(&token, &decoding_key, &validation);

  assert!(decoded_claims.is_ok());
  assert_eq!(decoded_claims.unwrap().claims, claims);
}
