use utoipa::OpenApi;

use crate::handlers::{admin_auth_handler, auth_handler, member_handler, store_handler};
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
        admin_auth_handler::admin_login,
        member_handler::member_status,
        member_handler::member_benefits,
        store_handler::nearby_stores,
    ),
    components(
        schemas(
            WechatLoginDto,
            UserResponse,
            auth_handler::LoginResponse,
            admin_auth_handler::AdminLoginDto,
            admin_auth_handler::AdminResponse,
            admin_auth_handler::AdminLoginResponse,
            member_handler::MemberStatusResponse,
            member_handler::MemberBenefitsResponse,
            store_handler::StoreNearbyResponse,
            response::ApiResponse<auth_handler::LoginResponse>,
            response::ApiResponse<admin_auth_handler::AdminLoginResponse>,
            response::ApiResponse<member_handler::MemberStatusResponse>,
            response::ApiResponse<member_handler::MemberBenefitsResponse>,
            response::ApiResponse<Vec<store_handler::StoreNearbyResponse>>,
            response::ErrorDetail,
            response::FieldError,
        )
    ),
    tags(
        (name = "Auth", description = "Login and tokens"),
        (name = "Admin", description = "Admin login"),
        (name = "Member", description = "Member status and benefits"),
        (name = "Store", description = "Store browsing"),
        (name = "System", description = "System endpoints")
    )
)]
pub struct ApiDoc;
