//! Command line of `arouted`.
//!
//! These are start-up settings only (where to listen, where files are). The
//! router's own configuration will live in the config store, not here.

use std::net::SocketAddr;
use std::path::PathBuf;

use aroute_model::api::DEFAULT_SOCKET_PATH;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "arouted", version, about = "ARoute Core daemon")]
pub struct Cli {
    #[command(flatten)]
    pub config: Config,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Args)]
pub struct Config {
    /// Address of the HTTP API and Web-UI.
    ///
    /// Local only by default: the API has no authentication yet.
    #[arg(long, env = "AROUTE_LISTEN", default_value = "127.0.0.1:8080")]
    pub listen: SocketAddr,

    /// Unix socket for the `aroute` CLI.
    #[arg(long, env = "AROUTE_SOCKET", default_value = DEFAULT_SOCKET_PATH)]
    pub socket: PathBuf,

    /// Directory with the Web-UI static files.
    #[arg(
        long,
        env = "AROUTE_WEBUI_DIR",
        default_value = "/usr/share/aroute/webui"
    )]
    pub webui_dir: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the OpenAPI document and exit.
    Openapi,
}
