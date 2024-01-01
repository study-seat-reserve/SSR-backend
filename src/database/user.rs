use super::common::*;

// 註冊
pub async fn insert_new_user_info(
  pool: &Pool<Sqlite>,
  user_info: user::UserInfo,
) -> Result<(), Status> {
  log::info!("Inserting new user information");

  let sql = "
    INSERT INTO Users 
      (user_name, password_hash, email, user_role, verified, verification_token) 
    VALUES 
      (?1, ?2, ?3, ?4, ?5, ?6)";

  handle_sqlx(
    query(sql)
      .bind(user_info.user_name)
      .bind(user_info.password_hash)
      .bind(user_info.email)
      .bind(user_info.user_role)
      .bind(user_info.verified)
      .bind(user_info.verification_token)
      .execute(pool)
      .await,
    "Inserting new user information",
  )?;

  log::info!("Successfully inserted new user information");
  Ok(())
}

pub async fn get_user_info(pool: &Pool<Sqlite>, user_name: &str) -> Result<user::UserInfo, Status> {
  let sql = "
    SELECT 
      user_name, password_hash, email, user_role, verified, verification_token
    FROM 
      Users
    WHERE 
      user_name = ?";

  let user_info = handle_sqlx(
    query_as::<_, user::UserInfo>(sql)
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
  let sql = "
    UPDATE Users 
    SET 
      verified = true 
    WHERE 
      verification_token = ?";

  let result = handle_sqlx(
    query(sql).bind(verification_token).execute(pool).await,
    "Updating user verified by verification_token",
  )?;

  let affected_rows = result.rows_affected();

  if affected_rows == 0 {
    log::warn!("No Users found for updation");
    return Err(Status::NotFound);
  }

  Ok(())
}

pub async fn insert_user_to_blacklist(
  pool: &Pool<Sqlite>,
  user_name: &str,
  start_time: i64,
  end_time: i64,
) -> Result<(), Status> {
  let result = handle_sqlx(
    query!(
      "INSERT INTO BlackList
        (user_name, start_time, end_time)
      VALUES
        (
          ?,
          datetime(?, 'unixepoch', '+8 hours'),
          datetime(?, 'unixepoch', '+8 hours')
        )",
      user_name,
      start_time,
      end_time
    )
    .execute(pool)
    .await,
    "Inserting user to balck list",
  )?;

  let affected_rows = result.rows_affected();

  if affected_rows == 0 {
    log::warn!("No insert operation was executed");
    return Err(Status::NotFound);
  }

  Ok(())
}

pub async fn delete_user_from_blacklist(
  pool: &Pool<Sqlite>,
  user_name: &str,
) -> Result<(), Status> {
  let result = handle_sqlx(
    query!(
      "DELETE FROM BlackList
      WHERE
        user_name = ? ",
      user_name,
    )
    .execute(pool)
    .await,
    "Deleting user from balck list",
  )?;

  let affected_rows = result.rows_affected();

  if affected_rows == 0 {
    log::warn!("No delete operation was executed from BlackList");
    return Err(Status::NotFound);
  }

  Ok(())
}

pub async fn is_user_in_blacklist(pool: &Pool<Sqlite>, user_name: &str) -> Result<bool, Status> {
  let now = naive_datetime_to_timestamp(get_now()).unwrap();

  let result = handle_sqlx(
    query_scalar!(
      "SELECT EXISTS(
        SELECT 1 FROM BlackList
        WHERE
          user_name = ? AND
          start_time <= datetime(?, 'unixepoch', '+8 hours') AND
          end_time > datetime(?, 'unixepoch', '+8 hours')
      )",
      user_name,
      now,
      now
    )
    .fetch_one(pool)
    .await,
    "Checking if the user is currently listed in the blacklist",
  )?;

  let is_within_blacklist: bool = result.map_or(false, |count| count != 0);

  Ok(is_within_blacklist)
}

#[cfg(test)]
mod tests {

  use super::*;

  #[tokio::test]
  async fn test_insert_new_user_info() {
    let pool = Pool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE Users (id INTEGER PRIMARY KEY, name TEXT)")
      .execute(&pool)
      .await
      .unwrap();

    let user_info = user::UserInfo {
      user_name: "testuser".to_string(),
      password_hash: "testpassword".to_string(),
      email: "test@email.ntou.edu.tw".to_string(),
      user_role: user::UserRole::RegularUser,
      verified: false,
      verification_token: "randomtoken".to_string(),
    };

    insert_new_user_info(&pool, user_info).await.unwrap();

    let user_in_db = sqlx::query_as::<_, user::UserInfo>("SELECT * FROM Users WHERE name = ?")
      .bind("testuser")
      .fetch_one(&pool)
      .await
      .unwrap();

    assert_eq!(user_in_db.user_name, "testuser");
  }
}
