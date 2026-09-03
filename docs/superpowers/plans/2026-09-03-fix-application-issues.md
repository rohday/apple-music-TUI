# Fix All Application Issues Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all critical, important, and minor issues identified during the comprehensive code review of `appleTUI`, ensuring process isolation, security, table scrolling, complete feature implementation, resilient playback, and clean lints.

**Architecture:** 
1. `PlaybackEngine` lifecycle management with child process tracking, `Drop` cleanup, signal handling (`SIGINT`, `SIGTERM`, `SIGHUP`), and removing `--no-sandbox`.
2. Atomic configuration & credential persistence with `0o600` creation permissions.
3. Ratatui viewport scrolling calculation in `main_view.rs` keeping the cursor visible in all list/table views.
4. Active authentication flow: show `AuthPrompt` modal when unauthenticated and handle `L` key to trigger interactive login.
5. Missing spec features: playlist track deletion (`d` / `Delete`), album/artist drill-down, queue accessibility.
6. Playback engine resilience: handle MusicKit errors, auto-advance on track completion, and non-blocking view data loading.
7. Test suite robustness and clippy cleanup across the codebase.

**Tech Stack:** Rust (edition 2021), `tokio`, `ratatui`, `crossterm`, `reqwest`, `chromiumoxide`, `serde`, `serde_json`.

## Global Constraints
- Target platform: Linux (x86_64).
- Zero dangling browser processes on exit or failure.
- Strict panic hooks and signal handlers to ensure terminal restoration.
- All existing and new tests must pass (`cargo test --all-targets`).
- Zero clippy warnings with `cargo clippy --all-targets`.

---

### Task 1: Process Supervision, Signal Handling & Security Hardening

**Files:**
- Modify: `src/playback/engine.rs`
- Modify: `src/main.rs`
- Test: `tests/playback_test.rs`

- [ ] **Step 1: Write test verifying PlaybackCommand::Stop does not permanently break engine**
- [ ] **Step 2: Remove `--no-sandbox` from browser startup arguments in `src/playback/engine.rs`**
- [ ] **Step 3: Implement process supervision & Drop guard on PlaybackEngine to kill browser child process on drop**
- [ ] **Step 4: Fix PlaybackCommand::Stop to pause and reset instead of breaking the loop and closing the browser**
- [ ] **Step 5: Add Unix signal handling for SIGINT, SIGTERM, SIGHUP in `src/main.rs`**
- [ ] **Step 6: Run tests to verify Task 1 passes**

---

### Task 2: Insecure Permissions Window & Atomic Credential File Storage

**Files:**
- Modify: `src/config.rs`
- Test: `tests/config_test.rs`

- [ ] **Step 1: Write test for atomic file creation and 0600 permissions in `tests/config_test.rs`**
- [ ] **Step 2: Update `AuthConfig::save_to` and `Config::save_to` to write to tempfile and atomically rename with 0600 permissions**
- [ ] **Step 3: Fix `clean_stale_browser_locks` to avoid race condition when browser is active**
- [ ] **Step 4: Fix `test_find_browser_binary` in `tests/config_test.rs` to not assert hard-coded machine dependency**
- [ ] **Step 5: Run `cargo test --test config_test`**

---

### Task 3: Ratatui Main View Viewport Scrolling & Modal Polish

**Files:**
- Modify: `src/ui/main_view.rs`
- Modify: `src/ui/modals.rs`
- Modify: `src/ui/player_bar.rs`
- Test: `tests/ui_render_test.rs`

- [ ] **Step 1: Write test in `tests/ui_render_test.rs` testing large list rendering with selected index > terminal height**
- [ ] **Step 2: Implement viewport windowing/scrolling in `src/ui/main_view.rs` for songs, playlists, albums, and artists**
- [ ] **Step 3: Improve `src/ui/modals.rs` to render responsive help overlay and scrollable playlist selector**
- [ ] **Step 4: Adjust `src/ui/player_bar.rs` controls line so it fits within 80 columns without clipping volume**
- [ ] **Step 5: Run `cargo test --test ui_render_test`**

---

### Task 4: Authentication State Machine & Interactive Login from TUI

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/main.rs`
- Modify: `src/events.rs`
- Modify: `src/auth/login.rs`
- Test: `tests/state_test.rs`

- [ ] **Step 1: Add unit test in `tests/state_test.rs` for AuthPrompt modal state and login triggers**
- [ ] **Step 2: In `main.rs`, trigger `ModalState::AuthPrompt` on startup if `!is_auth`**
- [ ] **Step 3: In `events.rs`, handle `KeyCode::Char('l')` / `KeyCode::Char('L')` to launch interactive login from TUI**
- [ ] **Step 4: In `auth/login.rs`, improve regex/extraction for JWT developer token in web bundle**
- [ ] **Step 5: Run `cargo test --test state_test`**

---

### Task 5: Complete Missing Spec Features: Playlist Track Deletion, Drill-down, Queue

**Files:**
- Modify: `src/api/client.rs`
- Modify: `src/events.rs`
- Modify: `src/app/state.rs`
- Modify: `src/ui/sidebar.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write test for playlist track deletion and queue navigation in `tests/integration_test.rs`**
- [ ] **Step 2: Add `delete_playlist_track` method to `AppleMusicClient`**
- [ ] **Step 3: Bind `d` / `Delete` in `events.rs` to delete selected song in `PlaylistDetail`**
- [ ] **Step 4: Add `ActiveView::Queue` to sidebar or key navigation**
- [ ] **Step 5: Implement album and artist Enter action to load and display their tracks**
- [ ] **Step 6: Run `cargo test --test integration_test`**

---

### Task 6: Playback Engine Resilience, MusicKit Error Handling & API Error Codes

**Files:**
- Modify: `src/playback/engine.rs`
- Modify: `src/api/client.rs`
- Test: `tests/playback_test.rs`
- Test: `tests/client_test.rs`

- [ ] **Step 1: Write test for API error code inspection and playback error handling**
- [ ] **Step 2: Check evaluate result in `run_browser_playback_loop` and propagate errors**
- [ ] **Step 3: Add track completion detection to auto-advance queue in browser mode**
- [ ] **Step 4: Enhance API client to check `resp.status().is_success()` and return informative errors**
- [ ] **Step 5: Run `cargo test`**

---

### Task 7: Clippy Warnings, Format Strings, and Full Verification

**Files:**
- Modify: various files as reported by clippy
- Run: `cargo clippy --all-targets`
- Run: `cargo test --all-targets`

- [ ] **Step 1: Fix unused async on `PlaybackEngine::new`**
- [ ] **Step 2: Fix uninlined format args and `drop(_guard)`**
- [ ] **Step 3: Fix float comparisons in test files**
- [ ] **Step 4: Run full verification suite (`cargo check`, `cargo test`, `cargo clippy`)**
