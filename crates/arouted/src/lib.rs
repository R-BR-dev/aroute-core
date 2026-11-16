//! The ARoute Core daemon, as a library.
//!
//! `main.rs` only parses arguments and calls into this crate. Keeping the logic
//! in a library is what lets the integration tests in `tests/` build exactly the
//! router the real daemon serves.

pub mod api;
pub mod config;
pub mod server;
pub mod state;
