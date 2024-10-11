use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use crate::auth::{AuthSession};
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError, APIResult};
use crate::permissions::{get_permissions_iter, has_permission, Permission, UserPermission};
use crate::schema::permission;
use crate::auth::util::auth_and_path_to_id_is_me_or_i_am_admin;

#[utoipa::path(
    post,
    path = "/permissions/{user_id}/add"
)]
pub async fn post_permission_add(
    mut conn: DBConnection,
    Path(u_id): Path<i32>,
    Json(permission): Json<UserPermission>
) -> APIResult<()> {

    if has_permission(&mut conn, u_id, permission).await {
        return Err(APIError::PermissionAlreadyAdded)
    }
    
    let user_permission = Permission {
        user_id: u_id,
        user_permission: permission,
    };

    diesel::insert_into(permission::table)
        .values(&user_permission)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/permissions/{user_id}/remove"
)]
pub async fn post_permission_remove(
    mut conn: DBConnection,
    Path(u_id): Path<i32>,
    Json(permission): Json<UserPermission>
) -> APIResult<()> {

    if !has_permission(&mut conn, u_id, permission).await {
        return Err(APIError::PermissionNotThere)
    }

    diesel::delete(permission::table)
        .filter(permission::user_id.eq(u_id))
        .filter(permission::user_permission.eq(permission))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/permissions/{user_id}/has"
)]
#[debug_handler]
pub async fn post_permission_has(
    auth_session: AuthSession,
    Path(u_id): Path<i32>,
    Json(permission): Json<UserPermission>
) -> APIResult<Json<bool>> {
    let mut conn = auth_and_path_to_id_is_me_or_i_am_admin(auth_session, u_id).await?;
    
    Ok(Json(has_permission(&mut conn, u_id, permission).await))
}

#[utoipa::path(
    get,
    path = "/permissions/{user_id}"
)]
#[debug_handler]
pub async fn get_permissions(
    auth_session: AuthSession,
    Path(u_id): Path<i32>,
) -> APIResult<Json<Vec<UserPermission>>> {
    let mut conn = auth_and_path_to_id_is_me_or_i_am_admin(auth_session, u_id).await?;

    let permissions = get_permissions_iter(&mut conn, u_id).await?.collect();
    Ok(Json(permissions))
}


pub fn add_admin_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permissions/:id/add", post(post_permission_add))
        .route("/permissions/:id/remove", post(post_permission_remove))
}

pub fn add_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permissions/:id/has", post(post_permission_has))
        .route("/permissions/:id", get(get_permissions))
}