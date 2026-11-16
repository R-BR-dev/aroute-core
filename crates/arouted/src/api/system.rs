//! `/system` — the router itself.

use aroute_model::api::{API_VERSION, ErrorBody, SystemInfo};
use aroute_sys::host::HostInfo;
use axum::Json;
use axum::extract::State;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::ApiError;
use crate::state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(info))
}

/// Versions and basic facts about the router.
#[utoipa::path(
    get,
    path = "/system/info",
    tag = "system",
    responses(
        (status = 200, description = "The router is up", body = SystemInfo),
        (status = 500, description = "Host information could not be read", body = ErrorBody),
    )
)]
async fn info(State(state): State<AppState>) -> Result<Json<SystemInfo>, ApiError> {
    let host = HostInfo::read()?;

    Ok(Json(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        api_version: API_VERSION.to_owned(),
        hostname: host.hostname,
        os: host.os,
        kernel: host.kernel,
        uptime_seconds: host.uptime.as_secs(),
        daemon_uptime_seconds: state.daemon_uptime().as_secs(),
    }))
}
