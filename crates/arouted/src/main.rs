use std::io::IsTerminal;

use arouted::config::{Cli, Command};
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(Command::Openapi) = cli.command {
        println!("{}", arouted::api::openapi_json()?);
        return Ok(());
    }

    // Logs go to stderr; under systemd that is the journal. `AROUTE_LOG=debug`
    // (or any tracing filter) changes the level. Colours only for a terminal.
    let filter = EnvFilter::try_from_env("AROUTE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    arouted::server::run(cli.config).await
}
