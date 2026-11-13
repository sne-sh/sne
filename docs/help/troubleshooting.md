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

Neither the default version nor the [setup line](../getting-started/shell-setup.md) is active in this
terminal.

On Windows, run `sne use <version>` and open a new terminal. If the default version is missing from
your `PATH`, sne adds it. This happens when sne was installed before 0.1.0-beta.2 and then updated
with `sne update`, or when it was installed with `cargo install` or from the zip file. sne leaves
`PATH` alone when `SNE_DIR` or `SNE_NO_MODIFY_PATH=1` is set; it then prints the folder to add.

On Linux and macOS:

1. Check that the setup line is in your profile.
2. Open a new terminal.
3. Run `sne doctor`.

## `sne use` says "in every terminal"

The setup line has not run in this terminal, so sne changed the default version, which every terminal
uses. This is the normal behaviour in cmd and in any shell without the setup line. To give each
terminal its own version, add the [setup line](../getting-started/shell-setup.md) for your shell.

## PowerShell says running scripts is disabled

Windows PowerShell 5.1 blocks profile scripts by default, so it cannot load the setup line. `node` and
`sne` still work; only switching versions per terminal is missing. `install.ps1` and the setup wizard
leave the profile alone when they see this. To allow local scripts:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

Then run the installer again, or add the [setup line](../getting-started/shell-setup.md) yourself.

## "An Application Control policy has blocked this file"

Smart App Control, or a policy of your organization, refuses to run `sne.exe` because sne is not
code-signed yet. There is no "Run anyway" for this, and sne cannot work on that PC until one of these
changes:

* Smart App Control is turned off: **Windows Security**, **App & browser control**, **Smart App Control
  settings**. Windows does not let you turn it on again without reinstalling.
* Your administrator allows `sne.exe`, when the policy comes from your organization.

`install.ps1` checks this before it changes your `PATH` or profile.

## Windows Defender removed sne

Windows Defender can report `Behavior:Win32/Persistence` and delete `sne.exe`. It reacts to an unsigned
program that adds a scheduled task, which is what `sne notify on` does for the daily update check.
sne never adds that task without being asked.

1. Open **Windows Security**, **Virus & threat protection**, **Protection history**.
2. Open the entry, choose **Actions**, then **Restore**.
3. Run `sne notify off` if you prefer to be safe. `sne install` still tells you about new versions.

Please also [open an issue](https://github.com/sne-sh/sne/issues), so the false alarm can be reported
to Microsoft.

## Another Node.js is used instead of the one from sne

Something else puts a `node` earlier on your `PATH`: another version manager, or a Node.js installed
system-wide.

`sne use`, `sne default` and `sne doctor` name the folder when they see this.

* Remove that folder from your `PATH`. On Windows: Settings, "Edit environment variables for your
  account", `Path`.
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
