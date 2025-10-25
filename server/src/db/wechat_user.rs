use super::DbPool;
use chrono::{NaiveDateTime, Utc};
use sqlx::{Error, FromRow};

/// 数据库中的微信用户记录
#[derive(Debug, Clone, FromRow)]
pub struct WechatUser {
    pub id: u64,
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 新增或更新微信用户的请求载体
#[derive(Debug, Clone)]
pub struct NewWechatUser<'a> {
    pub openid: &'a str,
    pub nickname: Option<&'a str>,
    pub avatar: Option<&'a str>,
}
/// 通过 openid 查询微信用户
pub async fn get_by_openid(pool: &DbPool, openid: &str) -> Result<Option<WechatUser>, sqlx::Error> {
    let record = sqlx::query_as::<_, WechatUser>(
        "SELECT id, openid, nickname, avatar, phone, created_at, updated_at FROM wechat_users WHERE openid = ? LIMIT 1",
    )
    .bind(openid)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}
/// 插入或更新微信用户信息
pub async fn upsert_wechat_user(
    pool: &DbPool,
    payload: &NewWechatUser<'_>,
) -> Result<WechatUser, sqlx::Error> {
    let now = Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO wechat_users (openid, nickname, avatar, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE nickname = VALUES(nickname), avatar = VALUES(avatar), updated_at = VALUES(updated_at)"
    )
    .bind(payload.openid)
    .bind(payload.nickname)
    .bind(payload.avatar)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_by_openid(pool, payload.openid)
        .await?
        .ok_or(Error::RowNotFound)
}
/// 绑定手机号，成功后返回最新记录
pub async fn bind_phone(
    pool: &DbPool,
    openid: &str,
    phone: &str,
) -> Result<Option<WechatUser>, sqlx::Error> {
    let now = Utc::now().naive_utc();

    sqlx::query("UPDATE wechat_users SET phone = ?, updated_at = ? WHERE openid = ?")
        .bind(phone)
        .bind(now)
        .bind(openid)
        .execute(pool)
        .await?;

    get_by_openid(pool, openid).await
}
