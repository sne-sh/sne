# Troubleshooting

Start with `sne doctor`. It checks the setup in the current terminal and says what to fix.

## `sne` is not recognised or not found

The folder that holds `sne` is not on your `PATH`, or the terminal was open before you installed.

1. Open a new terminal and try again.
2. If it still fails, check where sne was installed:
   * Windows: `%LOCALAPPDATA%\Programs\sne`
   * Linux and macOS with `install.sh`: `~/.local/bin`
3. Add that folder to your `PATH`.

## `node` is not found after installing a version

The [setup line](../getting-started/shell-setup.md) is missing from your shell profile, or it has not
run in this terminal.

1. Check that the line is in your profile.
2. Open a new terminal.
3. Run `sne doctor`.

## PowerShell says running scripts is disabled

Windows PowerShell 5.1 blocks profile scripts by default. Allow local scripts once:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

## Another Node.js is used instead of the one from sne

Something else puts a `node` earlier on your `PATH`: another version manager, or a Node.js installed
system-wide.

* Remove the lines of nvm, fnm, volta or similar from your shell profile.
* Or uninstall the system-wide Node.js.
* Check which one runs: `which node` (Linux, macOS, Git Bash) or `Get-Command node` (PowerShell).

## A version is not installed although a newer one exists

`sne use` only chooses from installed versions. Install it first:

```sh
sne install 22
sne use 22
```

## No build for this platform

Some combinations do not exist on nodejs.org. The common case is 32-bit Windows, which has no builds
after Node.js 22. Choose an older version.

## Checksum mismatch

The download did not match the SHA-256 published by the server, so sne installed nothing. Try again.
If it keeps happening and you use `SNE_NODE_MIRROR`, the mirror may be out of date.

## Downloads fail behind a firewall

Point sne at a mirror you can reach with
[`SNE_NODE_MIRROR`](../reference/configuration.md).

## Windows SmartScreen or macOS Gatekeeper blocks the installer

The installers are not code-signed yet.

* Windows: choose **More info**, then **Run anyway**. Or use the `install.ps1` command instead.
* macOS: run `sudo installer -pkg <file> -target /`, or use the `install.sh` command instead.

## Still stuck

Open an issue at [github.com/sne-sh/sne/issues](https://github.com/sne-sh/sne/issues) and include the
output of `sne doctor` and `sne --version`.
