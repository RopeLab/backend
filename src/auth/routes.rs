use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use axum_login::{AuthnBackend, UserId};
use diesel::{Insertable, Queryable, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use diesel::ExpressionMethods;
use crate::auth::{AuthSession, Credentials, get_user_with_email, NewUser, User};
use crate::auth::util::{get_logged_in_id, path_id_is_admin_or_me};
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError, APIResult};
use crate::schema::users;
use crate::schema::users::{email, id};

#[utoipa::path(
    post,
    path = "/signup"
)]
async fn sign_up(
    mut conn: DBConnection,
    Json(credentials): Json<Credentials>
) -> APIResult<()> {

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
) -> APIResult<Json<UserId<Backend>>> {
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
async fn logout(mut auth_session: AuthSession) -> APIResult<()> {
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
    path = "/user/id"
)]
#[debug_handler]
pub async fn get_id(
    auth_session: AuthSession,
) -> APIResult<Json<UserId<Backend>>> {
    let user_id = get_logged_in_id(auth_session).await?;
    Ok(Json(user_id))
}

#[utoipa::path(
    get,
    path = "/user/{id}/email"
)]
#[debug_handler]
pub async fn get_email(
    auth_session: AuthSession,
    path: Path<String>,
) -> crate::error::APIResult<Json<String>> {
    let (u_id, mut conn) = path_id_is_admin_or_me(auth_session, path).await?;
    let mail = users::table
        .filter(id.eq(u_id))
        .select(email)
        .get_result(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    Ok(Json(mail))
}

#[utoipa::path(
    get,
    path = "/user/all"
)]
async fn get_all_users(DBConnection(mut conn): DBConnection) -> crate::error::APIResult<Json<Vec<User>>> {
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
        .route("/logout", post(logout))
        .route("/user/id", get(get_id))
        .route("/user/:id/email", get(get_email))
}

pub fn add_admin_auth_routes(router: Router<Backend>) -> Router<Backend> {
    router.route("/user/all", get(get_all_users))
}
