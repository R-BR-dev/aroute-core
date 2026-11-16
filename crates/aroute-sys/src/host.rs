//! Facts about the machine ARoute runs on, read from `/proc` and `/etc`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Why host information could not be read.
///
/// `thiserror` generates the `Display` and `Error` implementations from the
/// attributes; `#[source]` keeps the underlying I/O error attached as the cause.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("cannot read {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("unexpected content in {path}")]
    Parse { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostInfo {
    pub hostname: String,
    /// Human-readable OS name, `"Linux"` when the system does not say.
    pub os: String,
    pub kernel: String,
    /// Time since boot.
    pub uptime: Duration,
}

impl HostInfo {
    /// Reads the information of the running system.
    pub fn read() -> Result<Self, HostError> {
        Self::read_from(Path::new("/"))
    }

    /// Like [`HostInfo::read`], with `root` in place of `/`, so tests can point
    /// it at a directory they prepared.
    pub fn read_from(root: &Path) -> Result<Self, HostError> {
        let hostname = read_trimmed(&root.join("proc/sys/kernel/hostname"))?;
        let kernel = read_trimmed(&root.join("proc/sys/kernel/osrelease"))?;

        let uptime_path = root.join("proc/uptime");
        let uptime = parse_uptime(&read_trimmed(&uptime_path)?)
            .ok_or(HostError::Parse { path: uptime_path })?;

        // os-release is optional by specification, so its absence is not an error.
        let os = fs::read_to_string(root.join("etc/os-release"))
            .or_else(|_| fs::read_to_string(root.join("usr/lib/os-release")))
            .ok()
            .and_then(|text| parse_os_release(&text))
            .unwrap_or_else(|| "Linux".to_owned());

        Ok(Self {
            hostname,
            os,
            kernel,
            uptime,
        })
    }
}

fn read_trimmed(path: &Path) -> Result<String, HostError> {
    fs::read_to_string(path)
        .map(|text| text.trim().to_owned())
        .map_err(|source| HostError::Read {
            path: path.to_owned(),
            source,
        })
}

/// Extracts `PRETTY_NAME` from the text of an os-release file.
fn parse_os_release(text: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches(['"', '\'']).to_owned())
        .filter(|name| !name.is_empty())
}

/// Parses `/proc/uptime`, whose first field is the uptime in seconds.
fn parse_uptime(text: &str) -> Option<Duration> {
    let seconds: f64 = text.split_whitespace().next()?.parse().ok()?;
    Duration::try_from_secs_f64(seconds).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_release_pretty_name() {
        let text = "NAME=\"Fedora Linux\"\nVERSION_ID=44\nPRETTY_NAME=\"Fedora Linux 44 (Server Edition)\"\n";
        assert_eq!(
            parse_os_release(text).as_deref(),
            Some("Fedora Linux 44 (Server Edition)")
        );
    }

    #[test]
    fn os_release_without_quotes_or_name() {
        assert_eq!(
            parse_os_release("PRETTY_NAME=ARoute\n").as_deref(),
            Some("ARoute")
        );
        assert_eq!(parse_os_release("NAME=Fedora\n"), None);
        assert_eq!(parse_os_release("PRETTY_NAME=\"\"\n"), None);
    }

    #[test]
    fn uptime_first_field() {
        assert_eq!(
            parse_uptime("351735.42 1392812.77"),
            Some(Duration::from_secs_f64(351735.42))
        );
        assert_eq!(parse_uptime(""), None);
        assert_eq!(parse_uptime("soon"), None);
        assert_eq!(parse_uptime("-1.0 0"), None);
    }

    #[test]
    fn reads_from_a_prepared_root() {
        let root = tempfile::tempdir().unwrap();
        let write = |path: &str, content: &str| {
            let path = root.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        };
        write("proc/sys/kernel/hostname", "router\n");
        write("proc/sys/kernel/osrelease", "7.2.15-200.fc44.x86_64\n");
        write("proc/uptime", "120.50 400.00\n");
        write("etc/os-release", "PRETTY_NAME=\"Fedora Linux 44\"\n");

        let info = HostInfo::read_from(root.path()).unwrap();

        assert_eq!(
            info,
            HostInfo {
                hostname: "router".into(),
                os: "Fedora Linux 44".into(),
                kernel: "7.2.15-200.fc44.x86_64".into(),
                uptime: Duration::from_secs_f64(120.5),
            }
        );
    }

    #[test]
    fn missing_proc_file_names_the_path() {
        let root = tempfile::tempdir().unwrap();

        let error = HostInfo::read_from(root.path()).unwrap_err();

        assert!(matches!(error, HostError::Read { ref path, .. } if path.ends_with("hostname")));
    }

    #[test]
    fn missing_os_release_falls_back() {
        let root = tempfile::tempdir().unwrap();
        for (path, content) in [
            ("proc/sys/kernel/hostname", "router"),
            ("proc/sys/kernel/osrelease", "7.2.15"),
            ("proc/uptime", "1.00 1.00"),
        ] {
            let path = root.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }

        assert_eq!(HostInfo::read_from(root.path()).unwrap().os, "Linux");
    }

    #[test]
    fn reads_the_real_system() {
        let info = HostInfo::read().unwrap();

        assert!(!info.hostname.is_empty());
        assert!(!info.kernel.is_empty());
        assert!(info.uptime > Duration::ZERO);
    }
}
