# Installation

Pick your system. Each method ends with the same result: an `sne` program on your `PATH`.

## Windows

### One command (recommended)

Run this in PowerShell. It needs no administrator rights.

```powershell
irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
```

The script:

1. Downloads the latest release for your processor and checks its SHA-256.
2. Puts `sne.exe` in `%LOCALAPPDATA%\Programs\sne`.
3. Adds that folder to your user `PATH`.
4. Adds the [setup line](shell-setup.md) to your PowerShell profile.

Set these before running it to change what it does:

| Variable | Effect |
|---|---|
| `SNE_VERSION` | Install this release, for example `v0.1.0`, instead of the latest |
| `SNE_INSTALL_DIR` | Put `sne.exe` in this folder |
| `SNE_NO_MODIFY_PATH=1` | Leave `PATH` alone |
| `SNE_NO_PROFILE=1` | Leave the PowerShell profile alone |

```powershell
$env:SNE_NO_PROFILE = '1'
irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
```

### Setup wizard

Download a setup program from the [releases page](https://github.com/sne-sh/sne/releases/latest) and
run it. It does the same as the script and adds an entry under **Installed apps** so you can uninstall
it later.

| Your Windows | Setup wizard | Portable zip |
|---|---|---|
| 64-bit (most PCs) | `sne-<version>-x64-setup.exe` | `sne-v<version>-x86_64-pc-windows-msvc.zip` |
| 32-bit | `sne-<version>-x86-setup.exe` | `sne-v<version>-i686-pc-windows-msvc.zip` |
| ARM64 | `sne-<version>-arm64-setup.exe` | `sne-v<version>-aarch64-pc-windows-msvc.zip` |

{% hint style="info" %}
The installers are not code-signed yet, so Windows SmartScreen shows a warning. Choose **More info**,
then **Run anyway**.
{% endhint %}

### Portable

Unzip the portable file anywhere, add the folder to your `PATH`, then do the
[shell setup](shell-setup.md) yourself.

{% hint style="warning" %}
Node.js stopped publishing 32-bit Windows builds after version 22. On 32-bit Windows, sne can install
Node.js 22 and older.
{% endhint %}

## macOS and Linux

### One command (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | sh
```

It needs no root. The script:

1. Downloads the latest release for your system and checks its SHA-256.
2. Puts `sne` in `~/.local/bin`.
3. Adds the [setup lines](shell-setup.md) to your bash, zsh or fish profile.

| Variable | Effect |
|---|---|
| `SNE_VERSION` | Install this release, for example `v0.1.0`, instead of the latest |
| `SNE_INSTALL_DIR` | Put `sne` in this folder |
| `SNE_NO_PROFILE=1` | Leave your shell profile alone |

```sh
curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | SNE_INSTALL_DIR=/opt/bin sh
```

### Packages

These are on the [releases page](https://github.com/sne-sh/sne/releases/latest). They install `sne`
for every user of the machine and do not change any shell profile, so finish with the
[shell setup](shell-setup.md).

| System | File | Install with |
|---|---|---|
| macOS (Intel and Apple silicon) | `sne-<version>-macos-universal.pkg` | `sudo installer -pkg <file> -target /` |
| Debian, Ubuntu | `sne_<version>_amd64.deb` or `_arm64.deb` | `sudo dpkg -i <file>` |
| Fedora, RHEL, openSUSE | `sne-<version>-1.x86_64.rpm` or `.aarch64.rpm` | `sudo rpm -i <file>` |
| Alpine | `sne_<version>_x86_64.apk` or `_aarch64.apk` | `sudo apk add --allow-untrusted <file>` |
| Arch | `sne-<version>-1-x86_64.pkg.tar.zst` or `-aarch64` | `sudo pacman -U <file>` |

{% hint style="info" %}
The macOS package is not signed yet. Install it with the command above, or right-click it in Finder
and choose **Open**.
{% endhint %}

### Portable

Download `sne-v<version>-<target>.tar.gz`, unpack it, and put `sne` somewhere on your `PATH`.

| Target | For |
|---|---|
| `x86_64-apple-darwin`, `aarch64-apple-darwin` | macOS on Intel, macOS on Apple silicon |
| `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` | Any Linux (static, no library requirements) |
| `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | Recent glibc-based Linux |

## Build from source

You need a recent stable [Rust toolchain](https://rustup.rs).

```sh
git clone https://github.com/sne-sh/sne
cd sne
cargo install --path .
```

## Check the download yourself

Every release has a `SHA256SUMS.txt` file that lists each file with its SHA-256.

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
```

## Check that it works

Open a new terminal and run:

```sh
sne --version
sne doctor
```
