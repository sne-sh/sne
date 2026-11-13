# Changelog

## 0.1.0 (2026-11-13)

First release.

- `install`, `use`, `default`, `ls`, `ls-remote`, `uninstall`, `current`, `which`, `exec`, `alias`,
  `unalias`, `env`, `completions`, `doctor`.
- Shell setup for bash, zsh, fish, PowerShell and cmd, with optional automatic switching on `cd`.
- Reads `.nvmrc`, `.node-version` and `engines.node` in `package.json`.
- SHA-256 verification of every download; parallel installs; cached release list.
- Windows, Linux (glibc and musl) and macOS.
- Windows installers: a PowerShell one-liner (`install.ps1`) and a setup wizard built with Inno Setup.
- Windows builds for x64, x86 (32-bit) and ARM64, each as a setup wizard and a portable zip.
- Linux and macOS: `install.sh`, a universal macOS `.pkg`, and `.deb`, `.rpm`, `.apk` and Arch packages.
- Documentation in `docs/` (published with GitBook) and a project website in `site/`.
