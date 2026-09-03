# High-FPS Braille Fluid Ribbon Visualizer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the intrusive, slow-updating full-screen visualizer with an elegant, non-intrusive 20 FPS Unicode Braille fluid ribbon visualizer embedded directly into the bottom player bar.

**Architecture:** A mathematical Braille rasterizer computes smooth multi-harmonic traveling waves mapped to 2x4 dot-matrix Braille cells (`U+2800`..`U+28FF`). The player bar dynamically expands its height from 5 to 7 when toggled with `v`, embedding the animated wave ribbon while keeping all music tables, playlists, and lyrics completely visible. The event loop tick rate is boosted to 50ms (20 FPS) for fluid motion.

**Tech Stack:** Rust, Ratatui, Crossterm, Unicode Braille Patterns (`0x2800`..`0x28FF`).

## Global Constraints
- Main view (library songs, playlists, albums, artists, queue, side-by-side lyrics) must remain visible and fully interactive when visualizer is enabled.
- Frame rate must run at 20 FPS (50ms interval) with <1% CPU overhead.
- No blocky block characters (` ▂▃▄`); use fine-grained 2x4 dot Braille cells.
- Palette colors must dynamically adapt to the active `Theme`.

---

### Task 1: Braille Wave Rasterizer & Unit Tests

**Files:**
- Create/Modify: `src/ui/visualizer.rs`
- Test: `tests/visualizer_test.rs`

**Interfaces:**
- Produces:
  - `pub fn render_braille_ribbon(width: usize, time_secs: f64, is_playing: bool) -> (String, String)`
  - `pub fn braille_cell_from_levels(left_dot_height: usize, right_dot_height: usize) -> char`

- [ ] **Step 1: Write unit tests in `tests/visualizer_test.rs`**
  - Test that `braille_cell_from_levels(0, 0)` returns blank braille `⠀` (`\u{2800}`) or baseline dot.
  - Test that heights 1..=4 produce proper dot configurations.
  - Test that `render_braille_ribbon(width, time_secs, is_playing)` generates two non-empty lines with exact column count matching `width`.
  - Test that when `is_playing` is false, it returns flat resting baseline `⠤⠤⠤`.

- [ ] **Step 2: Run `cargo test --test visualizer_test` to verify failure**

- [ ] **Step 3: Implement Braille wave rasterizer in `src/ui/visualizer.rs`**
  - Implement mathematical wave functions with 3 harmonic sine/cosine frequencies and traveling phase.
  - Map column values into 2x4 braille dot positions:
    - Left dots: 1, 2, 3, 7 (values 0x1, 0x2, 0x4, 0x40)
    - Right dots: 4, 5, 6, 8 (values 0x8, 0x10, 0x20, 0x80)
  - Generate dual-line ribbon lines for expanded rendering.

- [ ] **Step 4: Run `cargo test --test visualizer_test` to verify pass**

- [ ] **Step 5: Commit Task 1**

---

### Task 2: Embed Expanded Braille Ribbon in Player Bar & Layout Resizing

**Files:**
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/player_bar.rs`
- Modify: `src/events.rs`
- Test: `tests/ui_render_test.rs`

**Interfaces:**
- Consumes: `render_braille_ribbon` from `src/ui/visualizer.rs`
- Produces:
  - Dynamic player bar layout adapting between compact (height 5) and expanded visualizer ribbon (height 7).

- [ ] **Step 1: Update `src/ui/mod.rs` main layout**
  - Set player bar height to `if state.show_visualizer { 7 } else { 5 }`.
  - Remove fullscreen visualizer replacement; keep `main_view` and `lyrics` active and rendered.

- [ ] **Step 2: Update `src/ui/player_bar.rs`**
  - In expanded mode (height 7), render:
    - Row 0: Track Title, Artist, Album, and animated status
    - Row 1-2: 2-line Braille fluid ribbon styled with `theme.accent` and `theme.secondary`
    - Row 3: Progress bar with elapsed / duration
    - Row 4: Controls (Shuffle, Repeat, Volume)
  - In compact mode (height 5), render smooth inline braille wave indicator next to track info.

- [ ] **Step 3: Update `src/events.rs`**
  - `v`: Toggle `state.show_visualizer` between compact and expanded player bar with status toast:
    - "Ribbon Visualizer: Enabled (Player bar expanded)"
    - "Ribbon Visualizer: Compact"
  - `Esc`: If `state.show_visualizer`, reset to compact mode.

- [ ] **Step 4: Update `tests/ui_render_test.rs`**
  - Verify rendering both compact and expanded states without panic or boundary overflow.

- [ ] **Step 5: Run all UI tests**

- [ ] **Step 6: Commit Task 2**

---

### Task 3: 50ms High-FPS Event Loop & Verification

**Files:**
- Modify: `src/config.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Update default `tick_rate_ms` in `src/config.rs`**
  - Change default `tick_rate_ms` from 250 to 50 for 20 FPS animation.

- [ ] **Step 2: Update `src/main.rs` ticker interval**
  - Ensure ticker uses `Duration::from_millis(config.tick_rate_ms.min(50))` when playback is active or visualizer is expanded.

- [ ] **Step 3: Run full test suite & clippy**
  - `cargo test --all-targets`
  - `cargo clippy --all-targets`

- [ ] **Step 4: Recompile and install binary**
  - `cargo install --path .`

- [ ] **Step 5: Commit and push Task 3**
