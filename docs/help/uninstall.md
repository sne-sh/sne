# Updating and uninstalling

## Update sne

Run the installer again. It replaces the program and keeps your installed Node.js versions.

{% tabs %}
{% tab title="Windows" %}
```powershell
irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
```

Or download and run the newer setup wizard.
{% endtab %}

{% tab title="macOS and Linux" %}
```sh
curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | sh
```

If you installed a package, install the newer package the same way.
{% endtab %}
{% endtabs %}

See the [changelog](https://github.com/sne-sh/sne/blob/main/CHANGELOG.md) for what each release
changed.

## Uninstall sne

### 1. Remove the program

| How you installed | How to remove |
|---|---|
| Windows setup wizard | **Settings > Apps > Installed apps > sne > Uninstall** |
| `install.ps1` | Delete `%LOCALAPPDATA%\Programs\sne` and remove that folder from your user `PATH` |
| `install.sh` | `rm ~/.local/bin/sne` |
| macOS package | `sudo rm /usr/local/bin/sne` |
| Debian, Ubuntu | `sudo dpkg -r sne` |
| Fedora, RHEL, openSUSE | `sudo rpm -e sne` |
| Alpine | `sudo apk del sne` |
| Arch | `sudo pacman -R sne` |

### 2. Remove the setup line

Delete the lines that mention `sne env` from your shell profile (`~/.bashrc`, `~/.zshrc`,
`~/.config/fish/conf.d/sne.fish`, or the file that `$PROFILE` names in PowerShell).

The line added by the installers checks that sne exists before it runs, so leaving it in place causes
no errors.

### 3. Remove the installed Node.js versions

This deletes every Node.js version that sne installed, with their global npm packages.

{% tabs %}
{% tab title="Windows" %}
```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\sne"
```
{% endtab %}

{% tab title="macOS and Linux" %}
```sh
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/sne"
```
{% endtab %}
{% endtabs %}
