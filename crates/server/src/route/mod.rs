pub mod sam;
use axum::{extract::DefaultBodyLimit, routing::post, Router};
use hyper::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

use self::sam::sam_anything;


pub fn routes() -> axum::Router<AppState> {
    Router::new()
    .route("/sam", post(sam_anything))
    .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
}