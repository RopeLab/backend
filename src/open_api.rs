use crate::events::event_user::EventUser;
use axum::{Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::backend::Backend;
use crate::auth::*;
use crate::auth::routes::*;
use crate::user_data::*;
use crate::permissions::*;
use crate::permissions::routes::*;
use crate::events::*;
use crate::events::event_user::*;
use crate::events::user_action::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        openapi,
        sign_up,
        login,
        logout,
        get_email,
        get_all_users,
        get_user_data,
        post_user_data,
        get_user_data_all,
        post_permission,
        get_permission,
        post_event,
        update_event,
        delete_event,
        get_event,
        get_event_all,
        get_event_users,
        register_to_event,
        unregister_from_event,
        change_guests,
        get_user_actions
    ), 
    components(schemas(
        User,
        Credentials,
        UserData,
        Permission,
        NewPermission,
        Event,
        NewEvent,
        EventUser,
        PublicEventUser,
        UserAndGuests,
        UserAction,
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