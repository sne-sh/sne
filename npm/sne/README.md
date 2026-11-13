# sne-cli

Simple Node Environment: a fast, cross-platform Node.js version manager. It installs Node.js versions
from nodejs.org and switches between them, per terminal or for every terminal.

```sh
npm install -g sne-cli      # or: yarn global add sne-cli, pnpm add -g sne-cli, bun add -g sne-cli
sne install lts
sne use 22
```

The command is `sne`. This package holds a small launcher for it. The sne program itself comes from a second package for your
system, such as `sne-linux-x64` or `sne-win32-x64`, which your package manager picks by itself.
Nothing is downloaded or run at install time.

## Read this first

A global package lives inside the Node.js version that installed it. If that version is one that sne
manages, switching to another version with `sne use` makes the `sne` command disappear until you
install this package there too. So install it with a Node.js that sne does not manage, or use one of
the [other ways to install](https://github.com/sne-sh/sne#install), which do not need Node.js at all.

The launcher starts Node.js before sne, which adds a little time to every `sne` command.

## More

- [Documentation](https://github.com/sne-sh/sne/tree/main/docs)
- [Source and issues](https://github.com/sne-sh/sne)
- License: MIT
