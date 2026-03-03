mod admin;
mod public;
mod shared;

use utoipa::OpenApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = shared::SharedApiDoc::openapi();
    doc.merge(public::PublicApiDoc::openapi());
    doc.merge(admin::AdminApiDoc::openapi());
    doc
}
