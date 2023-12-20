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
#[cfg(test)]
mod test {
  use super::*;
  use crate::model::user::{self};
  use chrono::{Duration, Utc};
  use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

  fn test_claims() {
    let claim = Claims {
      user: "istestuser".to_string(),
      role: user::UserRole::RegularUser,
      exp: 1234567,
    };

    let serialized = serde_json::to_string(&claim).unwrap();
    let deserialized: Claims = serde_json::from_str(&serialized).unwrap();

    assert_eq!(claim, deserialized);
  }

  #[test]
  fn test_token() {
    let key = "istestkey";
    let encoding_key = EncodingKey::from_secret(key.as_bytes());
    let decoding_key = DecodingKey::from_secret(key.as_bytes());

    let header = Header::default();

    let expiration = Utc::now()
      .checked_add_signed(Duration::hours(1))
      .expect("valid timestamp")
      .timestamp() as usize;

    let claims = Claims {
      user: "istestuser".to_string(),
      role: user::UserRole::RegularUser,
      exp: expiration,
    };

    let token = encode(&header, &claims, &encoding_key).unwrap();

    let validation = Validation::new(Algorithm::HS256);
    let decoded_claims = decode::<Claims>(&token, &decoding_key, &validation);
    assert!(decoded_claims.is_ok());

    let decoded_claims = decoded_claims.unwrap();
    assert_eq!(decoded_claims.claims, claims);
  }

  /* UNDO */
  // test_from_request
}
