use utoipa::OpenApi;

use crate::handlers::{
    address_handler, auth_handler, cart_handler, category_handler, config_handler, member_handler,
    order_handler, product_handler, runner_order_handler, store_handler,
};

#[derive(OpenApi)]
#[openapi(paths(
    auth_handler::wechat_login,
    auth_handler::send_sms_code,
    auth_handler::phone_sms_login,
    config_handler::get_config,
    address_handler::list_addresses,
    address_handler::create_address,
    address_handler::update_address,
    address_handler::delete_address,
    address_handler::set_default_address,
    member_handler::member_status,
    member_handler::member_benefits,
    store_handler::nearby_stores,
    store_handler::select_store,
    store_handler::current_store,
    cart_handler::get_cart,
    cart_handler::add_item,
    cart_handler::update_qty,
    cart_handler::remove_item,
    cart_handler::clear_cart,
    order_handler::create_order,
    order_handler::preview_order,
    order_handler::pay_order,
    order_handler::list_orders,
    order_handler::get_order,
    order_handler::cancel_order,
    order_handler::repurchase_order,
    runner_order_handler::create_runner_order,
    runner_order_handler::pay_runner_order,
    runner_order_handler::list_runner_orders,
    runner_order_handler::get_runner_order,
    runner_order_handler::cancel_runner_order,
    category_handler::list_categories,
    product_handler::list_products,
    product_handler::search_products,
    product_handler::get_product,
))]
pub struct PublicApiDoc;
