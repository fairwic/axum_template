use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::ProductService;
use axum_common::AppResult;
use axum_domain::product::entity::{Product, ProductStatus};
use axum_domain::product::repo::ProductRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryProductRepo {
    inner: Mutex<HashMap<Ulid, Product>>,
}

#[async_trait]
impl ProductRepository for InMemoryProductRepo {
    async fn list_by_category(
        &self,
        store_id: Ulid,
        category_id: Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|p| p.store_id == store_id && p.category_id == category_id)
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let guard = self.inner.lock().await;
        let items: Vec<Product> = guard
            .values()
            .filter(|p| p.store_id == store_id && p.title.contains(keyword))
            .cloned()
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn create(&self, product: &Product) -> AppResult<Product> {
        let mut guard = self.inner.lock().await;
        guard.insert(product.id, product.clone());
        Ok(product.clone())
    }
}

#[tokio::test]
async fn test_search_products() {
    let repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let service = ProductService::new(repo.clone());

    let store_id = Ulid::new();
    let category_id = Ulid::new();
    let product = Product::new(
        store_id,
        category_id,
        "椰子水".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        10,
        ProductStatus::On,
        vec!["new".into()],
    )
    .unwrap();
    repo.create(&product).await.unwrap();

    let page = service.search(store_id, "椰子", 1, 20).await.unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].title, "椰子水");
}
