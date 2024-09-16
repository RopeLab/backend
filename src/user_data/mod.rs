
use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use axum_login::UserId;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::{AuthSession, ID};
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::error::APIResult;
use crate::schema::user_data::user_id;
use crate::schema::user_data;
use crate::auth::util::{auth_and_path_to_id_is_me_or_i_am_admin, auth_to_id_is_me_or_i_am_admin};

#[derive(serde::Deserialize, Insertable, AsChangeset, ToSchema, Debug, serde::Serialize, Queryable, Selectable, PartialEq)]
#[diesel(table_name = user_data)]
pub struct UserData {
    pub user_id: ID,
    pub name: String,
    pub fetlife_name: String,
    pub experience_text: String,
    pub found_us_text: String,
    pub goal_text: String,
    pub role_factor: f64,
    pub open: bool,
    pub show_name: bool,
    pub show_role: bool,
    pub show_open: bool,
    pub show_fetlife: bool,
    pub new: bool,
}

#[utoipa::path(
    post,
    path = "/user_data"
)]
#[debug_handler]
pub async fn post_user_data(
    auth_session: AuthSession,
    Json(new_user_data): Json<UserData>
) -> APIResult<()> {
    let mut conn = auth_to_id_is_me_or_i_am_admin(auth_session, new_user_data.user_id).await?;

    diesel::insert_into(user_data::table)
        .values(&new_user_data)
        .on_conflict(user_id)
        .do_update()
        .set(&new_user_data)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/user_data/{id}"
)]
#[debug_handler]
pub async fn get_user_data(
    auth_session: AuthSession,
    Path(u_id): Path<ID>,
) -> APIResult<Json<UserData>> {
    let mut conn = auth_and_path_to_id_is_me_or_i_am_admin(auth_session, u_id).await?;
    let user_data = get_user_data_by_id(&mut conn, u_id).await?;
    Ok(Json(user_data))
}

pub async fn get_user_data_by_id(
    conn: &mut DBConnection, 
    id: UserId<Backend>
) -> APIResult<UserData> {
    let user_data = user_data::table
        .filter(user_id.eq(id))
        .select(UserData::as_select())
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    
    Ok(user_data)
}

#[utoipa::path(
    get,
    path = "/user_data/all"
)]
pub async fn get_user_data_all(DBConnection(mut conn): DBConnection) -> APIResult<Json<Vec<UserData>>> {
    let user_data = user_data::table
        .select(UserData::as_select())
        .get_results(&mut conn)
        .await
        .map_err(APIError::internal)?;

    Ok(Json(user_data))
}

pub fn add_admin_user_data_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user_data/all", get(get_user_data_all))
}


pub fn add_user_data_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user_data/:id", get(get_user_data))
        .route("/user_data", post(post_user_data))
}