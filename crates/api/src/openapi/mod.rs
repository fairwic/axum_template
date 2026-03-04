mod public;

use utoipa::OpenApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = public::PublicApiDoc::openapi();
    doc
}
