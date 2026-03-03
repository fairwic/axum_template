//! API Router Configuration

use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use crate::handlers::health_handler;
use crate::openapi::ApiDoc;
use crate::routes::{admin_auth, auth, member};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .merge(auth::routes())
        .merge(member::routes());
    let admin_routes = Router::new().merge(admin_auth::routes());

    let openapi_route = Router::new().route(
        "/api-docs/openapi.json",
        get(|| async { Json(ApiDoc::openapi()) }),
    );

    let swagger_ui_route = Router::new().route(
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

    Router::new()
        .merge(openapi_route)
        .merge(swagger_ui_route)
        .route("/health", get(health_handler::health_check))
        .nest("/api/v1", api_routes)
        .nest("/api/admin/v1", admin_routes)
        .with_state(state)
}
