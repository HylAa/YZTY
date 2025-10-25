use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{db::admin, ApiResponse, AppState};

static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    std::env::var("ADMIN_JWT_SECRET").unwrap_or_else(|_| {
        eprintln!(
            "警告: 未设置 ADMIN_JWT_SECRET，使用默认开发密钥。请在生产环境中设置强随机密钥。"
        );
        "dev-secret-change-me".to_string()
    })
});

fn encoding_key() -> EncodingKey {
    EncodingKey::from_secret(JWT_SECRET.as_bytes())
}

fn decoding_key() -> DecodingKey {
    DecodingKey::from_secret(JWT_SECRET.as_bytes())
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hashed)
}

pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    pub sub: String,
    pub uid: u64,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl AuthClaims {
    pub fn new(user_id: u64, username: &str, role: &str, ttl_hours: i64) -> Self {
        let now = Utc::now().timestamp();
        AuthClaims {
            sub: username.to_string(),
            uid: user_id,
            role: role.to_string(),
            iat: now,
            exp: now + Duration::hours(ttl_hours).num_seconds(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AdminProfile,
}

#[derive(Debug, Serialize)]
pub struct AdminProfile {
    pub id: u64,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
}

pub async fn api_admin_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> (StatusCode, Json<ApiResponse<LoginResponse>>) {
    let user = match admin::get_by_username(&state.db_pool, &payload.username).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::msg(401, "用户名或密码错误")),
            );
        }
        Err(err) => {
            eprintln!("查询管理员失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "服务器查询失败")),
            );
        }
    };

    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::msg(401, "用户名或密码错误")),
            );
        }
        Err(err) => {
            eprintln!("密码校验失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "密码校验失败")),
            );
        }
    }

    if let Err(err) = admin::update_last_login(&state.db_pool, user.id).await {
        eprintln!("更新最后登录时间失败: {}", err);
    }

    let ttl = std::env::var("ADMIN_JWT_TTL_HOURS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(24);

    let claims = AuthClaims::new(user.id, &user.username, &user.role, ttl);
    let token = match encode(&Header::new(Algorithm::HS256), &claims, &encoding_key()) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("生成 JWT 失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "生成令牌失败")),
            );
        }
    };

    let profile = AdminProfile {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::ok(LoginResponse {
            token,
            user: profile,
        })),
    )
}

pub async fn api_auth_me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<ApiResponse<AdminProfile>>) {
    let claims = match extract_claims_from_headers(&headers) {
        Ok(claims) => claims,
        Err(resp) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<AdminProfile> {
                    code: resp.code,
                    message: resp.message,
                    data: None,
                    pagination: None,
                    total: None,
                }),
            );
        }
    };

    let user = match admin::get_by_id(&state.db_pool, claims.uid).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::msg(401, "管理员不存在")),
            );
        }
        Err(err) => {
            eprintln!("查询管理员失败: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::msg(500, "查询管理员失败")),
            );
        }
    };

    let profile = AdminProfile {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    };

    (StatusCode::OK, Json(ApiResponse::ok(profile)))
}

pub fn extract_claims_from_headers(headers: &HeaderMap) -> Result<AuthClaims, ApiResponse<()>> {
    let auth_value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiResponse::msg(401, "缺少认证信息"))?;

    if !auth_value.starts_with("Bearer ") {
        return Err(ApiResponse::msg(401, "认证头格式错误"));
    }

    let token = &auth_value[7..];
    decode::<AuthClaims>(token, &decoding_key(), &Validation::new(Algorithm::HS256))
        .map(|data| data.claims)
        .map_err(|_| ApiResponse::msg(401, "登录状态已失效"))
}
