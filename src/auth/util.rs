use axum::extract::Path;
use axum_login::UserId;
use crate::auth::AuthSession;
use crate::backend::{Backend, DBConnection};
use crate::error::APIError;
use crate::permissions::is_admin;
use crate::error::Result;

pub async fn is_logged_in(
    auth_session: AuthSession,
) -> Result<DBConnection> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    
    let conn = auth_session.backend.get_connection().await?;
    Ok(conn)
}

pub async fn id_is_admin_or_me(
    auth_session: AuthSession,
    id: UserId<Backend>,
) -> Result<(UserId<Backend>, DBConnection)> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let user = auth_session.user.unwrap();
    
    let mut conn = auth_session.backend.get_connection().await?;
    if user.id != id && !is_admin(&mut conn, user.id).await {
        return Err(APIError::UNAUTHORIZED);
    }

    Ok((id, conn))
}


pub async fn path_id_is_admin_or_me(
    auth_session: AuthSession,
    path: Path<String>,
) -> Result<(UserId<Backend>, DBConnection)> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let user = auth_session.user.unwrap();

    let id = parse_path_id(path)?;
    let mut conn = auth_session.backend.get_connection().await?;
    if user.id != id && !is_admin(&mut conn, user.id).await {
        return Err(APIError::UNAUTHORIZED);
    }
    
    Ok((id, conn))
}

pub fn parse_path_id(Path(id): Path<String>) -> Result<i32> {
    let id = id.parse::<i32>();
    if id.is_err() {
        return Err(APIError::InvalidPath);
    }
    
    Ok(id.unwrap())
}

