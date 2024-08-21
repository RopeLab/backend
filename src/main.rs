
pub mod schema;
pub mod open_api;
pub mod auth;
pub mod db_backend;


use axum::{
    http::StatusCode,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use axum_login::{AuthManagerLayerBuilder, login_required};
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::auth::{add_auth_routes, add_login_auth_routes, AuthBackend};
use crate::db_backend::{new_pool, Pool};
use crate::open_api::add_swagger_route;



#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_diesel_async_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let pool = new_pool().await;
    
    let auth_backend = AuthBackend{ pool: pool.clone() };
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();
    
    // set up connection pool
    let mut router = Router::<Pool>::new();
    router = add_login_auth_routes(router);
    router = router.route_layer(login_required!(AuthBackend, login_url = "/login"));
    router = add_auth_routes(router);
    router = add_swagger_route(router);
    router = router.route("/", get(|| async { "This is the Rope Lab Website Backend" }));
    router = router.layer(auth_layer);

    let app = router.with_state(pool);
    
    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    open::that("localhost:3000/swagger-ui").unwrap();
    axum::serve(listener, app).await.unwrap();
}


// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn internal_error_from_string(text: &str) -> (StatusCode, String)
{
    (StatusCode::INTERNAL_SERVER_ERROR, text.to_string())
}