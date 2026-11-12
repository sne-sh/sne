# Contributing to sne

Thanks for helping. Bug reports, platform fixes and performance work are all welcome.

## Getting started

You need a recent stable [Rust toolchain](https://rustup.rs). Then:

```sh
cargo build
cargo test
```

`cargo test` runs the unit tests and an end-to-end test (`tests/e2e.rs`) that starts a local fake
mirror, so it needs no internet access and installs nothing outside the `target` directory.

To try a build without touching your real setup, point it at a throwaway directory:

```sh
SNE_DIR=/tmp/sne-dev cargo run -- install lts
```

## Before opening a pull request

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same three checks on Windows, Linux and macOS.

## Ground rules

- **The hot path stays fast.** `sne use`, `sne use --auto` and `sne env` run in every shell and on every
  `cd`. They must not touch the network or parse the release index. If you change them, run
  `benches/bench.sh` before and after.
- **All three platforms matter.** Code behind `#[cfg(windows)]` or `#[cfg(not(windows))]` needs its
  counterpart, and the end-to-end test should cover new behaviour on both.
- **Keep dependencies few.** Each one adds to startup time, binary size and build time.

## Layout

| Path | Contents |
|---|---|
| `src/main.rs` | Command-line definition |
| `src/commands/` | One module per group of commands |
| `src/resolve.rs` | Turning `22`, `lts`, ranges into an exact version |
| `src/index.rs` | Release list and its on-disk cache |
| `src/download.rs`, `src/extract.rs` | Streaming download, checksum, unpacking |
| `src/link.rs` | Symlinks and junctions |
| `src/shell.rs` | Setup code for each shell |
| `src/version_file.rs` | `.nvmrc`, `.node-version`, `package.json` |
