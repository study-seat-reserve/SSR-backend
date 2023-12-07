use crate::{model::*, utils::*};
use sqlx::{query, query_as, Pool, Sqlite};

// 註冊
pub async fn insert_new_user_info(
  pool: &Pool<Sqlite>,
  user_name: &str,
  password_hash: &str,
  email: &str,
  verification_token: &str,
) -> Result<(), Status> {
  log::info!("Inserting new user information");

  let user_role = user::UserRole::RegularUser.to_string();

  handle_sqlx(query("INSERT INTO Users (user_name, password_hash, email, user_role, verified, verification_token) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(user_name)
        .bind(password_hash)
        .bind(email)
        .bind(user_role)
        .bind(false)
        .bind(verification_token)
        .execute(pool)
        .await,
  "Inserting new user information")?;

  log::info!("Successfully inserted new user information");
  Ok(())
}

pub async fn get_user_info(pool: &Pool<Sqlite>, user_name: &str) -> Result<user::UserInfo, Status> {
  let user_info = handle_sqlx(
    query_as::<_, user::UserInfo>(
      "SELECT user_name, password_hash, email, user_role, verified
         FROM Users
         WHERE user_name = ?",
    )
    .bind(user_name)
    .fetch_one(pool)
    .await,
    "Selecting user info",
  )?;

  Ok(user_info)
}

pub async fn update_user_verified_by_token(
  pool: &Pool<Sqlite>,
  verification_token: &str,
) -> Result<(), Status> {
  let result = handle_sqlx(
    query("UPDATE Users SET verified = true WHERE verification_token = ?")
      .bind(verification_token)
      .execute(pool)
      .await,
    "Updating user verified by verification_token",
  )?;

  let affected_rows = result.rows_affected();

  if affected_rows == 0 {
    log::warn!("No Users found for updation");
    return Err(Status::NotFound);
  }

  Ok(())
}

pub async fn check_if_user_name_exists(
  pool: &Pool<Sqlite>,
  user_name: &str,
) -> Result<bool, Status> {
  let result: Option<(i64,)> = handle_sqlx(
    query_as::<_, (i64,)>("SELECT COUNT(*) FROM Users WHERE user_name = ?")
      .bind(user_name)
      .fetch_optional(pool)
      .await,
    "Querying select operation",
  )?;

  match result {
    Some(_) => {
      log::debug!("username: {} exists", user_name);
      Ok(true)
    }
    None => {
      log::debug!("username: {} does not exist", user_name);
      Ok(false)
    }
  }
}
