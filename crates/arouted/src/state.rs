//! State shared by all request handlers.

use std::time::{Duration, Instant};

/// Handed to every handler by axum, which clones it per request.
///
/// It holds only an `Instant` today, which is cheap to copy. When it grows
/// (config store, uplink manager), the shared parts go behind an `Arc` so that
/// cloning stays cheap and all handlers see the same data.
#[derive(Debug, Clone)]
pub struct AppState {
    started: Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    /// Time since the daemon started.
    pub fn daemon_uptime(&self) -> Duration {
        self.started.elapsed()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
