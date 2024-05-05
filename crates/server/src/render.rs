use axum::{response::IntoResponse, http::HeaderValue};
use hyper::{StatusCode, header};
use serde::Serialize;


#[derive(Debug, serde::Serialize)]
pub struct Payload<T: serde::Serialize>{
    pub(crate) code: u16,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<T>,
}

impl<T:serde::Serialize> Default for Payload<T> {
    fn default() -> Self {
        Self {
            code: 200,
            message: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T:serde::Serialize> Payload<T> {
    pub fn new<S: Into<String>>(code: StatusCode, message:S, data:T) ->Self {
        Self { code:code.as_u16(), message:message.into(), data: Some(data) }
    }

    pub fn success(data:T) ->Self {
        Self::new(StatusCode::OK, "", data)
    }

    pub fn error<S: Into<String>>(code: StatusCode, message:S) ->Self {
        Self { code: code.as_u16(), message: message.into(), data: None }
    }
}

impl<T> IntoResponse for Payload<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match serde_json::to_vec(&self) {
            Ok(bytes) => {
                let status_code = match self.code {
                    400 => StatusCode::BAD_REQUEST,
                    401 => StatusCode::UNAUTHORIZED,
                    403 => StatusCode::FORBIDDEN,
                    500=> StatusCode::INTERNAL_SERVER_ERROR,
                    200 => StatusCode::OK,
                    _ => StatusCode::OK,
                };
                (
                    status_code,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/json"),
                    )],
                    bytes,
                )
            }
            .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}