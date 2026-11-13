# First steps

This page takes you from a fresh install to a working Node.js. It assumes you have done the
[installation](installation.md) and opened a new terminal.

## 1. Install a Node.js version

```sh
sne install lts
```

This downloads the newest LTS release from nodejs.org, checks it, and unpacks it. The first version
you install becomes the default.

You can install several at once. They download in parallel:

```sh
sne install 20 22 24
```

## 2. Use it

```sh
node -v
npm -v
```

## 3. Switch versions

```sh
sne use 20
node -v
```

`sne use` changes the current terminal only. Other terminals keep their own version.

## 4. Choose the version for new terminals

```sh
sne default 22
```

## 5. See what you have

```sh
sne ls                # installed versions
sne current           # the version active in this terminal
sne ls-remote --lts   # versions you can install
```

## 6. Work on a project

If the project has an `.nvmrc`, a `.node-version` or an `engines.node` field in `package.json`, you do
not need to name a version:

```sh
cd my-project
sne install   # installs the version the project asks for
sne use       # switches to it
```

With `--use-on-cd` in your [setup line](shell-setup.md), the switch happens by itself when you enter
the folder.

## Next

* [Project version files](../guides/version-files.md)
* [Commands](../reference/commands.md)
