//! Wechat mini-program auth client

use std::time::Duration;

use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use axum_domain::{WechatAuthClient, WechatSession};
use serde::Deserialize;

#[derive(Clone)]
pub struct WechatMiniProgramClient {
    client: reqwest::Client,
    app_id: String,
    app_secret: String,
    api_base: String,
}

impl WechatMiniProgramClient {
    pub fn new(
        app_id: String,
        app_secret: String,
        api_base: String,
        timeout_secs: u64,
    ) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| AppError::Internal(format!("failed to build wechat client: {err}")))?;

        Ok(Self {
            client,
            app_id,
            app_secret,
            api_base,
        })
    }
}

#[derive(Debug, Deserialize)]
struct Code2SessionResponse {
    openid: Option<String>,
    session_key: Option<String>,
    unionid: Option<String>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

#[async_trait]
impl WechatAuthClient for WechatMiniProgramClient {
    async fn code2session(&self, code: &str) -> AppResult<WechatSession> {
        if self.app_id.trim().is_empty() || self.app_secret.trim().is_empty() {
            return Err(AppError::Validation(
                "微信登录未配置 app_id/app_secret".into(),
            ));
        }
        if code.trim().is_empty() {
            return Err(AppError::Validation("微信登录 code 不能为空".into()));
        }

        let url = format!("{}/sns/jscode2session", self.api_base.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|err| AppError::Internal(format!("wechat request failed: {err}")))?;

        let payload: Code2SessionResponse = response
            .json()
            .await
            .map_err(|err| AppError::Internal(format!("wechat response parse failed: {err}")))?;

        if let Some(errcode) = payload.errcode {
            return Err(AppError::Validation(format!(
                "微信登录失败({errcode}): {}",
                payload.errmsg.unwrap_or_else(|| "未知错误".into())
            )));
        }

        let openid = payload
            .openid
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::Internal("wechat openid missing".into()))?;
        let session_key = payload
            .session_key
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::Internal("wechat session_key missing".into()))?;

        Ok(WechatSession {
            openid,
            session_key,
            unionid: payload.unionid,
        })
    }
}
