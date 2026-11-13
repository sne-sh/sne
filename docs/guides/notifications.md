# New version notifications

sne tells you when a new version of sne itself is out. On macOS and Linux desktops this is on by
default. On Windows you turn it on with one command.

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

On macOS and Linux desktops, sne turns notifications on by itself the first time you run
`sne install`. It prints one line when it does. It does nothing on servers, in containers or in CI.
To install without them, set `SNE_NO_NOTIFY=1` before running `install.sh`.

On Windows, notifications are off until you ask for them: run `sne notify on`, or tick **Tell me when
a new version of sne is available** in the setup wizard.

```sh
sne notify        # show whether notifications are on
sne notify off    # turn them off
sne notify on     # turn them on
```

{% hint style="warning" %}
On Windows the daily check is a task in Task Scheduler. sne is not code-signed yet, and antivirus
programs can treat an unsigned program that adds a scheduled task as malware. Windows Defender has
removed sne for this reason. That is why sne never adds the task without being asked. If it happens to
you, see [Troubleshooting](../help/troubleshooting.md#windows-defender-removed-sne).
{% endhint %}

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

With or without notifications, `sne install` asks GitHub for the latest sne version at most once a day,
in the background, without slowing the install. When a newer version exists, `sne install` and
`sne doctor` print one line about it, telling you to run `sne update`.

Set `SNE_NO_UPDATE_CHECK=1` to turn this off. It is also off when the `CI` variable is set.

## Speed

None of this touches `sne use`, `sne env` or the automatic switch on `cd`. Those commands never use
the network.

