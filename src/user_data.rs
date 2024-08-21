use axum::{debug_handler, Json, Router};
use axum::routing::get;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::{AuthSession};
use crate::backend::Backend;
use crate::error::APIError;
use crate::error::Result;
use crate::schema::user_data::user_id;
use crate::schema::user_data;

#[derive(serde::Serialize, Queryable, Selectable, Debug, PartialEq, ToSchema)]
#[diesel(table_name = user_data)]
pub struct UserData {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub fetlife_name: String,
    pub experience_text: String,
    pub found_us_text: String,
    pub goal_text: String,
    pub role_factor: f64,
    pub active_factor: f64,
    pub passive_factor: f64,
    pub open: bool,
    pub show_name: bool,
    pub show_role: bool,
    pub show_experience: bool,
    pub show_open: bool,
}

#[utoipa::path(
    get,
    path = "/user_data/me"
)]
#[debug_handler]
pub async fn user_data(auth_session: AuthSession) -> Result<Json<UserData>> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let mut conn = auth_session.backend.get_connection().await?;

    let user = auth_session.user.unwrap();
    
    let user_data = user_data::table
        .filter(user_id.eq(user.id))
        .select(UserData::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(Json(user_data))
}

pub fn add_user_data_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user_data/me", get(user_data))
}