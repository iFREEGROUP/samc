use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde_json::json;

#[derive(Debug,thiserror::Error)]
pub enum Error {
    #[error("load image data error:{0}")]
    ImageLoadError(#[from] image::ImageError),
    #[error("tensor data error:{0}")]
    TensorError(#[from] segment::Error),
    #[error("require param")]
    RequireParam,
    #[error("参数错误: {0}")]
    ErrorParam(String),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
    #[error("{0}")]
    RequestApi(#[from] reqwest::Error),
    #[error("not dir")]
    NotDir,
}

#[allow(unreachable_patterns)]
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Error::ImageLoadError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::TensorError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::RequireParam => (StatusCode::BAD_REQUEST, "require param".to_owned()),
            Error::ErrorParam(e) => (StatusCode::BAD_REQUEST, e),
            Error::NotDir => (StatusCode::BAD_REQUEST, "not dir".to_owned()),
            Error::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::RequestApi(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "server error".to_owned()),
        };
        
        let body = Json(json!({
            "code": status.as_u16(),
            "message": message,
            "data":"",
        }));

        (status, body).into_response()
    }
}