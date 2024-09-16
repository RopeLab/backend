pub mod routes;

use axum::{async_trait};
use std::collections::{HashSet};
use axum_login::{AuthzBackend, UserId};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use utoipa::{ToSchema};
use crate::auth::ID;
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::schema::{permission};
use crate::schema::permission::{user_id, user_permission};
use crate::error::APIResult;

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
    pub id: ID,
    pub user_id: ID,
    pub user_permission: UserPermission,
}

#[derive(serde::Deserialize, Insertable, AsChangeset, ToSchema, Debug)]
#[diesel(table_name = permission)]
pub struct NewPermission {
    pub user_id: ID,
    pub user_permission: UserPermission,
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
) -> APIResult<impl Iterator<Item = UserPermission>> {
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

    async fn get_user_permissions(&self, user: &Self::User) -> APIResult<HashSet<Self::Permission>> {
        let mut conn = self.get_connection().await?;

        let permissions = get_permissions_iter(&mut conn, user.id).await?;
        Ok(HashSet::from_iter(permissions))
    }

    async fn get_all_permissions(&self, user: &Self::User) -> APIResult<HashSet<Self::Permission>> {
        self.get_user_permissions(user).await
    }

    async fn has_perm(&self, user: &Self::User, perm: Self::Permission) -> APIResult<bool> {
        let mut conn = self.get_connection().await?;
        Ok(has_permission(&mut conn, user.id, perm).await)
    }
}


