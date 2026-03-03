//! User DTOs

use axum_domain::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：WechatLoginDto，微信登录请求参数
pub struct WechatLoginDto {
    /// 参数：code，微信登录临时凭证
    pub code: String,
    /// 参数：nickname，昵称
    pub nickname: Option<String>,
    /// 参数：avatar，头像 URL
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：SendSmsCodeDto，发送短信验证码请求参数
pub struct SendSmsCodeDto {
    /// 参数：phone，手机号
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PhoneSmsLoginDto，手机号验证码登录请求参数
pub struct PhoneSmsLoginDto {
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：sms_code，短信验证码
    pub sms_code: String,
    /// 参数：wechat_code，微信授权码
    pub wechat_code: String,
    /// 参数：nickname，昵称
    pub nickname: Option<String>,
    /// 参数：avatar，头像 URL
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：UserResponse，用户信息响应数据
pub struct UserResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：openid，微信 OpenID
    pub openid: String,
    /// 参数：nickname，昵称
    pub nickname: Option<String>,
    /// 参数：avatar，头像 URL
    pub avatar: Option<String>,
    /// 参数：phone，手机号
    pub phone: Option<String>,
    /// 参数：is_member，是否会员
    pub is_member: bool,
    /// 参数：created_at，创建时间
    pub created_at: DateTime<Utc>,
    /// 参数：updated_at，更新时间
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            openid: user.openid,
            nickname: user.nickname,
            avatar: user.avatar,
            phone: user.phone,
            is_member: user.is_member,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
