
pub mod schema;
pub mod open_api;
pub mod auth;
pub mod backend;
mod user_data;
pub mod error;
pub mod permissions;
pub mod util;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use axum_login::{AuthManagerLayerBuilder, login_required, permission_required};
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::auth::{add_auth_routes, add_admin_auth_routes};
use crate::backend::Backend;
use crate::open_api::add_swagger_route;
use crate::permissions::{add_admin_permission_routes, add_permission_routes, has_permission, UserPermission};
use crate::user_data::add_user_data_routes;


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


    let backend = Backend::new().await.unwrap();
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();
    
    // set up connection pool
    let mut router = Router::<Backend>::new();
    router = add_admin_auth_routes(router);
    router = add_admin_permission_routes(router);
    router = router.route_layer(permission_required!(Backend, UserPermission::Admin));

    router = add_swagger_route(router);
    router = add_auth_routes(router);
    router = add_user_data_routes(router);
    router = add_permission_routes(router);

    router = router.route("/", get(|| async { "This is the Rope Lab Website Backend" }));
    router = router.layer(auth_layer);

    let app = router.with_state(backend);
    
    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    open::that("localhost:3000/swagger-ui").unwrap();
    axum::serve(listener, app).await.unwrap();
}

