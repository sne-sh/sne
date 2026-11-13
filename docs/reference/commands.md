# Commands

Run `sne help` or `sne <command> --help` to see this in your terminal. Wherever a command takes a
version, you can use anything from the [version syntax](version-syntax.md).

## Short forms

| Short | Same as |
|---|---|
| `sne -v` | `sne --version` |
| `sne -h` | `sne --help` |
| `sne i 22` | `sne install 22` |
| `sne u 22` | `sne use 22` |
| `sne d 22` | `sne default 22` |
| `sne l` | `sne ls` |
| `sne lr` | `sne ls-remote` |
| `sne rm 18` | `sne uninstall 18` |
| `sne c` | `sne current` |
| `sne w 22` | `sne which 22` |
| `sne x 20 -- npm test` | `sne exec 20 -- npm test` |
| `sne up` | `sne update` |
| `sne dr` | `sne doctor` |

## Summary

| Command | What it does |
|---|---|
| [`sne install`](#sne-install) | Install one or more versions |
| [`sne use`](#sne-use) | Switch this terminal to an installed version |
| [`sne default`](#sne-default) | Set the version new terminals start with |
| [`sne ls`](#sne-ls) | List installed versions |
| [`sne ls-remote`](#sne-ls-remote) | List versions available to install |
| [`sne uninstall`](#sne-uninstall) | Remove an installed version |
| [`sne current`](#sne-current) | Show the version active in this terminal |
| [`sne which`](#sne-which) | Show the path of a version's `node` program |
| [`sne exec`](#sne-exec) | Run a command with a specific version |
| [`sne alias`](#sne-alias) | Name a version, or list aliases |
| [`sne unalias`](#sne-unalias) | Remove an alias |
| [`sne env`](#sne-env) | Print the setup code for your shell |
| [`sne completions`](#sne-completions) | Print a tab-completion script |
| [`sne doctor`](#sne-doctor) | Check that sne is set up correctly |
| [`sne update`](#sne-update) | Update sne itself |
| [`sne self check`](#sne-self-check) | Look for a new version of sne |
| [`sne notify`](#sne-notify) | Turn new version notifications on or off |

## sne install

```
sne install [VERSIONS]...
```

Installs one or more versions. With several versions, the downloads run in parallel. With no version,
installs what the project asks for in its [version file](../guides/version-files.md).

Each download is checked against the SHA-256 published by nodejs.org. An interrupted install leaves
nothing behind.

```sh
sne install lts
sne install 20 22 24
sne install ">=20 <23"
sne install
```

Short form: `sne i`.

## sne use

```
sne use [VERSION]
```

Switches the current terminal to an installed version. With no version, uses what the project asks
for. Other terminals are not affected.

```sh
sne use 22
sne use lts/iron
sne use work
sne use
```

## sne default

```
sne default <VERSION>
```

Sets the version that new terminals start with.

## sne ls

```
sne ls
```

Lists installed versions. Also available as `sne list`.

## sne ls-remote

```
sne ls-remote [--lts] [FILTER]
```

Lists the versions you can install.

| Option | Effect |
|---|---|
| `--lts` | Show only LTS releases |
| `FILTER` | Show only versions that match, for example `22`, `>=20` or `lts/iron` |

The list is cached for one hour, so repeated calls do not use the network.

## sne uninstall

```
sne uninstall <VERSION>
```

Removes an installed version. Short form: `sne rm`.

## sne current

```
sne current
```

Prints the version active in this terminal.

## sne which

```
sne which [VERSION]
```

Prints the full path of the `node` program for a version. With no version, uses what the project asks
for.

## sne exec

```
sne exec <VERSION> -- <COMMAND>...
```

Runs one command with a specific version, without switching the terminal. It needs no shell setup.

```sh
sne exec 20 -- npm test
sne exec lts -- node script.js
```

## sne alias

```
sne alias [NAME] [VERSION]
```

Gives a version a name. With no arguments, lists all aliases.

```sh
sne alias work 20
sne alias
```

## sne unalias

```
sne unalias <NAME>
```

Removes an alias.

## sne env

```
sne env [--shell <SHELL>] [--use-on-cd]
```

Prints the code that sets up the current shell. You normally call it from your shell profile; see
[Shell setup](../getting-started/shell-setup.md).

| Option | Effect |
|---|---|
| `--shell <SHELL>` | One of `bash`, `zsh`, `fish`, `powershell`, `cmd`. Detected when left out. |
| `--use-on-cd` | Also switch versions when you `cd` into a project |

## sne completions

```
sne completions <SHELL>
```

Prints a tab-completion script for `bash`, `zsh`, `fish`, `powershell` or `elvish`.

## sne doctor

```
sne doctor
```

Checks the setup in this terminal and says what to fix. Run it first when something does not work.

## sne update

```
sne update
```

Replaces sne with the latest release. The download is checked against the SHA-256 in the release's
`SHA256SUMS.txt` first. `sne self update` does the same.

## sne self check

```
sne self check [--notify]
```

Asks GitHub whether a newer version of sne exists and prints the result.

| Option | Effect |
|---|---|
| `--notify` | Also show a desktop notification, once per new version. The daily background check uses this. |

## sne notify

```
sne notify [on|off]
```

Turns desktop notifications about new sne versions on or off. With no argument, shows the setting. See
[New version notifications](../guides/notifications.md).
