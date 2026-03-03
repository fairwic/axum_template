use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::AdminService;
use axum_common::AppResult;
use axum_domain::admin::entity::{Admin, AdminRole};
use axum_domain::admin::repo::AdminRepository;
use tokio::sync::Mutex;

#[derive(Default)]
struct InMemoryAdminRepo {
    inner: Mutex<HashMap<String, Admin>>,
}

#[async_trait]
impl AdminRepository for InMemoryAdminRepo {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(phone).cloned())
    }

    async fn create(&self, admin: &Admin) -> AppResult<Admin> {
        let mut guard = self.inner.lock().await;
        guard.insert(admin.phone.clone(), admin.clone());
        Ok(admin.clone())
    }
}

#[tokio::test]
async fn test_admin_login_success() {
    let repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let service = AdminService::new(repo);

    service
        .create_admin(
            "13800000000".into(),
            "pass".into(),
            AdminRole::Platform,
            None,
        )
        .await
        .unwrap();

    let admin = service.login("13800000000", "pass").await.unwrap();
    assert_eq!(admin.phone, "13800000000");
}

#[tokio::test]
async fn test_admin_login_wrong_password() {
    let repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let service = AdminService::new(repo);

    service
        .create_admin(
            "13800000001".into(),
            "pass".into(),
            AdminRole::Platform,
            None,
        )
        .await
        .unwrap();

    let err = service.login("13800000001", "bad").await.unwrap_err();
    assert!(err.to_string().contains("password"));
}
