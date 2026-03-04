use async_trait::async_trait;
use scylla::{Session, SessionBuilder};

use {
    crate::config::AppConfig,
    axum_domain::{
        DomainError,
        snapshot::model::{ProductSnapshot, ShopSnapshot},
        snapshot::ports::HotStore,
    },
};

pub struct ScyllaHotStore {
    session: Session,
    keyspace: String,
    product_table_prefix: String,
    shop_table_prefix: String,
    auto_create: bool,
}

impl ScyllaHotStore {
    pub async fn from_config(config: &AppConfig) -> Result<Self, DomainError> {
        let mut builder = SessionBuilder::new();
        for node in &config.scylla.contact_points {
            builder = builder.known_node(node);
        }

        let session = builder
            .build()
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        Ok(Self {
            session,
            keyspace: config.scylla.keyspace.clone(),
            product_table_prefix: config.scylla.product_table_prefix.clone(),
            shop_table_prefix: config.scylla.shop_table_prefix.clone(),
            auto_create: config.scylla.auto_create,
        })
    }

    fn product_table(&self, platform: &str) -> String {
        build_table_name(self.product_table_prefix.as_str(), platform)
    }

    fn shop_table(&self, platform: &str) -> String {
        build_table_name(self.shop_table_prefix.as_str(), platform)
    }

    async fn ensure_tables(&self, platform: &str) -> Result<(), DomainError> {
        if !self.auto_create {
            return Ok(());
        }

        let product_table = self.product_table(platform);
        let shop_table = self.shop_table(platform);

        let create_product = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
              product_id text PRIMARY KEY,
              platform text,
              shop_id text,
              sku text,
              title text,
              price_minor bigint,
              old_price_minor bigint,
              sales bigint,
              rating double,
              rating_count bigint,
              category_id text,
              category_level1_id text,
              category_slug text,
              vendor_id text,
              image_urls list<text>,
              updated_at bigint
            )
            "#,
            self.keyspace, product_table
        );

        let create_shop = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
              shop_id text PRIMARY KEY,
              platform text,
              name text,
              score double,
              logo text,
              followers bigint,
              comment_count bigint,
              total_sales bigint,
              information text,
              updated_at bigint
            )
            "#,
            self.keyspace, shop_table
        );

        self.session
            .query(create_product, ())
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        self.session
            .query(create_shop, ())
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl HotStore for ScyllaHotStore {
    async fn save_product(&self, snapshot: &ProductSnapshot) -> Result<(), DomainError> {
        self.ensure_tables(snapshot.platform.as_str()).await?;

        let table = self.product_table(snapshot.platform.as_str());
        let cql = product_insert_cql(self.keyspace.as_str(), table.as_str());

        let updated_at = snapshot.observed_at.timestamp();
        let values = (
            snapshot.platform_product_id.as_str(),
            snapshot.platform.as_str(),
            snapshot.platform_shop_id.as_str(),
            snapshot.sku.as_deref(),
            snapshot.title.as_str(),
            snapshot.price_minor,
            snapshot.old_price_minor,
            snapshot.sales,
            snapshot.rating,
            snapshot.rating_count,
            snapshot.category_id.as_deref(),
            snapshot.category_level1_id.as_deref(),
            snapshot.category_slug.as_deref(),
            snapshot.vendor_id.as_deref(),
            snapshot.image_urls.clone(),
            updated_at,
        );

        self.session
            .query(cql, values)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn save_shop(&self, snapshot: &ShopSnapshot) -> Result<(), DomainError> {
        self.ensure_tables(snapshot.platform.as_str()).await?;

        let table = self.shop_table(snapshot.platform.as_str());
        let cql = shop_insert_cql(self.keyspace.as_str(), table.as_str());

        let updated_at = snapshot.observed_at.timestamp();
        let values = (
            snapshot.platform_shop_id.as_str(),
            snapshot.platform.as_str(),
            snapshot.name.as_deref(),
            snapshot.score,
            snapshot.logo.as_deref(),
            snapshot.followers,
            snapshot.comment_count,
            snapshot.total_sales,
            snapshot.information.as_deref(),
            updated_at,
        );

        self.session
            .query(cql, values)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        Ok(())
    }
}

fn normalize_platform(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn build_table_name(prefix: &str, platform: &str) -> String {
    format!("{}_{}", prefix, normalize_platform(platform))
}

fn product_insert_cql(keyspace: &str, table: &str) -> String {
    format!(
        r#"INSERT INTO {}.{} (
          product_id,
          platform,
          shop_id,
          sku,
          title,
          price_minor,
          old_price_minor,
          sales,
          rating,
          rating_count,
          category_id,
          category_level1_id,
          category_slug,
          vendor_id,
          image_urls,
          updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        keyspace, table
    )
}

fn shop_insert_cql(keyspace: &str, table: &str) -> String {
    format!(
        r#"INSERT INTO {}.{} (
          shop_id,
          platform,
          name,
          score,
          logo,
          followers,
          comment_count,
          total_sales,
          information,
          updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        keyspace, table
    )
}

#[cfg(test)]
mod tests {
    use super::{build_table_name, normalize_platform, product_insert_cql, shop_insert_cql};

    #[test]
    fn platform_suffix_is_sanitized() {
        assert_eq!(normalize_platform("Yandex"), "yandex");
        assert_eq!(normalize_platform("temu-1"), "temu_1");
    }

    #[test]
    fn build_scylla_table_name() {
        assert_eq!(
            build_table_name("product_by_id", "yandex"),
            "product_by_id_yandex"
        );
    }

    #[test]
    fn product_insert_cql_contains_table() {
        let cql = product_insert_cql("catalog", "product_by_id_yandex");
        assert!(cql.contains("INSERT INTO catalog.product_by_id_yandex"));
    }

    #[test]
    fn shop_insert_cql_contains_table() {
        let cql = shop_insert_cql("catalog", "shop_by_id_yandex");
        assert!(cql.contains("INSERT INTO catalog.shop_by_id_yandex"));
    }
}
