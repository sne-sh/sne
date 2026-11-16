# Changelog

## 0.1.0 (2026-11-16)

The first stable release. If you run a beta, `sne update` moves you to it. From now on a stable sne is
offered stable releases only.

### Security

- Linux and macOS: an altered or damaged Node.js archive can no longer write files outside the folder
  it is unpacked into. The archive is unpacked while it downloads, before its SHA-256 can be checked,
  and a link in it followed by a file "inside" that link could write anywhere the user can. Now links
  must point inside the install folder, nothing is written through a link, a hard link may only name
  another file of the archive, and an absolute path is skipped. The checksum still failed in that
  case, but too late. Please update.
- Linux and macOS: setuid, setgid and sticky bits in an archive are dropped. They mattered when sne ran
  as root, for example in a container.
- Windows: sne shows its notifications itself, through the Windows API. It used to start Windows
  PowerShell with an encoded script from a scheduled task, which is also how a lot of malware behaves
  and what antivirus programs look for.

### New

- `sne install <version> --reinstall-packages-from <installed version>` installs the global npm
  packages of one version into another, at the same versions. Global packages belong to one Node
  version, so a new version used to start without them.

### Changes

- Downloads are tried again, twice, after a dropped connection or a busy server (HTTP 5xx or 429).
  One hiccup used to fail the whole install. A download that stops sending for good ends after 30
  minutes instead of waiting forever.
- `sne uninstall` moves the version aside before deleting it, so a version in use is never left half
  removed. Before, a running `node` could stop the delete halfway, after its aliases were gone.
- New version notifications check every 4 hours instead of once a day, so you hear about a release on
  the day it comes out. The notification reads "Update available" with the new version. If you turned
  notifications on before, run `sne notify on` once more to switch to the new schedule.
- HTTPS certificates are checked against the system's certificate store, like a browser or curl does.
  Behind a company proxy that inspects HTTPS, every download used to fail with a certificate error.
  A Linux system with no certificate bundle, such as a slim container, still works with the
  certificates built into sne.
- Apple silicon and Windows on ARM: a Node version with no build for the processor, such as Node 14 on
  a Mac with an M chip, is installed as its x64 build, which runs under emulation. Before, sne said no
  release matched.
- `SNE_ARCH` chooses the processor type of the Node builds to install, for example `SNE_ARCH=x64`.
- Proxies set in `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY` are now documented.
- `cargo install sne` no longer needs `--version`.
- On npm, a stable release is `latest` and a beta is `next`: `npm install -g sne-cli` gives the stable
  version, and `npm install -g sne-cli@next` the newest beta.

### Removed

Fewer, better-tested files. Nobody on a supported system loses anything.

- **32-bit Windows.** Node.js stopped publishing 32-bit Windows builds after version 22, which reaches
  its end of life in April 2027, and Windows 11 has no 32-bit edition. 0.1.0-beta.5 is the last sne for
  it. On a 64-bit Windows, `SNE_ARCH=x86` still installs 32-bit Node.js builds.
- **The glibc Linux files** (`*-unknown-linux-gnu.tar.gz`). The static `*-unknown-linux-musl` files
  run on every Linux, and the packages, `install.sh`, `sne update`, Homebrew, the AUR and npm already used them.
- **The Alpine `.apk` and the Arch package file.** On Alpine, run `sudo apk del sne`, then install with
  `install.sh`. On Arch, use the AUR package `sne-bin`.

## 0.1.0-beta.5 (2026-11-13)

This release is about security. Please update.

### Security

- Windows: new version notifications are no longer turned on by sne, `install.ps1` or the setup wizard. Windows Defender reported the daily update-check task as `Behavior:Win32/Persistence` and deleted sne. `sne notify on` still turns them on. Linux and macOS are unchanged.
- Project version files (`.nvmrc`, `.node-version`, `package.json`) are read only when they are small regular files. A repository could make `.nvmrc` a link to a private file, whose first line sne printed on `cd`, or put terminal escape codes in it.
- Downloads use HTTPS only. A plain `http://` address in `SNE_NODE_MIRROR`, `SNE_RELEASES_URL` or `SNE_DOWNLOAD_BASE` is refused, except for `127.0.0.1` and `localhost`, and so is a redirect from HTTPS to HTTP.
- A stable sne is offered stable releases only by `sne update` and the notifications. A beta is offered both.
- cmd: the setup lines are quoted, so a folder name with `&` no longer runs part of the path as a command.
- `sne doctor` reports a data folder that other users can write to (Linux and macOS).
- Every release file has a build provenance record: `gh attestation verify <file> --repo sne-sh/sne`.
- `sne.exe` carries its product name, version and icon.

### Changes

- `install.ps1` and `install.sh` run sne once before changing `PATH` or a profile, and say why when the system refuses to run it, for example Smart App Control.
- `install.ps1` and the setup wizard leave a PowerShell profile alone when the execution policy would refuse to load it.
- `sne update` leaves an install from Scoop, Homebrew, npm, Cargo or a Linux package to that package manager, and prints its command.
- The `cd` hook runs the sne program directly. With an npm install, a change of folder went from about 94 ms to 12 ms on Windows.
- Install with Scoop (`scoop bucket add sne https://github.com/sne-sh/sne`) or Homebrew (`brew tap sne-sh/sne https://github.com/sne-sh/sne`).
- sne is on crates.io: `cargo install sne --version <version>`.
- sne is on npm as `sne-cli`: `npm install -g sne-cli`, or the same with yarn, pnpm or bun.
- The README has a privacy section and a code signing policy.

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
