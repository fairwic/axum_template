use utoipa::OpenApi;

use crate::handlers::user_handler;
use axum_application::dtos::user_dto::{CreateUserDto, UpdateUserDto, UserResponse};
use axum_common::response;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Backend Template API",
        version = "1.0.0",
        description = "Minimal user CRUD example"
    ),
    paths(
        user_handler::create_user,
        user_handler::get_user,
        user_handler::list_users,
        user_handler::update_user,
        user_handler::delete_user,
    ),
    components(
        schemas(
            CreateUserDto,
            UpdateUserDto,
            UserResponse,
            response::ApiResponse<UserResponse>,
            response::ApiResponse<Vec<UserResponse>>,
            response::ApiResponse<String>,
            response::ErrorDetail,
            response::FieldError,
        )
    ),
    tags(
        (name = "User", description = "User CRUD"),
        (name = "System", description = "System endpoints")
    )
)]
pub struct ApiDoc;
