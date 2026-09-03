# Advanced Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 6 user-requested features: Multi-palette Theme Engine (`t`), Live Spectrum Visualizer (`v`), In-View Fuzzy Filtering (`f`), Track Radio/Station (`R`), Synced Side-by-Side Lyrics Panel (`y`), and MPRIS2 / D-Bus Media Key Integration (`playerctl`). Each feature will be tested, verified, committed, and pushed individually.

**Architecture:**
- `src/ui/theme.rs`: Dynamic `ThemePreset` with 5 colorways (Apple Dark, Catppuccin Mocha, Tokyo Night, Gruvbox Dark, Nord).
- `src/ui/player_bar.rs` & `src/ui/visualizer.rs`: Procedural multi-frequency waveform equalizer using Unicode blocks.
- `src/app/state.rs` & `src/events.rs`: In-view fuzzy filter buffer and filter matcher for table rows.
- `src/api/client.rs`: Station creation endpoint and auto-queue generation.
- `src/api/lyrics.rs` & `src/ui/lyrics.rs`: LRCLIB timestamped lyrics client and side-by-side 60/40 layout split.
- `src/playback/mpris.rs`: D-Bus MPRIS2 server forwarding hardware media keys and publishing track metadata.

**Tech Stack:** Rust 2021, `ratatui 0.29`, `crossterm 0.28`, `tokio 1.43`, `reqwest`, `mpris-server 0.10`.

---

### Task 1: Theme Engine (`t` key)

**Files:**
- Modify: `src/ui/theme.rs`
- Modify: `src/config.rs`
- Modify: `src/app/state.rs`
- Modify: `src/events.rs`
- Modify: `src/ui/modals.rs`
- Modify: `src/ui/player_bar.rs`
- Modify: `src/ui/main_view.rs`
- Modify: `src/ui/sidebar.rs`
- Test: `tests/theme_test.rs`

- [ ] **Step 1: Write unit test in `tests/theme_test.rs` for theme switching, serialization, and color palettes**
- [ ] **Step 2: Define `ThemePreset` enum and dynamic `Theme` in `src/ui/theme.rs`**
- [ ] **Step 3: Update `Config` and `AppState` with `theme: ThemePreset`**
- [ ] **Step 4: Update UI modules (`main_view.rs`, `player_bar.rs`, `sidebar.rs`, `modals.rs`) to use dynamic theme from state**
- [ ] **Step 5: Bind `t` in `src/events.rs` to cycle themes, update state, save config, and toast notification**
- [ ] **Step 6: Run `cargo test --test theme_test` and `cargo clippy --all-targets`**
- [ ] **Step 7: Commit and push Task 1 (`git commit -m "feat(ui): add dynamic multi-palette theme engine with 't' key shortcut" && git push origin main`)**

---

### Task 2: Live Spectrum Visualizer (`v` key)

**Files:**
- Create: `src/ui/visualizer.rs`
- Modify: `src/ui/player_bar.rs`
- Modify: `src/app/state.rs`
- Modify: `src/events.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/visualizer_test.rs`

- [ ] **Step 1: Write test for equalizer frequency bar calculation in `tests/visualizer_test.rs`**
- [ ] **Step 2: Implement waveform algorithm in `src/ui/visualizer.rs` producing Unicode block heights**
- [ ] **Step 3: Add animated visualizer widget into `src/ui/player_bar.rs` when track is Playing**
- [ ] **Step 4: Add `show_visualizer` flag to `AppState` and bind `v` in `src/events.rs` to toggle visualizer view**
- [ ] **Step 5: Run `cargo test --test visualizer_test` and `cargo clippy --all-targets`**
- [ ] **Step 6: Commit and push Task 2 (`git commit -m "feat(ui): add live spectrum visualizer to player bar and full visualizer view" && git push origin main`)**

---

