//! 内存热存储实现
//!
//! 脚手架默认的 `HotStore` 实现，用 `Arc<Mutex<HashMap>>` 保存最近一次写入的
//! 产品/店铺快照，替代生产环境的 ScyllaDB。适用于开发、测试与最小可运行示例。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use axum_domain::{
    snapshot::model::{ProductSnapshot, ShopSnapshot},
    snapshot::ports::HotStore,
    DomainError,
};

/// 基于内存的热存储，键为 `platform:aggregate_id`。
#[derive(Clone, Default)]
pub struct InMemoryHotStore {
    products: Arc<Mutex<HashMap<String, ProductSnapshot>>>,
    shops: Arc<Mutex<HashMap<String, ShopSnapshot>>>,
}

impl InMemoryHotStore {
    /// 创建空的内存热存储。
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HotStore for InMemoryHotStore {
    async fn save_product(&self, snapshot: &ProductSnapshot) -> Result<(), DomainError> {
        let key = format!(
            "{}:{}",
            snapshot.platform.as_str(),
            snapshot.platform_product_id
        );
        self.products.lock().await.insert(key, snapshot.clone());
        Ok(())
    }

    async fn save_shop(&self, snapshot: &ShopSnapshot) -> Result<(), DomainError> {
        let key = format!(
            "{}:{}",
            snapshot.platform.as_str(),
            snapshot.platform_shop_id
        );
        self.shops.lock().await.insert(key, snapshot.clone());
        Ok(())
    }
}
