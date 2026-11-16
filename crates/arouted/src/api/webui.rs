//! Serves the Web-UI, a single-page app made of static files.

use std::path::Path;

use axum::Router;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use tower_http::services::ServeDir;

/// Makes `router` answer everything it has no route for from `dir`.
///
/// Existing files are served as they are. Any other path gets `index.html`,
/// because the app does its own routing in the browser (`/rules`, `/uplinks`, …).
///
/// The directory is looked at on every request, not at start-up, so installing
/// the `aroute-webui` package takes effect without restarting the daemon.
pub fn attach(router: Router, dir: &Path) -> Router {
    let index = dir.join("index.html");

    let spa_fallback = move || {
        let index = index.clone();
        async move {
            match tokio::fs::read(&index).await {
                Ok(html) => Html(html).into_response(),
                Err(_) => not_installed(),
            }
        }
    };

    router.fallback_service(ServeDir::new(dir).fallback(spa_fallback.into_service()))
}

fn not_installed() -> Response {
    (
        StatusCode::NOT_FOUND,
        "ARoute Core is running, but the Web-UI is not installed (package aroute-webui).\n",
    )
        .into_response()
}
