# New version notifications

sne can tell you when a new version of sne itself is out. This is off unless you turn it on.

## Check now

```sh
sne self check
```

This asks GitHub for the latest release and prints either that you are up to date, or the new version
with the command that updates it.

## Get a desktop notification

```sh
sne notify on
```

From then on, sne checks once a day in the background. When a new version is out, your system shows a
notification. Each version notifies once.

```sh
sne notify        # show whether notifications are on
sne notify off    # turn them off
```

You can also turn this on while installing: tick **Tell me when a new version of sne is available** in
the Windows setup wizard, or set `SNE_NOTIFY=1` before running `install.ps1` or `install.sh`.

## What it sets up

| System | Daily check | Notification |
|---|---|---|
| Windows | A task named **sne update check** in Task Scheduler, for your user only | A Windows notification; clicking it opens the releases page |
| macOS | A launchd agent in `~/Library/LaunchAgents` | A Notification Center message |
| Linux | A systemd user timer named `sne-update-check` | `notify-send`, from the `libnotify` package |

`sne notify off` removes all of it. No administrator rights are needed.

{% hint style="info" %}
On Linux without systemd, `sne notify on` prints a crontab line you can add instead.
{% endhint %}

## The one-line reminder

Even with notifications off, `sne install` asks GitHub for the latest sne version at most once a day,
in the background, without slowing the install. When a newer version exists, `sne install` and
`sne doctor` print one line about it.

Set `SNE_NO_UPDATE_CHECK=1` to turn this off. It is also off when the `CI` variable is set.

## Speed

None of this touches `sne use`, `sne env` or the automatic switch on `cd`. Those commands never use
the network.

## Update

Run the installer again. See [Updating and uninstalling](../help/uninstall.md).
