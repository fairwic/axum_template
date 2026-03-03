use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::CartService;
use axum_common::AppResult;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryCartRepo {
    carts: Mutex<HashMap<(Ulid, Ulid), Cart>>,
}

#[async_trait]
impl CartRepository for InMemoryCartRepo {
    async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<Cart>> {
        let guard = self.carts.lock().await;
        Ok(guard.get(&(user_id, store_id)).cloned())
    }

    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        let mut guard = self.carts.lock().await;
        let cart = Cart::new(user_id, store_id);
        guard.insert((user_id, store_id), cart.clone());
        Ok(cart)
    }

    async fn upsert_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        let cart = guard
            .entry((user_id, store_id))
            .or_insert_with(|| Cart::new(user_id, store_id));
        cart.upsert_item(product_id, qty, price_snapshot);
        Ok(())
    }

    async fn remove_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.remove_item(product_id);
        }
        Ok(())
    }

    async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()> {
        let mut guard = self.carts.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.items.clear();
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_cart_add_update_remove() {
    let repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let service = CartService::new(repo.clone());

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product_id = Ulid::new();

    service
        .add_item(user_id, store_id, product_id, 1, 990)
        .await
        .unwrap();

    let cart = service.get_cart(user_id, store_id).await.unwrap();
    assert_eq!(cart.items.len(), 1);

    service
        .add_item(user_id, store_id, product_id, 2, 990)
        .await
        .unwrap();
    let cart = service.get_cart(user_id, store_id).await.unwrap();
    assert_eq!(cart.items[0].qty, 2);

    service
        .remove_item(user_id, store_id, product_id)
        .await
        .unwrap();
    let cart = service.get_cart(user_id, store_id).await.unwrap();
    assert_eq!(cart.items.len(), 0);
}

#[tokio::test]
async fn test_cart_clear() {
    let repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let service = CartService::new(repo.clone());

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product_id = Ulid::new();

    service
        .add_item(user_id, store_id, product_id, 1, 990)
        .await
        .unwrap();

    service.clear(user_id, store_id).await.unwrap();
    let cart = service.get_cart(user_id, store_id).await.unwrap();
    assert_eq!(cart.items.len(), 0);
}
