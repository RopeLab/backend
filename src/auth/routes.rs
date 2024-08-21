use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::{debug_handler, Json, Router};
use axum::routing::{get, post};
use axum_login::{AuthnBackend, UserId};
use diesel::{Insertable, Queryable, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;
use crate::auth::{AuthSession, Credentials, get_user_with_email, NewUser, User};
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError};
use crate::schema::users;


#[utoipa::path(
    post,
    path = "/signup"
)]
async fn sign_up(
    mut conn: DBConnection,
    Json(credentials): Json<Credentials>
) -> crate::error::Result<()> {

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

#[utoipa::path(
    post,
    path = "/login"
)]
#[debug_handler]
async fn login(
    mut auth_session: AuthSession,
    Json(credentials): Json<Credentials>,
) -> crate::error::Result<Json<UserId<Backend>>> {
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
async fn logout(mut auth_session: AuthSession) -> crate::error::Result<()> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }

    auth_session.logout()
        .await
        .map_err(APIError::internal)?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/user/list"
)]
async fn list_users(DBConnection(mut conn): DBConnection) -> crate::error::Result<Json<Vec<User>>> {
    let res = users::table
        .select(User::as_select())
        .load(&mut conn)
        .await
        .map_err(APIError::internal)?;
    Ok(Json(res))
}

pub fn add_auth_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/signup", post(sign_up))
        .route("/login", post(login))
        .route("/logout", post(login))
}

pub fn add_admin_auth_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user/list", get(list_users))
}
