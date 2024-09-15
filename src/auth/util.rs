use crate::auth::{AuthSession, ID};
use crate::backend::{DBConnection};
use crate::error::APIError;
use crate::permissions::is_admin;
use crate::error::APIResult;

pub async fn auth_to_conn_expect_logged_in(
    auth_session: AuthSession,
) -> APIResult<DBConnection> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    
    let conn = auth_session.backend.get_connection().await?;
    Ok(conn)
}

pub async fn auth_to_conn_expect_logged_in_check_is_admin(
    auth_session: AuthSession,
) -> APIResult<(bool, DBConnection)> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let user = auth_session.user.unwrap();

    let mut conn = auth_session.backend.get_connection().await?;
    let admin = is_admin(&mut conn, user.id).await;
    Ok((admin, conn))
}

pub async fn auth_to_logged_in_id(
    auth_session: AuthSession,
) -> APIResult<ID> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    
    Ok(auth_session.user.unwrap().id)
}

pub async fn auth_to_is_admin_and_conn(
    auth_session: AuthSession,
) -> APIResult<(bool, DBConnection)> {
    let mut conn = auth_session.backend.get_connection().await?;
    if auth_session.user.is_none() {
        
        return Ok((false, conn));
    }
    let user = auth_session.user.unwrap();
    
    let admin = is_admin(&mut conn, user.id).await;
    Ok((admin, conn))
}

pub async fn auth_to_id_is_me_or_i_am_admin(
    auth_session: AuthSession,
    id: ID,
) -> APIResult<DBConnection> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let user = auth_session.user.unwrap();
    
    let mut conn = auth_session.backend.get_connection().await?;
    if user.id != id && !is_admin(&mut conn, user.id).await {
        return Err(APIError::UNAUTHORIZED);
    }

    Ok(conn)
}


pub async fn auth_and_path_to_id_is_me_or_i_am_admin(
    auth_session: AuthSession,
    user_id: ID,
) -> APIResult<DBConnection> {
    if auth_session.user.is_none() {
        return Err(APIError::UNAUTHORIZED);
    }
    let user = auth_session.user.unwrap();
    
    let mut conn = auth_session.backend.get_connection().await?;
    if user.id != user_id && !is_admin(&mut conn, user.id).await {
        return Err(APIError::UNAUTHORIZED);
    }
    
    Ok(conn)
}