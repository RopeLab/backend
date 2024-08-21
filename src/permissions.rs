use axum::{async_trait, debug_handler, Json, Router};
use std::collections::{BTreeMap, HashSet};
use std::ops::BitAnd;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_login::{AuthzBackend, UserId};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info};
use tracing::log::log;
use utoipa::{ToSchema};
use utoipa::openapi::{RefOr, Response};
use crate::auth::AuthSession;
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::schema::{permission};
use crate::schema::permission::{user_id, user_permission};
use crate::error::Result;
use crate::util::path_id_is_admin_or_me;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Userpermission"]
#[repr(u8)]
pub enum UserPermission {
    Admin,
    Verified
}

#[derive(serde::Serialize, Queryable, Selectable, ToSchema, Debug)]
#[diesel(table_name = permission)]
pub struct Permission {
    pub id: i32,
    pub user_id: i32,
    pub user_permission: UserPermission,
}

#[derive(serde::Deserialize, Insertable, AsChangeset, ToSchema, Debug)]
#[diesel(table_name = permission)]
pub struct NewPermission {
    pub user_id: i32,
    pub user_permission: UserPermission,
}

#[utoipa::path(
    post,
    path = "/permission"
)]
pub async fn post_permission(
    mut conn: DBConnection,
    Json(new_permission): Json<NewPermission>
) -> Result<()> {

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
    path: Path<String>,
) -> Result<Json<Vec<UserPermission>>> {
    let (id, mut conn) = path_id_is_admin_or_me(auth_session, path).await?;
    
    let permissions = get_permissions_iter(&mut conn, id).await?.collect();
    Ok(Json(permissions))
}

pub async fn has_permission(
    conn: &mut DBConnection,
    id: UserId<Backend>,
    perm: UserPermission,
) -> bool {
    let found = permission::table
        .filter(user_id.eq(id))
        .filter(user_permission.eq(perm))
        .select(Permission::as_select())
        .get_result(&mut conn.0)
        .await
        .is_ok();

    found
}

pub async fn is_admin(
    conn: &mut DBConnection,
    id: UserId<Backend>,
) -> bool {
    has_permission(conn, id, UserPermission::Admin).await
}

pub async fn get_permissions_iter(
    conn: &mut DBConnection,
    id: UserId<Backend>,
) -> Result<impl Iterator<Item = UserPermission>> {
    let permissions = permission::table
        .filter(user_id.eq(id))
        .select(Permission::as_select())
        .get_results(&mut conn.0)
        .await
        .map_err(APIError::internal)?
        .into_iter()
        .map(|p| {p.user_permission});

    Ok(permissions)
}

#[async_trait]
impl AuthzBackend for Backend {
    type Permission = UserPermission;

    async fn get_user_permissions(&self, user: &Self::User) -> Result<HashSet<Self::Permission>> {
        let mut conn = self.get_connection().await?;

        let permissions = get_permissions_iter(&mut conn, user.id).await?;
        Ok(HashSet::from_iter(permissions))
    }

    async fn get_all_permissions(&self, user: &Self::User) -> Result<HashSet<Self::Permission>> {
        self.get_user_permissions(user).await
    }

    async fn has_perm(&self, user: &Self::User, perm: Self::Permission) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        Ok(has_permission(&mut conn, user.id, perm).await)
    }
}

pub fn add_admin_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permission", post(post_permission))
}

pub fn add_permission_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/permission/:id", get(get_permission))
}
