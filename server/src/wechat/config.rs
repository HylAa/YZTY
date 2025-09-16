use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct WechatConfig {
    pub app_id: String,
    pub app_secret: String,
}

impl WechatConfig {
    pub fn from_env() -> Result<Self, String> {
        let app_id = std::env::var("WECHAT_APPID")
            .map_err(|_| "WECHAT_APPID not set in environment")?;

        let app_secret = std::env::var("WECHAT_APPSECRET")
            .map_err(|_| "WECHAT_APPSECRET not set in environment")?;

        if app_id.is_empty() || app_secret.is_empty() {
            return Err("WeChat credentials cannot be empty".to_string());
        }

        Ok(Self {
            app_id,
            app_secret,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.app_id.starts_with("wx") {
            return Err("Invalid WeChat AppID format".to_string());
        }

        if self.app_secret.len() != 32 {
            return Err("Invalid WeChat AppSecret length".to_string());
        }

        Ok(())
    }
}

pub type SharedConfig = Arc<WechatConfig>;