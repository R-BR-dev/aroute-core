//! The REST API and the Web-UI file server.
//!
//! Routes are registered on an [`OpenApiRouter`], which builds the axum router
//! and the OpenAPI document from the same declarations, so the document cannot
//! drift from the code.

use std::path::Path;

use axum::Json;
use axum::Router;
use axum::routing::{any, get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

mod error;
pub mod system;
mod webui;

pub use error::ApiError;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ARoute Core API",
        description = "Management API of an ARoute router. The Web-UI and the `aroute` CLI are clients of it."
    ),
    tags((name = "system", description = "The router itself: versions, host facts"))
)]
struct ApiDoc;

/// Every API route, with its documentation.
fn api_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api/v1", system::routes())
}

/// The OpenAPI document as pretty-printed JSON — the content of `openapi.json`.
pub fn openapi_json() -> serde_json::Result<String> {
    serde_json::to_string_pretty(&api_routes().into_openapi())
}

/// Builds the complete application: API, OpenAPI document, Web-UI files.
pub fn router(state: AppState, webui_dir: &Path) -> Router {
    let (api, openapi) = api_routes().split_for_parts();

    let api = api
        .route(
            "/api/v1/openapi.json",
            get(move || async move { Json(openapi) }),
        )
        // Without this, an unknown API path would fall through to the Web-UI
        // and answer with HTML.
        .route("/api/{*path}", any(|| async { ApiError::NotFound }))
        .with_state(state);

    webui::attach(api, webui_dir).layer(TraceLayer::new_for_http())
}
