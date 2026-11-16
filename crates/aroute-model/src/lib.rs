//! ARoute data model.
//!
//! Types shared by every component: the daemon serialises them, the CLI and the
//! installer deserialise them. This crate does no I/O, so it can be used anywhere.
//!
//! * [`api`] — bodies of REST API requests and responses.
//! * `config` (from M1) — the router's desired-state configuration.

pub mod api;
