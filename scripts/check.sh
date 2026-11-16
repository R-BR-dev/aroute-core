#!/bin/sh
# Format check, lints and tests: exactly what CI runs.
#
# Uses Fedora's Rust toolchain so the result matches CI and the RPM build:
#   sudo dnf install cargo rustfmt clippy gcc
# or, without installing anything on the host:
#   podman run --rm -v "$PWD:/src" -w /src --security-opt label=disable \
#       registry.fedoraproject.org/fedora:44 \
#       sh -c 'dnf -y install cargo rustfmt clippy gcc >/dev/null && sh scripts/check.sh'
set -eu
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
