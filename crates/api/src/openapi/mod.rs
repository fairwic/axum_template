mod public;

use utoipa::OpenApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    public::PublicApiDoc::openapi()
}
