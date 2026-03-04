use utoipa::OpenApi;

use crate::handlers::address_handler;

#[derive(OpenApi)]
#[openapi(paths(address_handler::list_addresses))]
pub struct PublicApiDoc;
