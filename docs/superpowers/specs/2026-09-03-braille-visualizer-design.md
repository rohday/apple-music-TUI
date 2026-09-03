# Design Specification: High-FPS Braille Fluid Ribbon Visualizer

**Date:** 2026-09-03  
**Status:** Approved  
**Author:** Antigravity / Pair Programming  

---

## 1. Executive Summary

Replace the current blocky, slow-updating full-screen visualizer with a non-intrusive, high-FPS (20 FPS / 50ms) Unicode Braille fluid ribbon visualizer embedded directly into the bottom player bar. The user can toggle between compact player bar and expanded visualizer ribbon bar using the `v` key without ever occluding or hiding their music library tables, playlists, or side-by-side lyrics.

---

## 2. Goals & User Experience

1. **Non-Intrusive & Integrated**: The visualizer is part of the player bar at the bottom of the screen. The main view (songs, playlists, queue, lyrics) remains 100% visible and interactive at all times.
2. **Smooth Framerate**: Refresh rate updated to 50ms interval (20 FPS) for smooth, continuous fluid wave motion.
3. **High-Density Braille Graphics**: Uses 2x4 dot Unicode braille cells (`U+2800`..`U+28FF`) to render fine-grained curved audio waves with gradient levels.
4. **Adaptive Playback State**:
   - `Playing`: Active dynamic traveling waves with harmonic interference and peak pulses.
   - `Paused`: Stationary frozen wave.
   - `Stopped`: Calm flat resting baseline (`⠤⠤⠤⠤`).
5. **Dynamic Theme Integration**: Gradients and wave colors match the user's active theme palette (`theme.accent` and `theme.secondary`).

---

## 3. Architecture & Components

### A. Braille Wave Generator (`src/ui/visualizer.rs`)
- High-resolution wave formula:
  $$h(x, t) = \sum_{k=1}^3 A_k \sin\left(\omega_k x + \phi_k t\right) \cdot \cos\left(\psi_k t\right)$$
- Discretized into 4 vertical dot levels per column (8 sub-pixel dots per braille character cell).
- Encodes dot matrix into standard Braille UTF-8 scalar values:
  - Column 1: dots 1, 2, 3, 7 (values 0x1, 0x2, 0x4, 0x40)
  - Column 2: dots 4, 5, 6, 8 (values 0x8, 0x10, 0x20, 0x80)
- Produces a string of braille wave lines across the available terminal width.

### B. Player Bar Integration (`src/ui/player_bar.rs` & `src/ui/mod.rs`)
- In `src/ui/mod.rs`:
  - When `state.show_visualizer` is `false`: Bottom player bar height is 5 (Compact Mode).
  - When `state.show_visualizer` is `true`: Bottom player bar height is 7 (Expanded Ribbon Mode).
  - The main content area smoothly resizes to fit, never disappearing.
- In `src/ui/player_bar.rs`:
  - When expanded, renders the animated braille fluid ribbon on dedicated lines between the track info and the playback controls.
  - In compact mode, shows a subtle inline braille wave indicator next to the playback time.

### C. Event Loop Framerate (`src/main.rs`)
- Adjust UI interval ticker to `Duration::from_millis(50)` (20 FPS) when visualizer or playback is active.
- Configurable default tick rate in `Config::default()` set to `50ms`.

---

## 4. Keybindings
- `v`: Toggle between Compact Player Bar and Expanded Braille Ribbon Visualizer.
- `Esc`: If in Expanded mode and no modal is open, return to Compact mode.

---

## 5. Verification & Testing
- Unit test in `tests/visualizer_test.rs` verifying braille cell generation, valid Unicode ranges, and amplitude clamping.
- Integration test checking toggle behavior between Compact and Expanded player bar states.
- Performance verification ensuring <1% CPU overhead at 20 FPS.
