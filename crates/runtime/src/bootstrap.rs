use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::services::store_service::LbsService;
use axum_application::{
    AddressService, AdminService, CartService, CategoryService, OrderService, ProductService,
    RunnerOrderService, StoreService, UserService,
};
use axum_domain::{CacheService, SmsGateway, TransactionManager, WechatAuthClient};
use axum_infra::{
    AppConfig, CacheProvider, LbsProvider, LogSmsGateway, MemoryCacheService, PgAddressRepository,
    PgAdminRepository, PgCartRepository, PgCategoryRepository, PgGoodsOrderRepository,
    PgProductRepository, PgRunnerOrderRepository, PgStoreRepository, PgTransactionManager,
    PgUserRepository, RedisCacheService, WechatMiniProgramClient,
};

pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let store_repo = Arc::new(PgStoreRepository::new(pool.clone()));
    let category_repo = Arc::new(PgCategoryRepository::new(pool.clone()));
    let product_repo = Arc::new(PgProductRepository::new(pool.clone()));
    let cart_repo = Arc::new(PgCartRepository::new(pool.clone()));
    let goods_order_repo = Arc::new(PgGoodsOrderRepository::new(pool.clone()));
    let runner_order_repo = Arc::new(PgRunnerOrderRepository::new(pool.clone()));
    let address_repo = Arc::new(PgAddressRepository::new(pool.clone()));
    let admin_repo = Arc::new(PgAdminRepository::new(pool.clone()));
    let tx_manager: Arc<dyn TransactionManager> = Arc::new(PgTransactionManager::new(pool));

    let wechat_auth: Arc<dyn WechatAuthClient> = Arc::new(WechatMiniProgramClient::new(
        config.wechat.app_id.clone(),
        config.wechat.app_secret.clone(),
        config.wechat.api_base.clone(),
        config.wechat.timeout_secs,
    )?);

    let cache_service: Arc<dyn CacheService> = match config.runtime.cache_provider {
        CacheProvider::Memory => Arc::new(MemoryCacheService::new()),
        CacheProvider::Redis => {
            Arc::new(RedisCacheService::new(&config.redis.url, config.redis.max_connections).await?)
        }
    };

    let sms_gateway: Arc<dyn SmsGateway> = Arc::new(LogSmsGateway);
    let lbs_service: Arc<dyn LbsService> = match config.runtime.lbs_provider {
        LbsProvider::Noop => Arc::new(NoopLbs),
    };

    let user_service = UserService::new(user_repo).with_auth(
        wechat_auth,
        cache_service,
        sms_gateway,
        config.sms.login_code_ttl_secs,
    );
    let admin_service = AdminService::new(admin_repo);
    let store_service = StoreService::new(store_repo.clone(), lbs_service);
    let category_service = CategoryService::new(category_repo);
    let product_service = ProductService::new(product_repo.clone());
    let cart_service = CartService::new(cart_repo);
    let order_service =
        OrderService::new(goods_order_repo, product_repo.clone(), store_repo.clone())
            .with_transaction_manager(tx_manager.clone());
    let runner_order_service =
        RunnerOrderService::new(runner_order_repo, store_repo).with_transaction_manager(tx_manager);
    let address_service = AddressService::new(address_repo);

    Ok(AppState::new(
        user_service,
        admin_service,
        store_service,
        category_service,
        product_service,
        cart_service,
        config.auth.jwt_secret.clone(),
        config.auth.jwt_ttl_secs,
        config.sms.login_code_ttl_secs,
    )
    .with_address_service(address_service)
    .with_order_services(order_service, runner_order_service))
}

struct NoopLbs;

#[async_trait]
impl LbsService for NoopLbs {
    async fn distance_km(
        &self,
        _from: (f64, f64),
        _to: (f64, f64),
    ) -> axum_core_kernel::AppResult<f64> {
        Ok(0.0)
    }
}
