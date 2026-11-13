# How it works

## Switching

`sne env` runs once when a terminal starts. It does two things:

1. Creates a link for that terminal, inside sne's data folder, that points at the default version.
2. Prints shell code that puts the link on your `PATH`.

From then on, `sne use` only re-points the link. `PATH` does not change again, so a switch is a single
filesystem operation and takes effect at once for `node`, `npm`, `npx` and anything else in that
version's folder.

```
PATH  ->  shells/<terminal>  ->  versions/v22.11.0
                              (sne use 20 moves this arrow)
```

The link is a symbolic link on Linux and macOS and a directory junction on Windows. Junctions need
neither administrator rights nor Developer Mode.

Each terminal has its own link, so two terminals can use two versions at the same time.

## Why there are no shims

Some version managers put a small program named `node` first on `PATH`. Every time you run `node`,
that program works out which version to use and then starts the real one. This costs time on every
call, and more on Windows, where starting a program is slow.

sne has no such program. The `node` on your `PATH` is the real Node.js.

## Installing

1. sne reads the list of releases from nodejs.org. The list is cached for one hour and downloaded
   again only when the server says it changed.
2. It downloads the archive for your system and hashes it while it arrives.
3. On Linux and macOS, the archive is decompressed and unpacked as it downloads and is never written to
   disk. On Windows, the zip is saved to a temporary file and unpacked on several threads.
4. The files go into a temporary folder. Only after the SHA-256 matches the one in the server's
   `SHASUMS256.txt` is the folder renamed into place.

Because of the last step, a cancelled or failed install never leaves a half-installed version.

## What runs on every prompt

Nothing. With `--use-on-cd`, a small hook runs when the folder changes. It reads the project's
[version file](../guides/version-files.md), never uses the network, and exits.
