use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::{AddressService, CreateAddressInput, UpdateAddressInput};
use axum_core_kernel::AppResult;
use axum_domain::address::entity::Address;
use axum_domain::address::repo::AddressRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryAddressRepo {
    inner: Mutex<HashMap<Ulid, Address>>,
}

#[async_trait]
impl AddressRepository for InMemoryAddressRepo {
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Option<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .get(&address_id)
            .cloned()
            .filter(|item| item.user_id == user_id))
    }

    async fn create(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn update(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if guard
            .get(&address_id)
            .map(|item| item.user_id == user_id)
            .unwrap_or(false)
        {
            guard.remove(&address_id);
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_create_first_address_auto_default() {
    let repo: Arc<dyn AddressRepository> = Arc::new(InMemoryAddressRepo::default());
    let service = AddressService::new(repo);
    let user_id = Ulid::new();

    let address = service
        .create(
            user_id,
            CreateAddressInput {
                name: "张三".into(),
                phone: "13800000000".into(),
                detail: "A区101".into(),
                lat: Some(30.0),
                lng: Some(120.0),
                is_default: false,
            },
        )
        .await
        .unwrap();

    assert!(address.is_default);
}

#[tokio::test]
async fn test_set_default_address() {
    let repo: Arc<dyn AddressRepository> = Arc::new(InMemoryAddressRepo::default());
    let service = AddressService::new(repo);
    let user_id = Ulid::new();

    let first = service
        .create(
            user_id,
            CreateAddressInput {
                name: "张三".into(),
                phone: "13800000000".into(),
                detail: "A区101".into(),
                lat: Some(30.0),
                lng: Some(120.0),
                is_default: true,
            },
        )
        .await
        .unwrap();
    let second = service
        .create(
            user_id,
            CreateAddressInput {
                name: "李四".into(),
                phone: "13900000000".into(),
                detail: "B区202".into(),
                lat: None,
                lng: None,
                is_default: false,
            },
        )
        .await
        .unwrap();

    service.set_default(user_id, second.id).await.unwrap();
    let list = service.list(user_id).await.unwrap();

    let first_now = list.iter().find(|item| item.id == first.id).unwrap();
    let second_now = list.iter().find(|item| item.id == second.id).unwrap();
    assert!(!first_now.is_default);
    assert!(second_now.is_default);
}

#[tokio::test]
async fn test_update_address() {
    let repo: Arc<dyn AddressRepository> = Arc::new(InMemoryAddressRepo::default());
    let service = AddressService::new(repo);
    let user_id = Ulid::new();

    let address = service
        .create(
            user_id,
            CreateAddressInput {
                name: "张三".into(),
                phone: "13800000000".into(),
                detail: "A区101".into(),
                lat: Some(30.0),
                lng: Some(120.0),
                is_default: true,
            },
        )
        .await
        .unwrap();

    let updated = service
        .update(
            user_id,
            address.id,
            UpdateAddressInput {
                name: "王五".into(),
                phone: "13700000000".into(),
                detail: "C区303".into(),
                lat: Some(31.0),
                lng: Some(121.0),
                is_default: true,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "王五");
    assert_eq!(updated.phone, "13700000000");
    assert_eq!(updated.detail, "C区303");
}
