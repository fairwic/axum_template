mod public;

use utoipa::OpenApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let doc = public::PublicApiDoc::openapi();
    doc
}
