# Changelog

## 0.1.0-beta.3 (2026-11-13)

- New version notifications are on by default. The Windows installers turn them on, and elsewhere sne turns them on at the first `sne install` on a desktop. `sne notify off` turns them off, and `SNE_NO_NOTIFY=1` skips them when installing.
- Notifications, `sne update` and `sne self check` now include beta releases.

## 0.1.0-beta.2 (2026-11-13)

- No setup step after installing on Windows: the installers put the default Node version on `PATH`, so `node` and `sne use` work in every terminal, including cmd, Git Bash and both PowerShell editions.
- `sne use` in a terminal without the setup line now switches the version for every terminal, where it used to stop with an error.
- `install.ps1` sets up Windows PowerShell 5.1 and PowerShell 7 together.
- The Linux packages add the default Node version to `PATH` for login shells.
- Short forms: `sne -v` for the version, and one- or two-letter names for the common commands (`i`, `u`, `d`, `l`, `lr`, `rm`, `c`, `w`, `x`, `up`, `dr`).
- Release notes come from this changelog.

## 0.1.0-beta.1 (2026-11-13)

First public release, as a beta.

- `install`, `use`, `default`, `ls`, `ls-remote`, `uninstall`, `current`, `which`, `exec`, `alias`, `unalias`, `env`, `completions`, `doctor`.
- Shell setup for bash, zsh, fish, PowerShell and cmd, with optional automatic switching on `cd`.
- Reads `.nvmrc`, `.node-version` and `engines.node` in `package.json`.
- SHA-256 verification of every download; parallel installs; cached release list.
- Windows, Linux (glibc and musl) and macOS.
- Windows installers: a PowerShell one-liner (`install.ps1`) and a setup wizard built with Inno Setup.
- Windows builds for x64, x86 (32-bit) and ARM64, each as a setup wizard and a portable zip.
- Linux and macOS: `install.sh`, a universal macOS `.pkg`, and `.deb`, `.rpm`, `.apk` and Arch packages.
- Documentation in `docs/` (published with GitBook) and a project website in `site/`.
- `sne self check` looks for a new sne version; `sne notify on` shows a desktop notification when one is out (Task Scheduler, launchd or a systemd user timer). Off by default.
- `sne update` replaces sne with the latest release, after checking its SHA-256.
