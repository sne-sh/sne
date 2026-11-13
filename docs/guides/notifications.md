# New version notifications

sne tells you when a new version of sne itself is out, beta versions included. This is on by default,
and one command turns it off.

## Check now

```sh
sne self check
```

This asks GitHub for the latest release and prints either that you are up to date, or the new version.

## Update

```sh
sne update
```

This downloads the latest release for your system, checks its SHA-256, and replaces the sne program.
Your installed Node.js versions are not touched.

If sne was installed from a system package (`.deb`, `.rpm`, `.apk`, Arch or the macOS `.pkg`), the
program belongs to the system and `sne update` cannot replace it. Install the newer package instead.

## Desktop notifications

sne checks once a day in the background. When a new version is out, your system shows a notification
with the new version and the command to run. Each version notifies once.

Notifications are turned on for you:

- by the Windows setup wizard (untick **Tell me when a new version of sne is available** to skip it) and
  by `install.ps1`;
- on macOS and Linux desktops, by sne itself the first time you run `sne install`. It prints one line
  when it does. It does nothing on servers, in containers or in CI.

```sh
sne notify        # show whether notifications are on
sne notify off    # turn them off
sne notify on     # turn them on again
```

To install without notifications, set `SNE_NO_NOTIFY=1` before running `install.ps1` or `install.sh`.

## What it sets up

| System | Daily check | Notification |
|---|---|---|
| Windows | A task named **sne update check** in Task Scheduler, for your user only | A Windows notification with the sne logo; clicking it opens the release notes |
| macOS | A launchd agent in `~/Library/LaunchAgents` | A Notification Center message (macOS shows its own icon for it) |
| Linux | A systemd user timer named `sne-update-check` | `notify-send` with the sne logo, from the `libnotify` package |

`sne notify off` removes all of it. No administrator rights are needed.

{% hint style="info" %}
On Linux without systemd, `sne notify on` prints a crontab line you can add instead.
{% endhint %}

## The one-line reminder

With notifications off, `sne install` still asks GitHub for the latest sne version at most once a day,
in the background, without slowing the install. When a newer version exists, `sne install` and
`sne doctor` print one line about it, telling you to run `sne update`.

Set `SNE_NO_UPDATE_CHECK=1` to turn this off. It is also off when the `CI` variable is set.

## Speed

None of this touches `sne use`, `sne env` or the automatic switch on `cd`. Those commands never use
the network.

