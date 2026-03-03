//! API Router Configuration

use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use crate::handlers::health_handler;
use crate::openapi::ApiDoc;
use crate::routes::{
    address, admin_auth, admin_category, admin_order, admin_product, admin_runner_order,
    admin_store, auth, cart, category, member, order, product, runner_order, store,
};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::<AppState>::new()
        .merge(auth::routes())
        .merge(member::routes())
        .merge(address::routes())
        .merge(store::routes())
        .merge(cart::routes())
        .merge(order::routes())
        .merge(runner_order::routes())
        .merge(category::routes())
        .merge(product::routes());
    let admin_routes = Router::<AppState>::new()
        .merge(admin_auth::routes())
        .merge(admin_store::routes())
        .merge(admin_category::routes())
        .merge(admin_product::routes())
        .merge(admin_order::routes())
        .merge(admin_runner_order::routes());

    let openapi_route = Router::<AppState>::new().route(
        "/api-docs/openapi.json",
        get(|| async { Json(ApiDoc::openapi()) }),
    );

    let swagger_ui_route = Router::<AppState>::new().route(
        "/swagger-ui",
        get(|| async {
            axum::response::Html(
                r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="SwaggerUI" />
    <title>SwaggerUI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js" crossorigin></script>
<script>
    window.onload = () => {
    window.ui = SwaggerUIBundle({
        url: '/api-docs/openapi.json',
        dom_id: '#swagger-ui',
    });
    };
</script>
</body>
</html>
"#,
            )
        }),
    );

    Router::<AppState>::new()
        .merge(openapi_route)
        .merge(swagger_ui_route)
        .route("/health", get(health_handler::health_check))
        .nest("/api/v1", api_routes)
        .nest("/api/admin/v1", admin_routes)
        .with_state(state)
}
