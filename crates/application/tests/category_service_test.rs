use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::CategoryService;
use axum_common::AppResult;
use axum_domain::category::entity::{Category, CategoryStatus};
use axum_domain::category::repo::CategoryRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryCategoryRepo {
    inner: Mutex<HashMap<Ulid, Category>>,
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepo {
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.store_id == store_id)
            .cloned()
            .collect())
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
        let mut guard = self.inner.lock().await;
        guard.insert(category.id, category.clone());
        Ok(category.clone())
    }
}

#[tokio::test]
async fn test_list_categories_by_store() {
    let repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let service = CategoryService::new(repo.clone());

    let store_id = Ulid::new();
    let category = Category::new(store_id, "饮料".into(), 1, CategoryStatus::On).unwrap();
    repo.create(&category).await.unwrap();

    let list = service.list_by_store(store_id).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "饮料");
}
