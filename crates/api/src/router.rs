//! API Router Configuration

use axum::{middleware, routing::get, Json, Router};

use crate::auth::middleware::require_user_auth;
use crate::handlers::health_handler;
use crate::openapi::openapi;
use crate::routes::address;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let public_api_routes = Router::<AppState>::new().merge(address::routes());
    let protected_api_routes = Router::<AppState>::new()
        .merge(address::routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_user_auth,
        ));
    let api_routes = Router::<AppState>::new()
        .merge(public_api_routes)
        .merge(protected_api_routes);

    let openapi_route = Router::<AppState>::new()
        .route("/api-docs/openapi.json", get(|| async { Json(openapi()) }));

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
        .with_state(state)
}
