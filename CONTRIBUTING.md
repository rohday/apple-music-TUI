# Contributing to appleTUI

Thanks for your interest in contributing! This document covers the setup and
the conventions used in this repo.

## Development setup

```bash
git clone https://github.com/rohday/apple-music-TUI.git
cd apple-music-TUI
cargo build
cargo test
appletui --mock   # run without Apple Music credentials
```

Requirements: Linux, a Chromium-based browser, Rust 1.80+.

## Before opening a PR

Run all three gates locally — CI enforces them:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Conventions

- **Architecture**: the UI thread never performs network I/O. Key handling
  mutates `AppState` and enqueues `Job`s (`src/app/job.rs`); background tasks
  execute them and return `Effect`s which are applied by `apply_effect`.
  Keep network calls out of `src/events.rs`.
- **Tests**: pure state logic (`apply_effect`, queue ops, cache, art
  rendering) should be unit-testable without network or terminal. Add tests
  for bug fixes and new state behavior.
- **Errors**: surface failures to the user via `Effect::Error` / status
  messages rather than unwrapping or silently ignoring results.
- **Commits**: follow Conventional Commits (`feat(ui): ...`,
  `fix(playback): ...`).
- **Style**: keep the codebase warning-free; don't introduce new
  dependencies without discussing them in an issue first.

## Reporting bugs

Open an issue with:

1. What you did and what happened
2. Terminal emulator + version
3. Whether the issue reproduces with `appletui --mock`
4. Relevant logs (`RUST_LOG=debug appletui 2>appletui.log`)
