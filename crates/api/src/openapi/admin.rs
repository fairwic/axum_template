use utoipa::OpenApi;

use crate::handlers::{
    admin_auth_handler, admin_category_handler, admin_order_handler, admin_product_handler,
    admin_runner_order_handler, admin_store_handler, config_handler,
};

#[derive(OpenApi)]
#[openapi(paths(
    admin_auth_handler::admin_login,
    config_handler::admin_get_config,
    config_handler::admin_update_config,
    admin_store_handler::admin_list_stores,
    admin_store_handler::admin_create_store,
    admin_store_handler::admin_update_store,
    admin_category_handler::admin_create_category,
    admin_category_handler::admin_update_category,
    admin_product_handler::admin_create_product,
    admin_product_handler::admin_update_product,
    admin_order_handler::admin_list_orders,
    admin_order_handler::admin_accept_order,
    admin_order_handler::admin_dispatch_order,
    admin_order_handler::admin_complete_order,
    admin_runner_order_handler::admin_list_runner_orders,
    admin_runner_order_handler::admin_accept_runner_order,
    admin_runner_order_handler::admin_delivered_runner_order,
    admin_runner_order_handler::admin_complete_runner_order,
))]
pub struct AdminApiDoc;
