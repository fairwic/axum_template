use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：AdminLoginDto，管理员登录请求参数
pub struct AdminLoginDto {
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：password，登录密码
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：AdminResponse，管理员信息响应数据
pub struct AdminResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：role，管理员角色
    pub role: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：AdminLoginResponse，管理员登录响应数据
pub struct AdminLoginResponse {
    /// 参数：token，认证令牌
    pub token: String,
    /// 参数：admin，管理员信息
    pub admin: AdminResponse,
}
