use axum::Router;
use http::{HeaderValue, Method};
use http::header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::CorsLayer;
use crate::backend::Backend;

const FRONT_END_URLS: [&str; 3] = [
    "http://localhost:3000", 
    "http://localhost:1313",
    "http://localhost:5173"
];
pub fn add_cors_layer(router: Router<Backend>) -> Router<Backend> {
    let origins = FRONT_END_URLS.map(|s| s.parse::<HeaderValue>().unwrap());
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, AUTHORIZATION, ACCEPT]);

    router.layer(cors)
}