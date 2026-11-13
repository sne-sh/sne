<p align="center"><img src="assets/logo-512.png" alt="sne logo" width="120"></p>

# sne

**Simple Node Environment** (say "snee"): a fast Node.js version manager for Windows, Linux and macOS.

- **One small native binary.** No runtime, no shell-script framework to load on every prompt.
- **No shims.** `node`, `npm` and `npx` run directly; sne is not in the way of every call.
- **Switching is one filesystem operation.** `sne use` re-points a link; `PATH` never changes.
- **Works with your existing projects.** Reads `.nvmrc`, `.node-version` and `engines.node` in `package.json`.
- **Verified installs.** Every download is checked against the SHA-256 published by nodejs.org, and an
  interrupted install never leaves a half-installed version.
- **No admin rights on Windows.** Uses directory junctions, so Developer Mode is not needed.

[Website](https://sne-sh.github.io/sne/) | [Documentation](docs/README.md) | [Changelog](CHANGELOG.md)

> Status: beta (0.1.0-beta.2). The Windows build is tested end to end. The Linux and macOS code paths are
> covered by the test suite in CI but have had less real-world use.

## Install

### Windows

Run this in PowerShell. It needs no administrator rights:

```powershell
irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
```

It downloads the latest release, checks its SHA-256, puts `sne.exe` in `%LOCALAPPDATA%\Programs\sne`, adds
that folder to your `PATH`, and adds the setup line to your PowerShell profile. Set `SNE_NO_MODIFY_PATH=1`
or `SNE_NO_PROFILE=1` first to skip either step, or `SNE_VERSION=0.1.0-beta.2` to pick a release.

If you prefer a wizard, download a setup program from the
[releases page](https://github.com/sne-sh/sne/releases). It does the same and adds an uninstaller.
The installers are not code-signed yet, so Windows SmartScreen will ask you to confirm.

| Your Windows | Setup wizard | Portable (unzip and run) |
|---|---|---|
| 64-bit (most PCs) | `sne-<version>-x64-setup.exe` | `sne-<version>-x86_64-pc-windows-msvc.zip` |
| 32-bit | `sne-<version>-x86-setup.exe` | `sne-<version>-i686-pc-windows-msvc.zip` |
| ARM64 | `sne-<version>-arm64-setup.exe` | `sne-<version>-aarch64-pc-windows-msvc.zip` |

The portable zip holds only `sne.exe`: put it anywhere and add the shell setup line below yourself.
Node.js stopped publishing 32-bit Windows builds after version 22, so 32-bit Windows can install
Node 22 and older.

Windows PowerShell 5.1 blocks profile scripts by default. If the setup line does not run, allow local
scripts once with `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.

### Linux and macOS

```sh
curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | sh
```

It needs no root. It downloads the latest release, checks its SHA-256, puts `sne` in `~/.local/bin` and
adds the setup lines to your bash, zsh or fish profile. Set `SNE_INSTALL_DIR` to change the folder,
`SNE_NO_PROFILE=1` to leave your profile alone, or `SNE_VERSION=0.1.0-beta.2` to pick a release.

Packages are on the [releases page](https://github.com/sne-sh/sne/releases) too. These install `sne`
system-wide; add the shell setup line below afterwards.

| System | File | Install with |
|---|---|---|
| macOS (Intel and Apple silicon) | `sne-<version>-macos-universal.pkg` | `sudo installer -pkg <file> -target /` |
| Debian, Ubuntu | `sne_<version>_amd64.deb` / `_arm64.deb` | `sudo dpkg -i <file>` |
| Fedora, RHEL, openSUSE | `sne-<version>-1.x86_64.rpm` / `.aarch64.rpm` | `sudo rpm -i <file>` |
| Alpine | `sne_<version>_x86_64.apk` / `_aarch64.apk` | `sudo apk add --allow-untrusted <file>` |
| Arch | `sne-<version>-1-x86_64.pkg.tar.zst` / `-aarch64` | `sudo pacman -U <file>` |
| Any (portable) | `sne-<version>-<target>.tar.gz` | unpack and put `sne` on your `PATH` |

The macOS package is not signed yet: use the command above, or right-click it in Finder and choose Open.
Every file is listed with its SHA-256 in `SHA256SUMS.txt`.

To build from source instead, use a recent [Rust toolchain](https://rustup.rs):

```sh
cargo install --path .
```

### Shell setup

`install.ps1`, the Windows setup wizard and `install.sh` do this for you. Otherwise add one line to your
profile:


| Shell | File | Line |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(sne env --use-on-cd)"` |
| zsh | `~/.zshrc` | `eval "$(sne env --use-on-cd)"` |
| fish | `~/.config/fish/config.fish` | `sne env --use-on-cd \| source` |
| PowerShell | `$PROFILE` | `sne env --use-on-cd \| Out-String \| Invoke-Expression` |
| cmd | (per session) | `FOR /f "tokens=*" %i IN ('sne env --shell cmd') DO CALL %i` |

Leave out `--use-on-cd` if you do not want automatic switching when you enter a project.
Open a new terminal and run `sne doctor` to check the setup.

## Usage

```sh
sne install lts          # newest LTS release
sne install 20 22 24     # several versions, downloaded in parallel
sne install              # whatever the current project asks for

sne use 22               # switch this shell
sne use                  # switch to the project's version
sne default 22           # version for new shells

sne ls                   # installed versions
sne ls-remote --lts      # versions available to install
sne current              # active version
sne which 20             # path to that version's node executable

sne exec 20 -- npm test  # run one command with a specific version
sne alias work 20        # name a version, then: sne use work
sne uninstall 18

sne update               # update sne itself
sne notify on            # desktop notification when a new sne is out
```

A version can be written as `22`, `22.11`, `v22.11.0`, `latest`, `lts`, `lts/iron`, a range such as
`>=20 <23`, or an alias.

### Version files

When no version is given, and for automatic switching, sne looks in the current directory and then each
parent for the first of:

1. `.node-version`
2. `.nvmrc`
3. `package.json` with an `engines.node` field

With `--use-on-cd`, entering a project switches to its version if that version is installed and the
active one does not already satisfy the request. Leaving a project keeps the current version.

## How it works

`sne env` runs once per shell. It creates a link in sne's data directory that points at the default
version's `bin` directory, and puts that link on `PATH`. From then on, `sne use` only re-points the link
(a symlink on Linux and macOS, a junction on Windows). Each shell has its own link, so different
terminals can use different versions.

Installs stream the archive from nodejs.org, hash it while it downloads, and unpack it into a temporary
directory that is renamed into place only after the checksum matches. On Linux and macOS the archive is
decompressed as it arrives and never written to disk; on Windows the zip is unpacked on several threads.

## Configuration

| Variable | Purpose | Default |
|---|---|---|
| `SNE_DIR` | Where versions, aliases and the cache live | `%LOCALAPPDATA%\sne` on Windows, `$XDG_DATA_HOME/sne` or `~/.local/share/sne` elsewhere |
| `SNE_NO_UPDATE_CHECK` | Set to `1` so `sne install` never looks for a new sne version | not set |
| `SNE_NODE_MIRROR` | Download mirror | `https://nodejs.org/dist` (musl Linux: `https://unofficial-builds.nodejs.org/download/release`) |

## Performance

Measured on Windows 11 with the release build, averaged over 50 runs from PowerShell:

| Command | Time |
|---|---|
| `sne use 20` | 6.8 ms |
| `sne use --auto` (the cd hook) | 6.1 ms |
| `sne env` | 7.2 ms |
| `sne current` | 5.9 ms |
| starting an empty `cmd /c exit`, for scale | 8.9 ms |

Almost all of that is the cost of starting any process on Windows. Installing two Node versions in
parallel (`sne install lts 20`) took 2.4 s on a fast connection.

Run `benches/bench.sh` (needs [hyperfine](https://github.com/sharkdp/hyperfine)) to measure on your
machine; it also compares against fnm when fnm is installed.

## Development

```sh
cargo test          # unit tests plus an end-to-end test against a local fake mirror
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
