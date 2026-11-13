# Aliases and the default version

## The default version

The default is the version a new terminal starts with.

```sh
sne default 22
```

The first version you install becomes the default. Changing it does not affect terminals that are
already open.

## Aliases

An alias is your own name for a version.

```sh
sne alias work 20       # create or change an alias
sne use work            # use it like a version
sne exec work -- npm test
sne alias               # list all aliases
sne unalias work        # remove one
```

An alias points at one installed version. If you want it to follow newer releases, set it again after
you install them.

## A fixed path for editors and other tools

Tools that do not start from your shell, such as some editors, cannot see the per-terminal version.
Point them at a fixed version instead:

```sh
sne which 22
```

This prints the full path of that version's `node` program.
