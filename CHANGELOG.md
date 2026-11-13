# Changelog

## Unreleased

- Install with Scoop (`scoop bucket add sne https://github.com/sne-sh/sne`) or Homebrew (`brew tap sne-sh/sne https://github.com/sne-sh/sne`).
- sne is on crates.io: `cargo install sne --version <version>`.
- sne is on npm as `sne-cli`: `npm install -g sne-cli`, or the same with yarn, pnpm or bun.

## 0.1.0-beta.4 (2026-11-13)

- Windows: `sne use`, `sne default` and the first `sne install` add the default Node version to your `PATH` when it is missing. It was missing when sne was installed before 0.1.0-beta.2 and then updated with `sne update`, and after `cargo install` or the zip file. Before, sne only said to open a new terminal, and Node was still missing there.
- sne leaves `PATH` alone when `SNE_DIR` or `SNE_NO_MODIFY_PATH=1` is set, and prints the folder to add.
- `sne use`, `sne default` and `sne doctor` say when another Node comes first on `PATH`.
- `sne doctor` reports a default that points to a version that is no longer installed.
- `sne uninstall` says when it leaves no default version.

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
