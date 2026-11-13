# Welcome

**sne** (Simple Node Environment, say "snee") installs Node.js versions and switches between them. It
runs on Windows, Linux and macOS.

```sh
sne install lts
sne use 22
node -v
```

## What makes it different

* **One small native program.** There is no runtime to install first and no shell script to load on
  every prompt.
* **No shims.** `node`, `npm` and `npx` are the real Node.js programs. sne does not sit in front of them.
* **Switching is one filesystem operation.** `sne use` moves a link. Your `PATH` never changes.
* **It reads the files you already have.** `.nvmrc`, `.node-version` and `engines.node` in
  `package.json` all work.
* **Verified installs.** Every download is checked against the SHA-256 published by nodejs.org.
* **No admin rights on Windows.** sne uses directory junctions, so Developer Mode is not needed.

## Where to go next

| If you want to | Read |
|---|---|
| Install sne | [Installation](getting-started/installation.md) |
| Install your first Node.js version | [First steps](getting-started/first-steps.md) |
| Switch versions when you enter a project | [Switching automatically](guides/auto-switch.md) |
| Look up a command | [Commands](reference/commands.md) |
| Fix a problem | [Troubleshooting](help/troubleshooting.md) |

sne is free software under the MIT license. The source is at
[github.com/sne-sh/sne](https://github.com/sne-sh/sne).
