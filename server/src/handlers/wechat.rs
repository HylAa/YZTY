use crate::{
    db::wechat_user::{bind_phone as bind_phone_record, upsert_wechat_user, NewWechatUser, WechatUser as StoredWechatUser},
    wechat::crypto,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}

#[derive(Deserialize)]
pub struct JssdkReq {
    url: String,
}

#[derive(Serialize)]
pub struct JssdkCfg {
    #[serde(rename = "appId")]
    app_id: String,
    timestamp: u64,
    #[serde(rename = "nonceStr")]
    nonce_str: String,
    signature: String,
}

pub async fn api_wechat_jssdk(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JssdkReq>,
) -> impl IntoResponse {
    let client = state.wechat_client.clone();
    // 生成时间戳和随机字符串
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let nonce_str = crypto::generate_nonce_str(16);

    // 获取jsapi_ticket
    match client.get_jsapi_ticket().await {
        Ok(ticket) => {
            // 计算签名
            let signature = crypto::calculate_signature(&ticket, &nonce_str, timestamp, &req.url);

            let cfg = JssdkCfg {
                app_id: std::env::var("WECHAT_APPID").unwrap_or_default(),
                timestamp,
                nonce_str,
                signature,
            };

            (StatusCode::OK, Json(ApiResponse::ok(cfg)))
        }
        Err(e) => {
            eprintln!("Failed to get jsapi ticket: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<JssdkCfg>::error(
                    500,
                    format!("Failed to generate JSSDK config: {}", e),
                )),
            )
        }
    }
}

#[derive(Deserialize)]
pub struct CodeReq {
    code: String,
}

#[derive(Deserialize)]
pub struct BindPhoneReq {
    openid: String,
    phone: String,
}

#[derive(Serialize)]
pub struct WxUser {
    openid: String,
    nickname: String,
    headimgurl: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

impl From<StoredWechatUser> for WxUser {
    fn from(user: StoredWechatUser) -> Self {
        let StoredWechatUser {
            openid,
            nickname,
            avatar,
            phone,
            ..
        } = user;

        Self {
            openid,
            nickname: nickname.unwrap_or_else(|| "微信用户".to_string()),
            headimgurl: avatar.unwrap_or_default(),
            phone,
        }
    }
}

pub async fn api_wechat_userinfo(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CodeReq>,
) -> impl IntoResponse {
    let client = state.wechat_client.clone();
    match client.get_user_info_by_code(&req.code).await {
        Ok(user_info) => {
            let fallback_nickname = user_info.nickname.clone();
            let fallback_avatar = user_info.headimgurl.clone();
            let nickname_ref = (!fallback_nickname.trim().is_empty()).then_some(fallback_nickname.as_str());
            let avatar_ref = (!fallback_avatar.trim().is_empty()).then_some(fallback_avatar.as_str());

            match upsert_wechat_user(
                &state.db_pool,
                &NewWechatUser {
                    openid: &user_info.openid,
                    nickname: nickname_ref,
                    avatar: avatar_ref,
                },
            )
            .await
            {
                Ok(stored) => {
                    let StoredWechatUser {
                        openid,
                        nickname,
                        avatar,
                        phone,
                        ..
                    } = stored;

                    let user = WxUser {
                        openid,
                        nickname: nickname.unwrap_or(fallback_nickname),
                        headimgurl: avatar.unwrap_or(fallback_avatar),
                        phone,
                    };
                    (StatusCode::OK, Json(ApiResponse::ok(user)))
                }
                Err(e) => {
                    eprintln!("Failed to persist wechat user: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<WxUser>::error(
                            500,
                            "保存微信用户信息失败".to_string(),
                        )),
                    )
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get user info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<WxUser>::error(
                    500,
                    format!("Failed to get user info: {}", e),
                )),
            )
        }
    }
}

#[derive(Deserialize)]
pub struct DecryptReq {
    #[serde(rename = "encryptedData")]
    encrypted_data: String,
    iv: String,
    code: Option<String>, // 用于获取session_key的code
    session_key: Option<String>, // 或直接提供session_key
}

#[derive(Serialize)]
pub struct PhoneRes {
    #[serde(rename = "phoneNumber")]
    phone_number: String,
}

pub async fn api_wechat_decrypt_phone(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DecryptReq>,
) -> impl IntoResponse {
    let client = state.wechat_client.clone();
    // 获取session_key
    let session_key = if let Some(key) = req.session_key {
        key
    } else if let Some(code) = req.code {
        match client.get_phone_session(&code).await {
            Ok(session) => {
                session.session_key.unwrap_or_else(|| {
                    eprintln!("Session key not found in response");
                    String::new()
                })
            }
            Err(e) => {
                eprintln!("Failed to get session: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<PhoneRes>::error(
                        500,
                        format!("Failed to get session: {}", e),
                    )),
                );
            }
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<PhoneRes>::error(
                400,
                "Either code or session_key must be provided".to_string(),
            )),
        );
    };

    if session_key.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<PhoneRes>::error(
                500,
                "Failed to obtain session key".to_string(),
            )),
        );
    }

    // 解密手机号数据
    match crypto::decrypt_phone_data(&req.encrypted_data, &session_key, &req.iv) {
        Ok(decrypted_json) => {
            // 解析解密后的JSON数据
            #[derive(Deserialize)]
            struct PhoneData {
                #[serde(rename = "phoneNumber")]
                phone_number: Option<String>,
                #[serde(rename = "purePhoneNumber")]
                pure_phone_number: Option<String>,
            }

            match serde_json::from_str::<PhoneData>(&decrypted_json) {
                Ok(phone_data) => {
                    let phone_number = phone_data
                        .phone_number
                        .or(phone_data.pure_phone_number)
                        .unwrap_or_else(|| {
                            eprintln!("Phone number not found in decrypted data");
                            "".to_string()
                        });

                    if phone_number.is_empty() {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<PhoneRes>::error(
                                500,
                                "Phone number not found in decrypted data".to_string(),
                            )),
                        )
                    } else {
                        (
                            StatusCode::OK,
                            Json(ApiResponse::ok(PhoneRes { phone_number })),
                        )
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse decrypted phone data: {}", e);
                    eprintln!("Decrypted JSON: {}", decrypted_json);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<PhoneRes>::error(
                            500,
                            format!("Failed to parse phone data: {}", e),
                        )),
                    )
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to decrypt phone data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<PhoneRes>::error(
                    500,
                    format!("Failed to decrypt phone data: {}", e),
                )),
            )
        }
    }
}
pub async fn api_wechat_bind_phone(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BindPhoneReq>,
) -> impl IntoResponse {
    let BindPhoneReq { openid, phone } = req;
    let openid_ref = openid.trim();
    if openid_ref.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<WxUser>::error(400, "缺少 openid".to_string())),
        );
    }

    let phone_ref = phone.trim();
    let valid_phone = phone_ref.len() == 11 && phone_ref.chars().all(|c| c.is_ascii_digit());
    if !valid_phone {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<WxUser>::error(400, "手机号格式不正确".to_string())),
        );
    }

    match bind_phone_record(&state.db_pool, openid_ref, phone_ref).await {
        Ok(Some(record)) => (StatusCode::OK, Json(ApiResponse::ok(WxUser::from(record)))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<WxUser>::error(404, "未找到对应的微信用户".to_string())),
        ),
        Err(e) => {
            eprintln!("Failed to bind phone: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<WxUser>::error(500, "绑定手机号失败".to_string())),
            )
        }
    }
}
