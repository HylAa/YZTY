use crate::wechat::{cache::WechatCache, config::SharedConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const WECHAT_API_BASE: &str = "https://api.weixin.qq.com";
const ACCESS_TOKEN_CACHE_KEY: &str = "access_token";
const JSAPI_TICKET_CACHE_KEY: &str = "jsapi_ticket";
const TOKEN_EXPIRE_ADVANCE: u64 = 300; // 提前5分钟刷新
const TOKEN_TTL: u64 = 7000; // 缓存时间略小于7200秒

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    expires_in: Option<u64>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JsapiTicketResponse {
    ticket: Option<String>,
    expires_in: Option<u64>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WxSessionResponse {
    pub openid: Option<String>,
    pub session_key: Option<String>,
    pub unionid: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WxUserInfo {
    pub openid: String,
    pub nickname: Option<String>,
    pub sex: Option<i32>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub headimgurl: Option<String>,
    pub privilege: Option<Vec<String>>,
    pub unionid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WxUserResponse {
    pub openid: String,
    pub nickname: String,
    pub headimgurl: String,
}

pub struct WechatClient {
    config: SharedConfig,
    cache: WechatCache,
    http_client: Client,
}

impl WechatClient {
    pub fn new(config: SharedConfig) -> Self {
        Self {
            config,
            cache: WechatCache::new(),
            http_client: Client::new(),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, String> {
        // 检查缓存
        if let Some(token) = self.cache.get(ACCESS_TOKEN_CACHE_KEY).await {
            if !self
                .cache
                .should_refresh(ACCESS_TOKEN_CACHE_KEY, TOKEN_EXPIRE_ADVANCE)
                .await
            {
                return Ok(token);
            }
        }

        // 从微信API获取新token
        let url = format!(
            "{}/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            WECHAT_API_BASE, self.config.app_id, self.config.app_secret
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to request access token: {}", e))?;

        let token_response: AccessTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse access token response: {}", e))?;

        if let Some(errcode) = token_response.errcode {
            if errcode != 0 {
                return Err(format!(
                    "WeChat API error {}: {}",
                    errcode,
                    token_response.errmsg.unwrap_or_default()
                ));
            }
        }

        let access_token = token_response
            .access_token
            .ok_or_else(|| "Access token not found in response".to_string())?;

        // 缓存token
        self.cache
            .set(
                ACCESS_TOKEN_CACHE_KEY.to_string(),
                access_token.clone(),
                TOKEN_TTL,
            )
            .await;

        Ok(access_token)
    }

    pub async fn get_jsapi_ticket(&self) -> Result<String, String> {
        // 检查缓存
        if let Some(ticket) = self.cache.get(JSAPI_TICKET_CACHE_KEY).await {
            if !self
                .cache
                .should_refresh(JSAPI_TICKET_CACHE_KEY, TOKEN_EXPIRE_ADVANCE)
                .await
            {
                return Ok(ticket);
            }
        }

        // 获取access_token
        let access_token = self.get_access_token().await?;

        // 获取jsapi_ticket
        let url = format!(
            "{}/cgi-bin/ticket/getticket?access_token={}&type=jsapi",
            WECHAT_API_BASE, access_token
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to request jsapi ticket: {}", e))?;

        let ticket_response: JsapiTicketResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse jsapi ticket response: {}", e))?;

        if let Some(errcode) = ticket_response.errcode {
            if errcode != 0 {
                return Err(format!(
                    "WeChat API error {}: {}",
                    errcode,
                    ticket_response.errmsg.unwrap_or_default()
                ));
            }
        }

        let ticket = ticket_response
            .ticket
            .ok_or_else(|| "Jsapi ticket not found in response".to_string())?;

        // 缓存ticket
        self.cache
            .set(
                JSAPI_TICKET_CACHE_KEY.to_string(),
                ticket.clone(),
                TOKEN_TTL,
            )
            .await;

        Ok(ticket)
    }

    pub async fn get_user_info_by_code(&self, code: &str) -> Result<WxUserResponse, String> {
        // Step 1: 通过code获取access_token和openid
        let url = format!(
            "{}/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code",
            WECHAT_API_BASE, self.config.app_id, self.config.app_secret, code
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to exchange code for access token: {}", e))?;

        #[derive(Deserialize)]
        struct OAuthTokenResponse {
            access_token: Option<String>,
            openid: Option<String>,
            errcode: Option<i32>,
            errmsg: Option<String>,
        }

        let oauth_response: OAuthTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OAuth token response: {}", e))?;

        if let Some(errcode) = oauth_response.errcode {
            if errcode != 0 {
                return Err(format!(
                    "WeChat OAuth error {}: {}",
                    errcode,
                    oauth_response.errmsg.unwrap_or_default()
                ));
            }
        }

        let user_access_token = oauth_response
            .access_token
            .ok_or_else(|| "User access token not found".to_string())?;

        let openid = oauth_response
            .openid
            .ok_or_else(|| "OpenID not found".to_string())?;

        // Step 2: 使用access_token和openid获取用户信息
        let url = format!(
            "{}/sns/userinfo?access_token={}&openid={}&lang=zh_CN",
            WECHAT_API_BASE, user_access_token, openid
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get user info: {}", e))?;

        let user_info: WxUserInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info response: {}", e))?;

        Ok(WxUserResponse {
            openid: user_info.openid,
            nickname: user_info.nickname.unwrap_or_else(|| "微信用户".to_string()),
            headimgurl: user_info.headimgurl.unwrap_or_default(),
        })
    }

    pub async fn get_phone_session(&self, code: &str) -> Result<WxSessionResponse, String> {
        let _ = self.get_access_token().await?;

        let url = format!(
            "{}/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
            WECHAT_API_BASE, self.config.app_id, self.config.app_secret, code
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get session: {}", e))?;

        let session: WxSessionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse session response: {}", e))?;

        if let Some(errcode) = session.errcode {
            if errcode != 0 {
                return Err(format!(
                    "WeChat session error {}: {}",
                    errcode,
                    session.errmsg.as_ref().unwrap_or(&String::new())
                ));
            }
        }

        Ok(session)
    }
}

impl Clone for WechatClient {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            cache: self.cache.clone(),
            http_client: Client::new(),
        }
    }
}
