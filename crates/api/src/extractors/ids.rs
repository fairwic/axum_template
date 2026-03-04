use axum_core_kernel::{AppError, AppResult};
use ulid::Ulid;

pub fn parse_ulid(value: &str, field: &str) -> AppResult<Ulid> {
    Ulid::from_string(value).map_err(|_| AppError::Validation(format!("invalid {field}")))
}
