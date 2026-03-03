use std::sync::Arc;

use axum_application::{
    AddressService, AdminService, CartService, CategoryService, OrderService, ProductService,
    RunnerOrderService, StoreService, UserService,
};

#[derive(Clone)]
pub struct BizConfig {
    pub delivery_free_radius_km: f64,
    pub runner_service_fee: i32,
    pub customer_service_phone: String,
    pub runner_banner_enabled: bool,
    pub runner_banner_text: String,
    pub pay_timeout_secs: u64,
    pub auto_accept_secs: u64,
}

impl Default for BizConfig {
    fn default() -> Self {
        Self {
            delivery_free_radius_km: 3.0,
            runner_service_fee: 200,
            customer_service_phone: "400-000-0000".into(),
            runner_banner_enabled: true,
            runner_banner_text: "顺路代取快递".into(),
            pay_timeout_secs: 15 * 60,
            auto_accept_secs: 5 * 60,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub admin_service: Arc<AdminService>,
    pub store_service: Arc<StoreService>,
    pub category_service: Arc<CategoryService>,
    pub product_service: Arc<ProductService>,
    pub cart_service: Arc<CartService>,
    pub address_service: Option<Arc<AddressService>>,
    pub order_service: Option<Arc<OrderService>>,
    pub runner_order_service: Option<Arc<RunnerOrderService>>,
    pub jwt_secret: String,
    pub jwt_ttl_secs: u64,
    pub sms_code_ttl_secs: u64,
    pub biz_config: BizConfig,
}

impl AppState {
    pub fn new(
        user_service: UserService,
        admin_service: AdminService,
        store_service: StoreService,
        category_service: CategoryService,
        product_service: ProductService,
        cart_service: CartService,
        jwt_secret: String,
        jwt_ttl_secs: u64,
        sms_code_ttl_secs: u64,
    ) -> Self {
        Self {
            user_service: Arc::new(user_service),
            admin_service: Arc::new(admin_service),
            store_service: Arc::new(store_service),
            category_service: Arc::new(category_service),
            product_service: Arc::new(product_service),
            cart_service: Arc::new(cart_service),
            address_service: None,
            order_service: None,
            runner_order_service: None,
            jwt_secret,
            jwt_ttl_secs,
            sms_code_ttl_secs,
            biz_config: BizConfig::default(),
        }
    }

    pub fn with_order_services(
        mut self,
        order_service: OrderService,
        runner_order_service: RunnerOrderService,
    ) -> Self {
        self.order_service = Some(Arc::new(order_service));
        self.runner_order_service = Some(Arc::new(runner_order_service));
        self
    }

    pub fn with_address_service(mut self, address_service: AddressService) -> Self {
        self.address_service = Some(Arc::new(address_service));
        self
    }

    pub fn with_biz_config(mut self, biz_config: BizConfig) -> Self {
        self.biz_config = biz_config;
        self
    }
}
