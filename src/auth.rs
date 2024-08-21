
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use async_trait::async_trait;
use axum::{debug_handler, Json, Router};
use axum::routing::{get, post};
use axum_login::{AuthUser, AuthnBackend, UserId, AuthzBackend};
use diesel::{ExpressionMethods, Insertable, Queryable, QueryDsl, Selectable, SelectableHelper};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError};
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


#[utoipa::path(
    post,
    path = "/signup"
)]
async fn sign_up(
    mut conn: DBConnection,
    Json(credentials): Json<Credentials>
) -> Result<()> {

    if get_user_with_email(&mut conn, &credentials.email).await.is_some() {
        return Err(APIError::EmailUsed);
    }
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pw_hash = argon2.hash_password(credentials.password.as_bytes(), &salt).unwrap().to_string();

    let new_user = NewUser{
        email: credentials.email,
        pw_hash,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    
    Ok(())
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



#[utoipa::path(
    post,
    path = "/login"
)]
#[debug_handler]
async fn login(
    mut auth_session: AuthSession,
    Json(credentials): Json<Credentials>,
) -> Result<Json<UserId<Backend>>> {
    let user = match auth_session.authenticate(credentials.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(APIError::InvalidCredentials),
        Err(err) => return Err(APIError::internal(err)),
    };

    if let Err(err) = auth_session.login(&user).await {
        return Err(APIError::internal(err));
    }

    Ok(Json(auth_session.user.unwrap().id))
}

#[utoipa::path(
    post,
    path = "/logout"
)]
#[debug_handler]
async fn logout(mut auth_session: AuthSession) -> Result<()> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }

    auth_session.logout()
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

pub fn add_auth_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/signup", post(sign_up))
        .route("/login", post(login))
        .route("/logout", post(login))
}

#[utoipa::path(
    get,
    path = "/user/list"
)]
async fn list_users(DBConnection(mut conn): DBConnection) -> Result<Json<Vec<User>>> {
    let res = users::table
        .select(User::as_select())
        .load(&mut conn)
        .await
        .map_err(APIError::internal)?;
    Ok(Json(res))
}

pub fn add_admin_auth_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user/list", get(list_users))
}

