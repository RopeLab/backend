
pub mod schema;
pub mod open_api;
pub mod auth;
pub mod backend;
mod user_data;
pub mod error;
pub mod permissions;
pub mod events;
pub mod cors;
pub mod firebase;
pub mod mails;
pub mod markdown_files;

use std::fmt::Debug;
use axum::{
    Router,
    routing::get,
};
use std::net::SocketAddr;
use axum_login::{AuthManagerLayerBuilder, permission_required};
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::info;


use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::routes::{add_admin_auth_routes, add_auth_routes};
use crate::backend::Backend;
use crate::cors::add_cors_layer;
use crate::events::{add_admin_event_routes};
use crate::events::users::{add_admin_event_user_routes, add_event_user_routes};
use crate::events::public::add_public_event_routes;
use crate::events::user_action::add_user_action_routes;
use crate::mails::{send_mail, send_password_reset_mail};
use crate::markdown_files::routes::add_admin_markdown_files_routes;
use crate::open_api::add_swagger_route;
use crate::permissions::{UserPermission};
use crate::permissions::routes::{add_admin_permission_routes, add_permission_routes};
use crate::user_data::{add_admin_user_data_routes, add_user_data_routes, UserData};



#[tokio::main]
async fn main() {
    /*
    send_password_reset_mail("maarten.behn@gmail.com", UserData {
        user_id: 0,
        name: "Stroby".to_string(),
        fetlife_name: "".to_string(),
        experience_text: "".to_string(),
        found_us_text: "".to_string(),
        goal_text: "".to_string(),
        role_factor: 0.0,
        open: false,
        show_name: false,
        show_role: false,
        show_open: false,
        show_fetlife: false,
        new: false,
    }, "https://reset_password_test.de").await.unwrap();
    
     */

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
    
    let mut router = Router::<Backend>::new();
    
    router = add_admin_auth_routes(router);
    router = add_admin_permission_routes(router);
    router = add_admin_event_routes(router);
    router = add_admin_user_data_routes(router);
    router = add_admin_event_user_routes(router);
    router = add_admin_markdown_files_routes(router);
    router = router.route_layer(permission_required!(Backend, UserPermission::Admin));

    router = add_swagger_route(router);
    router = add_auth_routes(router);
    router = add_user_data_routes(router);
    router = add_permission_routes(router);
    router = add_event_user_routes(router);
    router = add_public_event_routes(router);
    router = add_user_action_routes(router);
    
    router = router.route("/", get(|| async { "This is the Rope Lab Website Backend" }));
    router = router.layer(auth_layer);
    router = add_cors_layer(router);

    let app = router.with_state(backend);
    
    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("API Dashboard at: https://localhost:3001/swagger-ui");
    //open::that("localhost:3001/swagger-ui").unwrap();
    axum::serve(listener, app).await.unwrap();
}

