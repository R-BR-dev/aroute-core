# ARoute Core: the arouted daemon and the aroute command-line client.
#
# Built with Fedora's Rust toolchain. Dependencies come from crates.io during
# %%build, so the build needs network access (plain rpmbuild in a container, see
# packaging/build-rpm.sh). Cargo.lock pins every version.

# Cargo strips the release binaries (profile.release in Cargo.toml), which
# leaves nothing for a debuginfo package.
%global debug_package %{nil}

Name:           aroute-core
Version:        0.0.1
Release:        1%{?dist}
Summary:        ARoute router core daemon and command-line client
License:        GPL-3.0-only
URL:            https://github.com/R-BR-dev/aroute-core
Source0:        %{url}/releases/download/v%{version}/%{name}-%{version}.tar.xz
ExclusiveArch:  x86_64

BuildRequires:  cargo >= 1.88
BuildRequires:  rust >= 1.88
BuildRequires:  gcc
BuildRequires:  systemd-rpm-macros
%{?systemd_requires}

%description
ARoute turns an x86_64 computer into a router that sends traffic through
several VPN and proxy uplinks according to rules.

This package contains arouted, the daemon that owns the router configuration
and exposes the management API, and aroute, its command-line client.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dpm 0755 target/release/arouted %{buildroot}%{_bindir}/arouted
install -Dpm 0755 target/release/aroute %{buildroot}%{_bindir}/aroute
install -Dpm 0644 packaging/systemd/arouted.service %{buildroot}%{_unitdir}/arouted.service
install -dm 0755 %{buildroot}%{_sysconfdir}/aroute

%check
cargo test --release --locked

%post
%systemd_post arouted.service

%preun
%systemd_preun arouted.service

%postun
%systemd_postun_with_restart arouted.service

%files
%license LICENSE
%doc README.md
%{_bindir}/arouted
%{_bindir}/aroute
%{_unitdir}/arouted.service
%dir %{_sysconfdir}/aroute

%changelog
* Mon Nov 16 2026 ARoute project - 0.0.1-1
- Daemon skeleton: system/info endpoint, OpenAPI document, Web-UI file serving.
- aroute CLI with the status command.
