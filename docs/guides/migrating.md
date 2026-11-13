# Coming from nvm or fnm

sne reads the same `.nvmrc` and `.node-version` files, so your projects need no changes.

## Commands side by side

| Task | nvm | fnm | sne |
|---|---|---|---|
| Install a version | `nvm install 22` | `fnm install 22` | `sne install 22` |
| Install the newest LTS | `nvm install --lts` | `fnm install --lts` | `sne install lts` |
| Switch this terminal | `nvm use 22` | `fnm use 22` | `sne use 22` |
| Set the default | `nvm alias default 22` | `fnm default 22` | `sne default 22` |
| List installed | `nvm ls` | `fnm ls` | `sne ls` |
| List available | `nvm ls-remote` | `fnm ls-remote` | `sne ls-remote` |
| Run one command | `nvm exec 20 npm test` | `fnm exec --using 20 npm test` | `sne exec 20 -- npm test` |
| Remove a version | `nvm uninstall 18` | `fnm uninstall 18` | `sne uninstall 18` |

## Moving over

1. [Install sne](../getting-started/installation.md).
2. Remove the nvm or fnm lines from your shell profile, so only one tool manages `PATH`.
3. Open a new terminal.
4. Install the versions you use: `sne install 20 22`.
5. Set your default: `sne default 22`.

{% hint style="warning" %}
sne does not import the versions that nvm or fnm installed, and it does not carry over global npm
packages. Install the versions again with sne, then reinstall the global packages you need.
{% endhint %}

## Differences to know

* Each Node.js version has its own global npm packages, as with nvm.
* Leaving a project folder keeps the current version. sne does not switch back to the default.
* On Windows, sne is the same program as on Linux and macOS, with the same commands.
