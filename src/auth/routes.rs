use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::{debug_handler, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use axum_login::UserId;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use diesel::ExpressionMethods;
use crate::auth::{AuthSession, Credentials, get_user_with_email, NewUser, User};
use crate::auth::util::{auth_to_logged_in_id, auth_and_path_to_id_is_me_or_i_am_admin};
use crate::backend::{Backend, DBConnection};
use crate::error::{APIError, APIResult};
use crate::events::slots::after_unregister;
use crate::events::users::{EventUser};
use crate::firebase::{firebase_get_user_data, firebase_is_user_new, firebase_is_user_verified, firebase_login_user, insert_user_data_from_firebase};
use crate::permissions::{is_admin, is_check_attended, is_verified, UserPermission};
use crate::permissions::routes::post_permission_add;
use crate::schema::{event_user, permission, user_action, user_data, users};
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
        Ok(None) => {
            if let Ok(firebase_id) = firebase_login_user(credentials.clone()).await {
                let firebase_user_data = firebase_get_user_data(&firebase_id).await?;
                let firebase_user_new = firebase_is_user_new(&firebase_id).await?;
                let firebase_verified = firebase_is_user_verified(&firebase_id).await?;

                let conn = auth_session.backend.get_connection().await?;
                sign_up(conn, Json(credentials.clone())).await?;
                let user = auth_session.authenticate(credentials.clone()).await
                    .map_err(APIError::internal)?;
                if user.is_none() {
                    return Err(APIError::Internal("Could not login user in firebase bridge".to_string()));
                }
                let user = user.unwrap();

                let mut conn = auth_session.backend.get_connection().await?;
                insert_user_data_from_firebase(user.id, firebase_user_data, firebase_user_new, &mut conn).await?;

                if firebase_verified {
                    post_permission_add(conn, Path(user.id), Json(UserPermission::Verified)).await?;
                }

                user
            } else {
                return Err(APIError::InvalidCredentials);
            }
        },
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
    let user_id = auth_to_logged_in_id(auth_session).await?;
    Ok(Json(user_id))
}

#[utoipa::path(
    get,
    path = "/user/{id}/email"
)]
#[debug_handler]
pub async fn get_email(
    auth_session: AuthSession,
    Path(u_id): Path<i32>,
) -> APIResult<Json<String>> {
    let mut conn = auth_and_path_to_id_is_me_or_i_am_admin(auth_session, u_id).await?;
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

#[utoipa::path(
    post,
    path = "/user/{id}/remove"
)]
async fn remove_user(
    mut conn: DBConnection,
    Path(u_id): Path<i32>
) -> APIResult<()> {
    diesel::delete(user_action::table)
        .filter(user_action::user_id.eq(u_id))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    let removed_event_users = diesel::delete(event_user::table)
        .filter(event_user::user_id.eq(u_id))
        .returning(EventUser::as_select())
        .get_results::<EventUser>(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    for removed_event_user in removed_event_users {
        after_unregister(removed_event_user, &mut conn).await?
    }

    diesel::delete(permission::table)
        .filter(permission::user_id.eq(u_id))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    diesel::delete(user_data::table)
        .filter(user_data::user_id.eq(u_id))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    diesel::delete(users::table)
        .filter(users::id.eq(u_id))
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;

    Ok(())
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
        .route( "/user/:id/remove", post(remove_user))
}