### Task 3: In-View Fuzzy Filtering (`f` key)

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/events.rs`
- Modify: `src/ui/main_view.rs`
- Modify: `src/ui/modals.rs`
- Test: `tests/filter_test.rs`

- [ ] **Step 1: Write test in `tests/filter_test.rs` verifying in-view song and playlist filtering**
- [ ] **Step 2: Add `filter_query: String` and `is_filtering: bool` to `AppState`**
- [ ] **Step 3: Update `src/ui/main_view.rs` to filter visible rows using query and render bottom filter bar when active**
- [ ] **Step 4: Bind `f` to enter filtering, `Esc` to clear/exit, typing to update query in `src/events.rs`**
- [ ] **Step 5: Run `cargo test --test filter_test` and `cargo clippy --all-targets`**
- [ ] **Step 6: Commit and push Task 3 (`git commit -m "feat(navigation): add in-view live fuzzy filtering with 'f' shortcut" && git push origin main`)**

---

### Task 4: Track Radio / Station (`R` key)

**Files:**
- Modify: `src/api/client.rs`
- Modify: `src/events.rs`
- Modify: `src/app/state.rs`
- Test: `tests/station_test.rs`

- [ ] **Step 1: Write test for station creation in `tests/station_test.rs`**
- [ ] **Step 2: Add `create_station_for_song` to `AppleMusicClient` (calling station endpoint or mock generation)**
- [ ] **Step 3: In `src/events.rs`, bind `R` (Shift+r) to fetch station tracks, replace queue, start playback, and switch view**
- [ ] **Step 4: Run `cargo test --test station_test` and `cargo clippy --all-targets`**
- [ ] **Step 5: Commit and push Task 4 (`git commit -m "feat(playback): add track station creation and autoplay with 'R' shortcut" && git push origin main`)**

---

### Task 5: Synced Side-by-Side Lyrics Panel (`y` key)

**Files:**
- Create: `src/api/lyrics.rs`
- Create: `src/ui/lyrics.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app/state.rs`
- Modify: `src/events.rs`
- Test: `tests/lyrics_test.rs`

- [ ] **Step 1: Write test in `tests/lyrics_test.rs` parsing LRC timecode format (`[mm:ss.xx]`) and matching active line**
- [ ] **Step 2: Create `src/api/lyrics.rs` to fetch synced lyrics from LRCLIB API**
- [ ] **Step 3: Create `src/ui/lyrics.rs` rendering current, past, and upcoming lyrics with auto-centering**
- [ ] **Step 4: Update `src/ui/mod.rs` layout: when `state.show_lyrics` is true, split main area 60% table / 40% lyrics**
- [ ] **Step 5: Bind `y` in `src/events.rs` to toggle side-by-side lyrics and trigger async fetch for current song**
- [ ] **Step 6: Run `cargo test --test lyrics_test` and `cargo clippy --all-targets`**
- [ ] **Step 7: Commit and push Task 5 (`git commit -m "feat(ui): add side-by-side synced lyrics panel with 'y' shortcut" && git push origin main`)**

---

### Task 6: MPRIS2 / D-Bus Desktop Integration (`playerctl`)

**Files:**
- Modify: `Cargo.toml` (add `mpris-server = { version = "0.10", features = ["tokio"] }`)
- Create: `src/playback/mpris.rs`
- Modify: `src/main.rs`
- Modify: `src/playback/mod.rs`
- Test: `tests/mpris_test.rs`

- [ ] **Step 1: Add `mpris-server` to `Cargo.toml`**
- [ ] **Step 2: Write test in `tests/mpris_test.rs` verifying MPRIS event conversion to `PlaybackCommand`**
- [ ] **Step 3: Implement `src/playback/mpris.rs` implementing `RootInterface` and `PlayerInterface`**
- [ ] **Step 4: Spawn MPRIS background task in `src/main.rs` bridging D-Bus calls to `playback` command sender**
- [ ] **Step 5: Run `cargo test` and verify `playerctl` communication**
- [ ] **Step 6: Commit and push Task 6 (`git commit -m "feat(desktop): add MPRIS2 D-Bus integration for media keys and playerctl" && git push origin main`)**
