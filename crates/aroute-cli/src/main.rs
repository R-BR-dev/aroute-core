//! `aroute` — command-line client of the ARoute Core.
//!
//! It talks to the running `arouted` over its Unix socket, using the same REST
//! API as the Web-UI. Access is controlled by the socket's file permissions
//! (root only), so no password is involved.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use aroute_model::api::{API_VERSION, DEFAULT_SOCKET_PATH, ErrorBody, SystemInfo};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aroute", version, about = "Control the ARoute Core")]
struct Cli {
    /// Unix socket of the running arouted.
    #[arg(long, global = true, env = "AROUTE_SOCKET", default_value = DEFAULT_SOCKET_PATH)]
    socket: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show whether the Core is running, and basic facts about the router.
    Status {
        /// Print the raw JSON answer.
        #[arg(long)]
        json: bool,
    },
}

// One request and exit: a single-threaded runtime is all this needs.
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = Client::new(&cli.socket)?;

    match cli.command {
        Command::Status { json } => {
            let info: SystemInfo = client.get("/system/info").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                print!("{}", render_status(&info));
            }
        }
    }
    Ok(())
}

/// HTTP client bound to the daemon's Unix socket.
struct Client {
    http: reqwest::Client,
    socket: PathBuf,
}

impl Client {
    fn new(socket: &Path) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .unix_socket(socket)
            .build()
            .context("cannot set up the HTTP client")?;
        Ok(Self {
            http,
            socket: socket.to_owned(),
        })
    }

    /// GETs `path` (relative to `/api/v1`) and decodes the JSON answer as `T`.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        // The host name is a placeholder: the connection goes to the socket.
        let url = format!("http://arouted/api/{API_VERSION}{path}");

        let response = self.http.get(url).send().await.with_context(|| {
            format!(
                "cannot reach arouted at {} (is it running? are you root?)",
                self.socket.display()
            )
        })?;

        let status = response.status();
        if !status.is_success() {
            // Prefer the API's own message; fall back to the bare status.
            match response.json::<ErrorBody>().await {
                Ok(body) => bail!("arouted answered {status}: {}", body.error.message),
                Err(_) => bail!("arouted answered {status}"),
            }
        }
        response
            .json()
            .await
            .context("arouted sent an answer this version of aroute cannot read")
    }
}

fn render_status(info: &SystemInfo) -> String {
    format!(
        "ARoute Core {version} (API {api})\n\
         Host:    {host}\n\
         System:  {os}\n\
         Kernel:  {kernel}\n\
         Uptime:  {uptime} (Core: {daemon})\n",
        version = info.version,
        api = info.api_version,
        host = info.hostname,
        os = info.os,
        kernel = info.kernel,
        uptime = format_uptime(info.uptime_seconds),
        daemon = format_uptime(info.daemon_uptime_seconds),
    )
}

/// `93784` → `"1d 2h 3m"`. Seconds are shown only below one minute.
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    let minutes = seconds % 3_600 / 60;

    match (days, hours, minutes) {
        (0, 0, 0) => format!("{seconds}s"),
        (0, 0, m) => format!("{m}m"),
        (0, h, m) => format!("{h}h {m}m"),
        (d, h, m) => format!("{d}d {h}h {m}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_formats() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(59), "59s");
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(3_600), "1h 0m");
        assert_eq!(format_uptime(93_784), "1d 2h 3m");
    }

    #[test]
    fn status_lists_every_fact() {
        let info = SystemInfo {
            version: "0.1.0".into(),
            api_version: "v1".into(),
            hostname: "router".into(),
            os: "Fedora Linux 44".into(),
            kernel: "7.2.15".into(),
            uptime_seconds: 93_784,
            daemon_uptime_seconds: 65,
        };

        assert_eq!(
            render_status(&info),
            "ARoute Core 0.1.0 (API v1)\n\
             Host:    router\n\
             System:  Fedora Linux 44\n\
             Kernel:  7.2.15\n\
             Uptime:  1d 2h 3m (Core: 1m)\n"
        );
    }

    #[test]
    fn command_line_is_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
