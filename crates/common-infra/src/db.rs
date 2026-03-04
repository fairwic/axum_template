//! SQLx error mapping helpers shared by infrastructure adapters.

use axum_core_kernel::AppError;

pub fn map_sqlx_error(err: sqlx::Error) -> AppError {
    AppError::database(err)
}

pub fn map_unique_violation(err: sqlx::Error, conflict_message: &'static str) -> AppError {
    match err {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            AppError::Conflict(conflict_message.to_owned())
        }
        other => map_sqlx_error(other),
    }
}
