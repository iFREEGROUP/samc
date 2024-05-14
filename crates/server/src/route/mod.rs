pub mod sam;
pub mod img;
use axum::{extract::DefaultBodyLimit, routing::{get, post}, Router};
use tower_http::cors::CorsLayer;

use crate::state::AppState;

use self::{img::{image_files, save_mask}, sam::{sam_anything, sam_anything_base64}};


pub fn routes() -> axum::Router<AppState> {
    Router::new()
    .route("/sam", post(sam_anything))
    .route("/sam_base64", post(sam_anything_base64))
    .route("/masks", post(save_mask))
    .route("/files", get(image_files))
    .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
    .layer(CorsLayer::permissive())
}