pub mod sam;
use axum::{extract::DefaultBodyLimit, routing::post, Router};
use hyper::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

use self::sam::{sam_anything, sam_anything_base64};


pub fn routes() -> axum::Router<AppState> {
    Router::new()
    .route("/sam", post(sam_anything))
    .route("/sam_base64", post(sam_anything_base64))
    .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
}