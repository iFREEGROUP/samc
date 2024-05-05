use axum::{body::Body, http::Request};
use axum::{
    http::{header, Uri},
    response::{IntoResponse, Response},
};
use mime_guess;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "build/"]
pub struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();
        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => {
                // Response::builder().status(StatusCode::NOT_FOUND).body(boxed(Full::from("404"))).unwrap()
                // 直接转回index.html 实现前端路由的跳转
                let path = "index.html";
                let content = Asset::get(path).unwrap();
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
        }
    }
}

pub async fn index_handler(request: Request<Body>) -> impl IntoResponse {
    handler_404(request.uri()).await
}

async fn handler_404(uri: &Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("build/") {
        path = path.replace("build/", "");
    }

    if path.is_empty() {
        path = "index.html".into()
    }

    StaticFile(path)
}