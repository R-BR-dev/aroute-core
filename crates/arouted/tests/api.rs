//! Requests against the real router, without opening a socket.
//!
//! `oneshot` hands one request straight to the router, so these tests exercise
//! routing, handlers and serialisation exactly as a network client would.

// The `get` helper is not a `#[test]` function, so the workspace's `unwrap` lint
// would apply to it; in a test, panicking is the right way to fail.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use aroute_model::api::{ErrorBody, SystemInfo};
use arouted::state::AppState;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn app(webui_dir: &Path) -> Router {
    arouted::api::router(AppState::new(), webui_dir)
}

fn app_without_webui() -> Router {
    app(Path::new("/nonexistent/webui"))
}

async fn get(app: Router, path: &str) -> (StatusCode, String, Vec<u8>) {
    let request = Request::get(path).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, content_type, body)
}

#[tokio::test]
async fn system_info_describes_this_build_and_host() {
    let (status, _, body) = get(app_without_webui(), "/api/v1/system/info").await;

    assert_eq!(status, StatusCode::OK);
    let info: SystemInfo = serde_json::from_slice(&body).unwrap();
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(info.api_version, "v1");
    assert!(!info.hostname.is_empty());
    assert!(!info.kernel.is_empty());
}

#[tokio::test]
async fn unknown_api_path_is_a_json_404() {
    let (status, content_type, body) = get(app_without_webui(), "/api/v1/no/such/thing").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(content_type, "application/json");
    let error: ErrorBody = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "not_found");
}

#[tokio::test]
async fn openapi_document_is_served_and_lists_system_info() {
    let (status, _, body) = get(app_without_webui(), "/api/v1/openapi.json").await;

    assert_eq!(status, StatusCode::OK);
    let document: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(document["paths"]["/api/v1/system/info"]["get"].is_object());
}

/// `openapi.json` in the repository is what the Web-UI generates its types from.
#[test]
fn committed_openapi_document_is_current() {
    let committed = include_str!("../../../openapi.json");
    let generated = arouted::api::openapi_json().unwrap();

    assert_eq!(
        committed.trim(),
        generated.trim(),
        "openapi.json is out of date; run: cargo run -p arouted -- openapi > openapi.json"
    );
}

#[tokio::test]
async fn webui_files_and_app_routes_are_served() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<title>ARoute</title>").unwrap();
    std::fs::write(dir.path().join("app.js"), "console.log('hi')").unwrap();

    let (status, content_type, body) = get(app(dir.path()), "/app.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.contains("javascript"), "{content_type}");
    assert_eq!(body, b"console.log('hi')");

    // A path that only the single-page app knows gets index.html.
    for path in ["/", "/rules/42"] {
        let (status, content_type, body) = get(app(dir.path()), path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            content_type.starts_with("text/html"),
            "{path}: {content_type}"
        );
        assert_eq!(body, b"<title>ARoute</title>", "{path}");
    }
}

#[tokio::test]
async fn missing_webui_says_so() {
    let (status, _, body) = get(app_without_webui(), "/").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        String::from_utf8(body)
            .unwrap()
            .contains("Web-UI is not installed")
    );
}
