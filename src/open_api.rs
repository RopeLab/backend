use axum::{Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::Pool;
use crate::auth::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        openapi,
        sign_up,
        login,
        list_users,
    ), 
    components(schemas(
        User,
        Credentials
    )))]
struct ApiDoc;

/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
pub async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn add_swagger_route(router: Router<Pool>) -> Router<Pool> {
    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}