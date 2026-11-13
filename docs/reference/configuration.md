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

sne still checks each download against the `SHASUMS256.txt` that the mirror serves.

## Installer settings

The install scripts have their own variables, such as `SNE_VERSION` and `SNE_INSTALL_DIR`. They are
listed on the [Installation](../getting-started/installation.md) page.
