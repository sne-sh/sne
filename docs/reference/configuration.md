# Configuration

sne has no configuration file. A few environment variables change where it stores things and where it
downloads from.

## Environment variables

| Variable | Purpose | Default |
|---|---|---|
| `SNE_DIR` | Folder for installed versions, aliases and the cache | See below |
| `SNE_NODE_MIRROR` | Base address for Node.js downloads | `https://nodejs.org/dist` |
| `SNE_NO_UPDATE_CHECK` | Set to `1` to stop `sne install` from looking for a new sne version | Not set |
| `SNE_NO_MODIFY_PATH` | Windows: set to `1` to stop `sne use`, `sne default` and `sne install` from adding the default Node version to your `PATH` | Not set |
| `SNE_ARCH` | Processor type of the Node.js builds to install: `x64`, `arm64`, `x86`, `armv7l`, `ppc64le`, `s390x` or `riscv64` | This system's |
| `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` | Proxy server for downloads, and the hosts that skip it | Not set |

## Apple silicon and Windows on ARM

Node.js 15 and older have no build for Apple silicon, and most versions before Node.js 20 have none for
Windows on ARM. When you install one of those, sne installs the x64 build, which macOS runs with
Rosetta 2 and Windows 11 with its built-in emulation, and says so. To always install a particular
build, set `SNE_ARCH`, for example `SNE_ARCH=x64 sne install 22`.

## Proxies and company certificates

sne uses the proxy in `HTTPS_PROXY` (or `ALL_PROXY`), like curl does, and skips it for the hosts in
`NO_PROXY`.

HTTPS certificates are checked against your system's certificate store: the Windows certificate
store, the macOS keychain, or the certificate bundle of your Linux distribution. If your company
inspects HTTPS traffic and has added its certificate to the system, sne accepts it. On a Linux system
with no certificate bundle, such as a slim container, sne uses the certificates built into it.

On Alpine and other musl-based Linux systems, the default mirror is
`https://unofficial-builds.nodejs.org/download/release`, because nodejs.org does not publish musl
builds.

sne also sets `SNE_SHELL_LINK` in each terminal. It is how sne finds that terminal's link. Do not set
it yourself.

## Where sne stores things

| System | Default folder |
|---|---|
| Windows | `%LOCALAPPDATA%\sne` |
| Linux, macOS | `$XDG_DATA_HOME/sne`, or `~/.local/share/sne` when that is not set |

Inside it:

| Folder | Contents |
|---|---|
| `versions` | One folder per installed Node.js version |
| `aliases` | Your aliases and the default version |
| `cache` | The cached list of Node.js releases |
| `shells` | One link per open terminal |

## Using a mirror

Set `SNE_NODE_MIRROR` to any server that has the same layout as `https://nodejs.org/dist`:

```sh
export SNE_NODE_MIRROR=https://mirror.example.com/node
sne install lts
```

The address must start with `https://`. Plain `http://` is accepted only for `127.0.0.1` and
`localhost`.

sne still checks each download against the `SHASUMS256.txt` that the mirror serves. That file comes
from the mirror too, so use a mirror only if you trust who runs it. See
[what a download is checked against](how-it-works.md#what-a-download-is-checked-against).

## Installer settings

The install scripts have their own variables, such as `SNE_VERSION` and `SNE_INSTALL_DIR`. They are
listed on the [Installation](../getting-started/installation.md) page.
