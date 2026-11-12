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

> Status: early (0.1.0). The Windows build is tested end to end. The Linux and macOS code paths are
> covered by the test suite in CI but have had less real-world use.

## Install

### Windows

Run this in PowerShell. It needs no administrator rights:

```powershell
irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
```

It downloads the latest release, checks its SHA-256, puts `sne.exe` in `%LOCALAPPDATA%\Programs\sne`, adds
that folder to your `PATH`, and adds the setup line to your PowerShell profile. Set `SNE_NO_MODIFY_PATH=1`
or `SNE_NO_PROFILE=1` first to skip either step, or `SNE_VERSION=v0.1.0` to pick a release.

If you prefer a wizard, download `sne-<version>-x64-setup.exe` from the
[releases page](https://github.com/sne-sh/sne/releases). It does the same and adds an uninstaller.
The installer is not code-signed yet, so Windows SmartScreen will ask you to confirm.

Windows PowerShell 5.1 blocks profile scripts by default. If the setup line does not run, allow local
scripts once with `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.

### Linux and macOS

Download the archive for your platform from the releases page and put `sne` somewhere on your `PATH`, or
build from source with a recent [Rust toolchain](https://rustup.rs):

```sh
cargo install --path .
```

### Shell setup

The Windows installers do this for PowerShell. For other shells, add one line to your profile:


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
