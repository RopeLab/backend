use axum::{Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::backend::Backend;
use crate::auth::*;
use crate::user_data::*;
use crate::permissions::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        openapi,
        sign_up,
        login,
        logout,
        list_users,
        get_user_data,
        post_user_data,
        post_permission,
        get_permission
    ), 
    components(schemas(
        User,
        Credentials,
        UserData,
        NewUserData,
        Permission,
        NewPermission,
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

pub fn add_swagger_route(router: Router<Backend>) -> Router<Backend> {
    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}