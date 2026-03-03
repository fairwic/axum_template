use utoipa::OpenApi;

use crate::handlers::{auth_handler, member_handler};
use axum_application::dtos::user_dto::{UserResponse, WechatLoginDto};
use axum_common::response;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Backend Template API",
        version = "1.0.0",
        description = "Minimal user CRUD example"
    ),
    paths(
        auth_handler::wechat_login,
        member_handler::member_status,
        member_handler::member_benefits,
    ),
    components(
        schemas(
            WechatLoginDto,
            UserResponse,
            auth_handler::LoginResponse,
            member_handler::MemberStatusResponse,
            member_handler::MemberBenefitsResponse,
            response::ApiResponse<auth_handler::LoginResponse>,
            response::ApiResponse<member_handler::MemberStatusResponse>,
            response::ApiResponse<member_handler::MemberBenefitsResponse>,
            response::ErrorDetail,
            response::FieldError,
        )
    ),
    tags(
        (name = "Auth", description = "Login and tokens"),
        (name = "Member", description = "Member status and benefits"),
        (name = "System", description = "System endpoints")
    )
)]
pub struct ApiDoc;
