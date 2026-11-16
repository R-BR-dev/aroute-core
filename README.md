# aroute-core

The core of [ARoute](https://github.com/R-BR-dev): a router for x86_64 machines that sends traffic
through several VPN and proxy uplinks according to rules.

This repository builds two programs:

* **`arouted`** — the daemon. It will own the router's configuration and drive the system (network,
  firewall, DNS, Wi-Fi, tunnels); today it serves the management API and the Web-UI files.
* **`aroute`** — the command-line client. It talks to the daemon over a Unix socket.

*Status: milestone M0 — a walking skeleton. One API endpoint, no router functions yet.*

## Layout

```
crates/
├── aroute-model/    data types shared by all components; no I/O
├── aroute-sys/      system adapters: the only crate that touches the host
├── aroute-engine/   proxy engines (sing-box, Xray); empty until M3
├── arouted/         the daemon: API, server, state
└── aroute-cli/      the `aroute` command
packaging/           RPM spec, container build script, systemd unit
scripts/             check.sh (what CI runs), make-dist.sh (source tarball)
openapi.json         API description, generated from the code
```

Dependencies point one way: `arouted` and `aroute-cli` use `aroute-model`; only `arouted` uses
`aroute-sys`. Keeping the model free of I/O is what lets the CLI, the installer and the tests share it.

## Building and running

ARoute targets Fedora and is built with Fedora's Rust toolchain:

```bash
sudo dnf install cargo rustfmt clippy gcc
```

```bash
cargo build
```

Run the daemon as an ordinary user by pointing it at a socket you may create:

```bash
cargo run -p arouted -- --socket "$XDG_RUNTIME_DIR/aroute.sock"
```

Then, in another terminal:

```bash
cargo run -p aroute-cli -- --socket "$XDG_RUNTIME_DIR/aroute.sock" status
```

```bash
curl http://127.0.0.1:8080/api/v1/system/info
```

### Daemon options

| Option | Environment | Default | |
|---|---|---|---|
| `--listen` | `AROUTE_LISTEN` | `127.0.0.1:8080` | HTTP API and Web-UI. Local only by default: the API has no authentication yet |
| `--socket` | `AROUTE_SOCKET` | `/run/aroute/api.sock` | Unix socket for the CLI, readable by its owner only |
| `--webui-dir` | `AROUTE_WEBUI_DIR` | `/usr/share/aroute/webui` | Static files of the Web-UI |
| | `AROUTE_LOG` | `info` | Log level or any `tracing` filter, e.g. `debug` |

Under systemd the same variables can be set in `/etc/aroute/arouted.env`.

## Checks

```bash
sh scripts/check.sh
```

runs the format check, clippy with warnings as errors, and the tests — the same script CI runs. To run
it with exactly CI's toolchain without installing anything:

```bash
podman run --rm -v "$PWD:/src" -w /src --security-opt label=disable registry.fedoraproject.org/fedora:44 sh -c 'dnf -y install cargo rustfmt clippy gcc >/dev/null && sh scripts/check.sh'
```

## The API description

`openapi.json` is generated from the route and type declarations and committed, so API changes are
visible in review and the Web-UI can generate its types from a tagged version. A test fails when the
file is stale; regenerate it with:

```bash
cargo run -p arouted -- openapi > openapi.json
```

## Packaging

```bash
podman run --rm -v "$PWD:/src" -w /src --security-opt label=disable registry.fedoraproject.org/fedora:44 sh packaging/build-rpm.sh
```

builds `aroute-core-<version>.x86_64.rpm` and the source RPM into `dist/`, runs rpmlint and an install
test. A clean checkout is packaged from git `HEAD`; with uncommitted changes the working tree is
packaged instead. Signing and publishing to <https://repo.rnbr-dev.com> are manual.

The version is stated in `Cargo.toml` (`[workspace.package]`) and in `packaging/rpm/aroute-core.spec`;
the build script refuses to continue if they differ.

## License

GPL-3.0-only. See [LICENSE](LICENSE).
