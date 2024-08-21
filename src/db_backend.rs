use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use http::request::Parts;
use http::StatusCode;
use crate::internal_error;

pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn new_pool() -> Pool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    let pool = Pool::builder().build(config).await.unwrap();

    pool
}

pub struct DatabaseConnection(
    pub bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>,
);

impl DatabaseConnection {
    pub async fn from_pool(pool: &Pool) -> Result<Self, (StatusCode, String)> {
        let conn = pool.get_owned().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
    where
        S: Send + Sync,
        Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);
        Self::from_pool(&pool).await
    }
}