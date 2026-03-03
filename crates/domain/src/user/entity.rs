//! User entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: Ulid,
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub is_member: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        openid: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> Result<Self, DomainError> {
        Self::validate_openid(&openid)?;

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            openid,
            nickname,
            avatar,
            phone: None,
            is_member: true,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_openid(openid: &str) -> Result<(), DomainError> {
        if openid.trim().is_empty() {
            return Err(DomainError::Validation("openid is required".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new_rejects_empty_openid() {
        let err = User::new("".into(), None, None).unwrap_err();
        assert!(err.to_string().contains("openid"));
    }

    #[test]
    fn test_user_new_success() {
        let user = User::new("openid-1".into(), Some("Alice".into()), None).unwrap();
        assert_eq!(user.openid, "openid-1");
        assert!(user.is_member);
    }
}
