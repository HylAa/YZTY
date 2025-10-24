use chrono::NaiveDateTime;
use sqlx::{FromRow, Row};

use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct AdminUser {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub struct NewAdminUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub display_name: Option<&'a str>,
    pub role: &'a str,
}

pub async fn ensure_admin_tables(pool: &DbPool) -> Result<(), sqlx::Error> {
    // 管理员表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admin_users (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(64) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            display_name VARCHAR(100) NULL,
            role VARCHAR(32) NOT NULL DEFAULT 'admin',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            last_login_at DATETIME NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_id(pool: &DbPool, id: u64) -> Result<Option<AdminUser>, sqlx::Error> {
    sqlx::query_as::<_, AdminUser>(
        "SELECT id, username, password_hash, display_name, role, created_at, updated_at, last_login_at FROM admin_users WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_by_username(pool: &DbPool, username: &str) -> Result<Option<AdminUser>, sqlx::Error> {
    sqlx::query_as::<_, AdminUser>(
        "SELECT id, username, password_hash, display_name, role, created_at, updated_at, last_login_at FROM admin_users WHERE username = ? LIMIT 1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn insert_admin_user(pool: &DbPool, payload: &NewAdminUser<'_>) -> Result<AdminUser, sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO admin_users (username, password_hash, display_name, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(payload.username)
    .bind(payload.password_hash)
    .bind(payload.display_name)
    .bind(payload.role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_by_username(pool, payload.username)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_last_login(pool: &DbPool, user_id: u64) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query("UPDATE admin_users SET last_login_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(())
}

pub async fn count_admin_users(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM admin_users")
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<i64, _>("count")?)
}
