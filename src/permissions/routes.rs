use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use diesel_async::RunQueryDsl;
use crate::auth::{AuthSession, ID};
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::permissions::{get_permissions_iter, has_permission, NewPermission, UserPermission};
use crate::schema::permission;
use crate::auth::util::auth_and_path_to_id_is_me_or_i_am_admin;

#[utoipa::path(
    post,
    path = "/permission"
)]
pub async fn post_permission(
    mut conn: DBConnection,
    Json(new_permission): Json<NewPermission>
) -> crate::error::APIResult<()> {

    if has_permission(&mut conn, new_permission.user_id, new_permission.user_permission).await {
        return Err(APIError::PermissionAlreadyAdded)
    }

    diesel::insert_into(permission::table)
        .values(&new_permission)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/permission/{id}"
)]
#[debug_handler]
pub async fn get_permission(
    auth_session: AuthSession,
    Path(u_id): Path<ID>,
) -> crate::error::APIResult<Json<Vec<UserPermission>>> {
    let mut conn = auth_and_path_to_id_is_me_or_i_am_admin(auth_session, u_id).await?;

    let permissions = get_permissions_iter(&mut conn, u_id).await?.collect();
    Ok(Json(permissions))
}


pub fn add_admin_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permission", post(post_permission))
}

pub fn add_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permission/:id", get(get_permission))
}