use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：MemberStatusResponse，会员状态响应数据
pub struct MemberStatusResponse {
    /// 参数：is_member，是否会员
    pub is_member: bool,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：MemberBenefitsResponse，会员权益响应数据
pub struct MemberBenefitsResponse {
    /// 参数：benefits，会员权益列表
    pub benefits: Vec<String>,
}
