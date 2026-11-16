//! Listeners and the life cycle of the daemon.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Context;
use sd_notify::NotifyState;
use tokio::net::{TcpListener, UnixListener};
use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::api;
use crate::config::Config;
use crate::state::AppState;

/// Serves the API until SIGTERM or Ctrl-C.
///
/// The same application is served twice: over TCP for browsers, and over a Unix
/// socket for the `aroute` CLI — the way back in when the web login is lost.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let app = api::router(AppState::new(), &config.webui_dir);

    let tcp = TcpListener::bind(config.listen)
        .await
        .with_context(|| format!("cannot listen on {}", config.listen))?;
    let unix = bind_unix_socket(&config.socket)
        .with_context(|| format!("cannot create socket {}", config.socket.display()))?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        listen = %tcp.local_addr()?,
        socket = %config.socket.display(),
        "arouted started"
    );

    // One token, cloned into every task that has to stop together.
    let shutdown = CancellationToken::new();
    tokio::spawn(cancel_on_signal(shutdown.clone()));

    // Tells systemd (Type=notify) that start-up is complete. Outside systemd
    // there is nobody to tell, and the call does nothing.
    if let Err(error) = sd_notify::notify(&[NotifyState::Ready]) {
        warn!("cannot notify systemd: {error}");
    }

    let tcp_server = axum::serve(tcp, app.clone())
        .with_graceful_shutdown(shutdown.clone().cancelled_owned())
        .into_future();
    let unix_server = axum::serve(unix, app)
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .into_future();

    // Runs both on this task; returns when both have finished, or at the first error.
    let result = tokio::try_join!(tcp_server, unix_server);

    let _ = fs::remove_file(&config.socket);
    result?;
    info!("arouted stopped");
    Ok(())
}

/// Creates the CLI socket, replacing one left behind by a previous run.
///
/// Only the owner may connect: whoever can open this socket controls the router
/// without a password.
fn bind_unix_socket(path: &Path) -> io::Result<UnixListener> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let listener = UnixListener::bind(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

async fn cancel_on_signal(shutdown: CancellationToken) {
    match wait_for_signal().await {
        Ok(name) => info!("received {name}, shutting down"),
        Err(error) => warn!("cannot watch signals, shutting down: {error}"),
    }
    shutdown.cancel();
}

async fn wait_for_signal() -> io::Result<&'static str> {
    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;

    // Whichever arrives first wins; the other branch is dropped.
    Ok(tokio::select! {
        _ = terminate.recv() => "SIGTERM",
        _ = interrupt.recv() => "SIGINT",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn socket_is_owner_only_and_replaces_a_stale_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api.sock");
        fs::write(&path, "stale").unwrap();

        let _listener = bind_unix_socket(&path).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[tokio::test]
    async fn socket_in_a_missing_directory_is_an_error() {
        let dir = tempfile::tempdir().unwrap();

        let result = bind_unix_socket(&dir.path().join("missing/api.sock"));

        assert!(result.is_err());
    }
}
