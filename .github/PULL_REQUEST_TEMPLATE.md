## What this changes

<!-- One or two sentences. Link the issue it fixes: "Fixes #123". -->

## Why

<!-- The problem this solves, or the reason for the change. -->

## How I tested it

<!-- The commands you ran and what you saw. -->

Tested on:

- [ ] Windows
- [ ] Linux
- [ ] macOS

## Checklist

- [ ] `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.
- [ ] New behaviour has a test, and a bug fix has a test that fails without the fix.
- [ ] `sne use`, `sne env` and the cd hook still do not use the network. If I changed them, the
      `benches/bench.sh` results from before and after are below.
- [ ] `docs/` and `CHANGELOG.md` are updated for anything a user can see.
