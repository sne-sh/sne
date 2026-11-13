# Shell setup

sne needs one line in your shell profile. The line runs `sne env` each time a terminal starts, which
puts the active Node.js version on your `PATH`.

{% hint style="info" %}
`install.ps1`, the Windows setup wizard and `install.sh` add this line for you. You only need this
page if you installed from a package, a portable file or from source, or if you use another shell.
{% endhint %}

## The line for each shell

{% tabs %}
{% tab title="bash" %}
Add to `~/.bashrc` (on macOS, `~/.bash_profile`):

```sh
eval "$(sne env --use-on-cd)"
```
{% endtab %}

{% tab title="zsh" %}
Add to `~/.zshrc`:

```sh
eval "$(sne env --use-on-cd)"
```
{% endtab %}

{% tab title="fish" %}
Add to `~/.config/fish/config.fish`:

```sh
sne env --use-on-cd | source
```
{% endtab %}

{% tab title="PowerShell" %}
Add to the file that `$PROFILE` names:

```powershell
sne env --use-on-cd | Out-String | Invoke-Expression
```
{% endtab %}

{% tab title="cmd" %}
cmd has no profile file. Run this in each session:

```bat
FOR /f "tokens=*" %i IN ('sne env --shell cmd') DO CALL %i
```
{% endtab %}
{% endtabs %}

Open a new terminal afterwards, then run `sne doctor` to check the setup.

## Options

| Option | Effect |
|---|---|
| `--use-on-cd` | Switch versions when you `cd` into a project. Leave it out if you do not want that. See [Switching automatically](../guides/auto-switch.md). |
| `--shell <name>` | Generate code for `bash`, `zsh`, `fish`, `powershell` or `cmd`. sne detects the shell when you leave this out. |

## Git Bash on Windows

Git Bash works with the bash line above. sne writes `PATH` entries in the `/c/...` form that Git Bash
expects.

## Windows PowerShell 5.1

Windows PowerShell 5.1 blocks profile scripts by default. If the setup line does not run, allow local
scripts once:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

## Tab completion

`sne completions <shell>` prints a completion script for `bash`, `zsh`, `fish`, `powershell` or
`elvish`.

{% tabs %}
{% tab title="bash" %}
```sh
sne completions bash > ~/.local/share/bash-completion/completions/sne
```
{% endtab %}

{% tab title="zsh" %}
```sh
sne completions zsh > "${fpath[1]}/_sne"
```
{% endtab %}

{% tab title="fish" %}
```sh
sne completions fish > ~/.config/fish/completions/sne.fish
```
{% endtab %}

{% tab title="PowerShell" %}
```powershell
sne completions powershell | Out-String | Invoke-Expression
```

Add that line to your profile to keep it.
{% endtab %}
{% endtabs %}
