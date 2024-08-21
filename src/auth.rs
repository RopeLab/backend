use std::convert::Infallible;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use async_trait::async_trait;
use axum::{Json, Router};
use axum::response::{IntoResponse};
use axum::routing::{get, post};
use axum_login::{AuthUser, AuthnBackend, UserId};
use diesel::{ExpressionMethods, Insertable, Queryable, QueryDsl, Selectable, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use utoipa::ToSchema;
use crate::{internal_error, internal_error_from_string, Pool};
use crate::db_backend::DatabaseConnection;
use crate::schema::users;

#[derive(serde::Serialize, Selectable, Queryable, ToSchema, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub pw_hash: String,
    pub salt: String,
}


#[derive(serde::Deserialize, Insertable, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub pw_hash: String,
    pub salt: String,
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

#[derive(Clone)]
pub struct AuthBackend {
    pub pool: Pool
}

#[derive(serde::Deserialize, Clone, ToSchema)]
pub struct Credentials {
    email: String,
    password: String,
}

#[utoipa::path(
    post,
    path = "/signup"
)]
async fn sign_up(
    mut conn: DatabaseConnection,
    Json(credentials): Json<Credentials>
) -> Result<(), (StatusCode, String)> {

    if get_user_with_email(&mut conn, &credentials.email).await.is_some() {
        return Err(internal_error_from_string("email allready used"));
    }
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pw_hash = argon2.hash_password(credentials.password.as_bytes(), &salt).unwrap().to_string();

    let new_user = NewUser{
        email: credentials.email,
        pw_hash,
        salt: "".to_owned(),
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .execute(&mut conn.0)
        .await
        .map_err(internal_error)?;
    
    Ok(())
}

async fn get_user_with_email(conn: &mut DatabaseConnection, email: &str) -> Option<User> {
    users::table
        .filter(users::columns::email.eq(email))
        .select(User::as_select())
        .first(&mut conn.0)
        .await
        .ok()
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = Infallible;

    async fn authenticate(&self, credentials: Credentials) -> Result<Option<Self::User>, Self::Error> {
        let mut conn = DatabaseConnection::from_pool(&self.pool).await.unwrap();
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
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut conn = self.pool
            .get()
            .await
            .map_err(internal_error)
            .unwrap();

        let res = users::table
            .find(user_id)
            .first(&mut conn)
            .await
            .ok();

        Ok(res)
    }
}

type AuthSession = axum_login::AuthSession<AuthBackend>;

#[utoipa::path(
    post,
    path = "/login"
)]
async fn login(
    mut auth_session: AuthSession,
    Json(credentials): Json<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(credentials.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}

#[utoipa::path(
    get,
    path = "/user/list"
)]
async fn list_users(DatabaseConnection(mut conn): DatabaseConnection) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let res = users::table
        .select(User::as_select())
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub fn add_login_auth_routes(router: Router<Pool>) -> Router<Pool> {
    router.route("/user/list", get(list_users))
}

pub fn add_auth_routes(router: Router<Pool>) -> Router<Pool> {
    router.route("/signup", post(sign_up))
        .route("/login", post(login))
}