use std::path::PathBuf;

use async_trait::async_trait;
use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_s3::{Client, config::Credentials, primitives::ByteStream};
use chrono::{Datelike, Timelike};
use serde_json::json;

use {
    crate::config::AppConfig,
    axum_domain::{
        DomainError,
        snapshot::model::{ProductSnapshot, ShopSnapshot},
        snapshot::ports::ColdStore,
    },
};

#[derive(Clone)]
pub struct S3ColdStore {
    client: Client,
    bucket: String,
    prefix: String,
}

impl S3ColdStore {
    pub async fn from_config(config: &AppConfig) -> Result<Self, DomainError> {
        let bucket = config
            .s3
            .bucket
            .clone()
            .ok_or_else(|| DomainError::Storage("COLD_S3_BUCKET is required".to_string()))?;

        let region_provider = RegionProviderChain::first_try(Some(
            aws_sdk_s3::config::Region::new(config.s3.region.clone()),
        ));

        let mut loader = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
        if let (Some(access_key), Some(secret_key)) =
            (config.s3.access_key.clone(), config.s3.secret_key.clone())
        {
            let credentials = Credentials::new(access_key, secret_key, None, None, "env");
            loader = loader.credentials_provider(credentials);
        }

        let shared_config = loader.load().await;
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&shared_config);
        if let Some(endpoint) = &config.s3.endpoint {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
            if config.s3.force_path_style {
                s3_config_builder = s3_config_builder.force_path_style(true);
            }
        }

        let client = Client::from_conf(s3_config_builder.build());
        Ok(Self {
            client,
            bucket,
            prefix: config.s3.prefix.trim_matches('/').to_string(),
        })
    }

    fn product_key(&self, snapshot: &ProductSnapshot) -> String {
        self.build_key(
            snapshot.platform.as_str(),
            "product",
            snapshot.observed_at,
            &snapshot.platform_product_id,
            snapshot.trace_id.to_string().as_str(),
        )
    }

    fn shop_key(&self, snapshot: &ShopSnapshot) -> String {
        self.build_key(
            snapshot.platform.as_str(),
            "shop",
            snapshot.observed_at,
            &snapshot.platform_shop_id,
            snapshot.trace_id.to_string().as_str(),
        )
    }

    fn build_key(
        &self,
        platform: &str,
        kind: &str,
        observed_at: chrono::DateTime<chrono::Utc>,
        aggregate_id: &str,
        trace_id: &str,
    ) -> String {
        let date = format!(
            "{:04}-{:02}-{:02}",
            observed_at.year(),
            observed_at.month(),
            observed_at.day()
        );
        let time = format!(
            "{:02}{:02}{:02}",
            observed_at.hour(),
            observed_at.minute(),
            observed_at.second()
        );

        let filename = format!("ts={time}_id={aggregate_id}_trace={trace_id}.json");
        let mut path = PathBuf::new();
        if !self.prefix.is_empty() {
            path.push(&self.prefix);
        }
        path.push(format!("platform={platform}"));
        path.push(format!("kind={kind}"));
        path.push(format!("dt={date}"));
        path.push(filename);
        path.to_string_lossy().replace('\\', "/")
    }

    async fn put_object(&self, key: String, payload: Vec<u8>) -> Result<(), DomainError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(payload))
            .send()
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ColdStore for S3ColdStore {
    async fn archive_product(&self, snapshot: &ProductSnapshot) -> Result<(), DomainError> {
        let key = self.product_key(snapshot);
        let record = json!({
            "trace_id": snapshot.trace_id,
            "platform": snapshot.platform.as_str(),
            "kind": "product",
            "observed_at": snapshot.observed_at,
            "payload": snapshot.raw_payload.clone(),
        });
        let payload =
            serde_json::to_vec(&record).map_err(|e| DomainError::Storage(e.to_string()))?;
        self.put_object(key, payload).await
    }

    async fn archive_shop(&self, snapshot: &ShopSnapshot) -> Result<(), DomainError> {
        let key = self.shop_key(snapshot);
        let record = json!({
            "trace_id": snapshot.trace_id,
            "platform": snapshot.platform.as_str(),
            "kind": "shop",
            "observed_at": snapshot.observed_at,
            "payload": snapshot.raw_payload.clone(),
        });
        let payload =
            serde_json::to_vec(&record).map_err(|e| DomainError::Storage(e.to_string()))?;
        self.put_object(key, payload).await
    }
}
