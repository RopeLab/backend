use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use diesel::QueryDsl;
use utoipa::ToSchema;
use crate::auth::AuthSession;
use crate::user_data::{get_user_data_by_id, post_user_data, UserData};
use crate::auth::util::{is_logged_in, parse_path_id};
use crate::backend::Backend;
use crate::error::Result;

#[derive(serde::Serialize, ToSchema, Debug, PartialEq)]
pub struct PublicUserData {
    pub user_id: i32,
    pub name: Option<String>,
    pub role_factor: Option<f64>,
    pub open: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/public_user_data/{id}"
)]
#[debug_handler]
pub async fn get_public_user_data(
    auth_session: AuthSession,
    path: Path<String>,
) -> Result<Json<PublicUserData>> {
    let mut conn = is_logged_in(auth_session).await?;
    let id = parse_path_id(path)?;

    let user_data = get_user_data_by_id(&mut conn, id).await?;
    Ok(Json(user_data.into()))
}

impl From<UserData> for PublicUserData {
    fn from(user_data: UserData) -> Self {
        PublicUserData {
            user_id: user_data.user_id,
            name: if user_data.show_name {Some(user_data.name)} else {None},
            role_factor: if user_data.show_role {Some(user_data.role_factor)} else {None},
            open: if user_data.show_open {Some(user_data.open)} else {None},
        }
    }
}

pub fn add_public_user_data_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/public_user_data/:id", get(get_public_user_data))
}