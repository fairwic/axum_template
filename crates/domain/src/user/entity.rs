//! User entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: Ulid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, email: String) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        Self::validate_email(&email)?;

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            name,
            email,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update(&mut self, name: String, email: String) -> Result<(), DomainError> {
        Self::validate_name(&name)?;
        Self::validate_email(&email)?;

        self.name = name;
        self.email = email;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("name is required".into()));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), DomainError> {
        if email.trim().is_empty() || !email.contains('@') {
            return Err(DomainError::Validation("email is invalid".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new_rejects_empty_name() {
        let err = User::new("".into(), "a@b.com".into()).unwrap_err();
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_user_new_rejects_invalid_email() {
        let err = User::new("Alice".into(), "invalid".into()).unwrap_err();
        assert!(err.to_string().contains("email"));
    }

    #[test]
    fn test_user_new_success() {
        let user = User::new("Alice".into(), "a@b.com".into()).unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, "a@b.com");
    }
}
