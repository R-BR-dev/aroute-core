//! System adapters.
//!
//! The only crate allowed to touch the host. Everything else in the Core works
//! with plain data and can therefore be tested without a router.
//!
//! * [`host`] — facts about the machine (name, OS, kernel, uptime).
//! * From M1: NetworkManager, netlink, nftables, dnsmasq; later hostapd and systemd.

pub mod host;
