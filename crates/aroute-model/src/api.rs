//! Bodies of REST API requests and responses.
//!
//! The derives do the work here: `Serialize` / `Deserialize` turn a struct into
//! JSON and back, `ToSchema` describes it in the OpenAPI document. Doc comments
//! on fields end up in that document too.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Version of the REST API; also the URL prefix (`/api/v1`).
pub const API_VERSION: &str = "v1";

/// Where `arouted` listens for the `aroute` CLI unless told otherwise.
pub const DEFAULT_SOCKET_PATH: &str = "/run/aroute/api.sock";

/// Response of `GET /api/v1/system/info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SystemInfo {
    /// Version of the ARoute Core.
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Version of the REST API.
    #[schema(example = "v1")]
    pub api_version: String,
    pub hostname: String,
    /// Operating system name.
    #[schema(example = "Fedora Linux 44 (Server Edition)")]
    pub os: String,
    /// Kernel release.
    pub kernel: String,
    /// Seconds since the system booted.
    pub uptime_seconds: u64,
    /// Seconds since `arouted` started.
    pub daemon_uptime_seconds: u64,
}

/// Body of every error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    /// Stable machine-readable identifier.
    #[schema(example = "not_found")]
    pub code: String,
    /// Description for humans; may change between versions.
    pub message: String,
}

impl ErrorBody {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names are part of the API contract: renaming one must fail a test.
    #[test]
    fn system_info_json_shape() {
        let info = SystemInfo {
            version: "0.1.0".into(),
            api_version: API_VERSION.into(),
            hostname: "router".into(),
            os: "Fedora Linux 44".into(),
            kernel: "7.2.15".into(),
            uptime_seconds: 60,
            daemon_uptime_seconds: 5,
        };

        let json = serde_json::to_value(&info).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "version": "0.1.0",
                "api_version": "v1",
                "hostname": "router",
                "os": "Fedora Linux 44",
                "kernel": "7.2.15",
                "uptime_seconds": 60,
                "daemon_uptime_seconds": 5,
            })
        );
        assert_eq!(serde_json::from_value::<SystemInfo>(json).unwrap(), info);
    }

    #[test]
    fn error_body_json_shape() {
        let json = serde_json::to_value(ErrorBody::new("not_found", "No such resource")).unwrap();

        assert_eq!(
            json,
            serde_json::json!({ "error": { "code": "not_found", "message": "No such resource" } })
        );
    }
}
