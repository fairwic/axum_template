//! Cart entity

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CartItem {
    pub product_id: Ulid,
    pub qty: i32,
    pub price_snapshot: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cart {
    pub id: Ulid,
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub items: Vec<CartItem>,
}

impl Cart {
    pub fn new(user_id: Ulid, store_id: Ulid) -> Self {
        Self {
            id: Ulid::new(),
            user_id,
            store_id,
            items: Vec::new(),
        }
    }

    pub fn upsert_item(&mut self, product_id: Ulid, qty: i32, price_snapshot: i32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            item.qty = qty;
            item.price_snapshot = price_snapshot;
            return;
        }
        self.items.push(CartItem {
            product_id,
            qty,
            price_snapshot,
        });
    }

    pub fn remove_item(&mut self, product_id: Ulid) {
        self.items.retain(|item| item.product_id != product_id);
    }
}
