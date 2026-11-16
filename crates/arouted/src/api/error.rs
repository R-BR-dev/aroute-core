//! The error type of request handlers.

use aroute_model::api::ErrorBody;
use aroute_sys::host::HostError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// What a handler returns when it cannot answer.
///
/// Handlers return `Result<_, ApiError>`. axum calls [`IntoResponse`] on the
/// error, so this one place decides the status code and JSON body of every failure.
#[derive(Debug)]
pub enum ApiError {
    NotFound,
    /// Something went wrong on the router. The cause is logged, not sent to the client.
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorBody::new("not_found", "No such API resource"),
            ),
            ApiError::Internal(cause) => {
                // `{:#}` prints the whole chain: "cannot read /proc/uptime: No such file…"
                tracing::error!("request failed: {cause:#}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorBody::new("internal", "Internal error; see the router's log"),
                )
            }
        };
        (status, Json(body)).into_response()
    }
}

/// Lets handlers use `?` on host errors: `HostInfo::read()?`.
impl From<HostError> for ApiError {
    fn from(error: HostError) -> Self {
        ApiError::Internal(error.into())
    }
}
