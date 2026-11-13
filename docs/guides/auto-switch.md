# Switching automatically

With automatic switching, entering a project folder puts that project's Node.js version in your
terminal.

## Turn it on

Add `--use-on-cd` to the setup line in your shell profile. The installers add it for you.

```sh
eval "$(sne env --use-on-cd)"
```

See [Shell setup](../getting-started/shell-setup.md) for the line for each shell. It works in bash,
zsh, fish and PowerShell.

## What happens when you change folder

1. sne looks for a [version file](version-files.md) in the new folder and its parents.
2. If it finds one, and the active version does not already satisfy it, sne switches to an installed
   version that does.
3. If no installed version satisfies it, sne leaves the terminal as it is. Run `sne install` in the
   folder to get the version.
4. If there is no version file, nothing changes.

{% hint style="info" %}
Leaving a project keeps the version you were using. sne does not switch back to the default.
{% endhint %}

## Turn it off

Remove `--use-on-cd` from the setup line and open a new terminal. You can still run `sne use` by hand
in any project.

## Cost

The check runs when the folder changes. It reads a few small files and never uses the network. On
Windows it takes about 6 ms, most of which is the time Windows needs to start any program.
