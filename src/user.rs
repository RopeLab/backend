use axum::extract::State;
use axum::{Json, Router};
use axum::routing::{get, post};
use diesel::{Insertable, Queryable, QueryDsl, Selectable, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use utoipa::ToSchema;
use crate::{DatabaseConnection, internal_error, Pool};
use crate::schema::users;

#[derive(serde::Serialize, Selectable, Queryable, ToSchema)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(serde::Deserialize, Insertable, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

#[utoipa::path(
    post,
    path = "/user/create"
)]
async fn create_user(
    State(pool): State<Pool>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;

    let res = diesel::insert_into(users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}


#[utoipa::path(
    get,
    path = "/user/list"
)]
async fn list_users(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let res = users::table
        .select(User::as_select())
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub fn add_user_routes(router: Router<Pool>) -> Router<Pool> {
    router.route("/user/list", get(list_users))
        .route("/user/create", post(create_user))
}