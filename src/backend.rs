use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use http::request::Parts;
use crate::error::{APIError, APIResult};

pub type DBPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct Backend {
    pub db_pool: DBPool
}

pub struct DBConnection (
    pub bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>
);


impl Backend {
    pub async fn new() -> APIResult<Self> {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        let pool = DBPool::builder()
            .build(config)
            .await
            .map_err(APIError::internal)?;

        Ok(Backend {
            db_pool: pool,
        })
    }

    pub async fn get_connection(&self) -> APIResult<DBConnection> {
        let conn = self.db_pool
            .get_owned()
            .await
            .map_err(APIError::internal)?;
        Ok(DBConnection(conn))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for DBConnection
    where
        S: Send + Sync,
        Backend: FromRef<S>,
{
    type Rejection = APIError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> APIResult<Self> {
        let backend = Backend::from_ref(state);
        backend.get_connection().await
    }
}