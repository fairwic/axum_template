//! Address service

use std::sync::Arc;

use crate::dtos::address_dto::{CreateAddressInput, UpdateAddressInput};
use axum_core_kernel::{AppError, AppResult};
use axum_domain::address::entity::Address;
use axum_domain::address::repo::AddressRepository;
use ulid::Ulid;

#[derive(Clone)]
pub struct AddressService {
    repo: Arc<dyn AddressRepository>,
}

impl AddressService {
    pub fn new(repo: Arc<dyn AddressRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, user_id: Ulid) -> AppResult<Vec<Address>> {
        let mut items = self.repo.list_by_user(user_id).await?;
        items.sort_by(|a, b| {
            b.is_default
                .cmp(&a.is_default)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        Ok(items)
    }

    pub async fn get_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Address> {
        self.repo
            .find_by_id(user_id, address_id)
            .await?
            .ok_or_else(|| AppError::NotFound("address not found".into()))
    }

    pub async fn create(&self, user_id: Ulid, input: CreateAddressInput) -> AppResult<Address> {
        let existing = self.repo.list_by_user(user_id).await?;
        let should_default = input.is_default || existing.is_empty();

        if should_default {
            self.clear_default(user_id).await?;
        }

        let address = Address::new(
            user_id,
            input.name,
            input.phone,
            input.detail,
            input.lat,
            input.lng,
            should_default,
        )?;
        self.repo.create(&address).await
    }

    pub async fn update(
        &self,
        user_id: Ulid,
        address_id: Ulid,
        input: UpdateAddressInput,
    ) -> AppResult<Address> {
        let mut address = self
            .repo
            .find_by_id(user_id, address_id)
            .await?
            .ok_or_else(|| AppError::NotFound("address not found".into()))?;

        let should_default = if address.is_default {
            true
        } else {
            input.is_default
        };

        if should_default {
            self.clear_default(user_id).await?;
        }

        address.update(
            input.name,
            input.phone,
            input.detail,
            input.lat,
            input.lng,
            should_default,
        )?;
        self.repo.update(&address).await
    }

    pub async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()> {
        let target = self
            .repo
            .find_by_id(user_id, address_id)
            .await?
            .ok_or_else(|| AppError::NotFound("address not found".into()))?;
        self.repo.delete(user_id, address_id).await?;

        if target.is_default {
            let remaining = self.repo.list_by_user(user_id).await?;
            if let Some(mut candidate) = remaining.into_iter().next() {
                candidate.set_default(true);
                self.repo.update(&candidate).await?;
            }
        }

        Ok(())
    }

    pub async fn set_default(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Address> {
        let mut target = self
            .repo
            .find_by_id(user_id, address_id)
            .await?
            .ok_or_else(|| AppError::NotFound("address not found".into()))?;
        self.clear_default(user_id).await?;
        target.set_default(true);
        self.repo.update(&target).await
    }

    async fn clear_default(&self, user_id: Ulid) -> AppResult<()> {
        let addresses = self.repo.list_by_user(user_id).await?;
        for mut address in addresses.into_iter().filter(|item| item.is_default) {
            address.set_default(false);
            self.repo.update(&address).await?;
        }
        Ok(())
    }
}
