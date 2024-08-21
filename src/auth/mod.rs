pub mod routes;
pub mod util;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use axum_login::{AuthnBackend, AuthUser, AuthzBackend, UserId};
use diesel::{ExpressionMethods, Insertable, Queryable, QueryDsl, Selectable, SelectableHelper};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::schema::users;
use crate::error::Result;

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(serde::Serialize, Selectable, Queryable, ToSchema, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub pw_hash: String,
}


#[derive(serde::Deserialize, Insertable, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub pw_hash: String,
}

#[derive(serde::Deserialize, Clone, ToSchema)]
pub struct Credentials {
    email: String,
    password: String,
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash.as_bytes()
    }
}

async fn get_user_with_email(conn: &mut DBConnection, email: &str) -> Option<User> {
    users::table
        .filter(users::columns::email.eq(email))
        .select(User::as_select())
        .first(&mut conn.0)
        .await
        .ok()
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = APIError;

    async fn authenticate(&self, credentials: Credentials) -> Result<Option<User>> {
        let mut conn = self.get_connection().await?;
        let user = get_user_with_email(&mut conn, &credentials.email).await;
        if user.is_none() { return Ok(None) }
        let user = user.unwrap();
        
        let parsed_hash = PasswordHash::new(&user.pw_hash).unwrap();
        return if Argon2::default().verify_password(credentials.password.as_bytes(), &parsed_hash).is_ok() {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> Result<Option<Self::User>> {
        let mut conn = self.db_pool
            .get()
            .await
            .map_err(APIError::internal)
            .unwrap();

        let res = users::table
            .find(user_id)
            .first(&mut conn)
            .await
            .ok();

        Ok(res)
    }
}






