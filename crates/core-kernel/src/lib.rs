//! Core kernel: shared primitives and error model.

pub mod error;

pub use error::{AppError, AppResult, DomainError};
