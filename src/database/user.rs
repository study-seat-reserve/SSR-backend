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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::user::UserInfo;
  use sqlx::PgPool;

  // 设置测试环境
  async fn setup() -> PgPool {
    let database_url = "postgres://localhost:5432";
    PgPool::connect(&database_url).await.unwrap()
  }

  #[tokio::test]
  async fn test_insert_user_info() {
    let pool = setup().await;

    let test_data = UserInfo {
      user_name: "testuser".to_string(),
      password_hash: "testpassword".to_string(),
      email: "test@eamil.ntou.edu.tw".to_string(),
      user_role: user::UserRole::RegularUser,
      verified: false,
      verification_token: "token123".to_string(),
    };

    let result = insert_new_user_info(&pool, test_data).await;

    assert!(result.is_ok());

    let saved = get_user_info(&pool, "test").await.unwrap();
    assert_eq!(saved.user_name, "test");
    assert_eq!(saved.email, "test@eamil.ntou.edu.tw");
  }

  #[tokio::test]
  async fn test_get_user_info() {
    let pool = setup().await;

    let test_data = UserInfo {
      user_name: "testuser".to_string(),
      password_hash: "testpassword".to_string(),
      email: "test@eamil.ntou.edu.tw".to_string(),
      user_role: user::UserRole::RegularUser,
      verified: false,
      verification_token: "token123".to_string(),
    };
    insert_new_user_info(&pool, test_data).await.unwrap();

    let result = get_user_info(&pool, "test").await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.user_name, "test");
  }

  #[tokio::test]
  async fn test_update_user_verified() {
    let pool = setup().await;

    let test_data = UserInfo {
      user_name: "testuser".to_string(),
      password_hash: "testpassword".to_string(),
      email: "test@email.ntou.edu.tw".to_string(),
      user_role: user::UserRole::RegularUser,
      verified: false,
      verification_token: "token123".to_string(),
    };
    insert_new_user_info(&pool, test_data).await.unwrap();

    let result = update_user_verified_by_token(&pool, "token123").await;

    assert!(result.is_ok());

    let user = get_user_info(&pool, "test").await.unwrap();
    assert!(user.verified);
  }
}
