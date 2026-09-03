# Feature Specifications: Themes, Visualizer, Fuzzy Filter, Radio Station, Synced Lyrics & MPRIS2

**Status:** Approved design ready for phased execution  
**Target:** apple-music-TUI (`appleTUI`)

---

## 1. Overview & Architecture

We will implement 6 distinct quality-of-life and visual features, delivered, tested, committed, and pushed **one by one**:

1. **Theme Engine (`t`)**: Dynamic multi-theme color palettes (Apple Dark, Catppuccin Mocha, Tokyo Night, Gruvbox Dark, Nord) persisted in `config.json`.
2. **Live Spectrum Visualizer (`v`)**: Real-time animated Unicode equalizer bars (` ▂▃▄▅▆▇█`) in the player bar and a dedicated visualizer mode.
3. **In-View Fuzzy Filtering (`f`)**: Interactive in-memory search bar for the active table view (Songs, Albums, Artists, Playlists, Queue) with instant filtering.
4. **Track Radio / Station (`R`)**: Apple Music continuous station creation from any selected song, auto-populating the queue.
5. **Synced Side-by-Side Lyrics Panel (`y`)**: Split-panel main layout with time-synced lyrics fetched from LRCLIB, auto-scrolling line-by-line with playback.
6. **MPRIS2 / D-Bus Desktop Integration**: Full `playerctl` and media-key control via D-Bus session bus with real-time metadata updates.

---

## 2. Feature Details

### Feature 1: Theme Engine
- **Module:** `src/ui/theme.rs`, `src/config.rs`, `src/app/state.rs`, `src/events.rs`.
- **Themes:**
  - `AppleDark`: Signature Apple Music Red (`#FA2D48`), Dark background.
  - `CatppuccinMocha`: Mauve (`#CBA6F7`), Peach (`#FAB387`), Sapphire (`#74C7EC`), Crust (`#11111B`).
  - `TokyoNight`: Electric Blue (`#7DCFFF`), Magenta (`#BB9AF7`), Deep Night (`#1A1B26`).
  - `GruvboxDark`: Warm Orange (`#FE8019`), Aqua (`#8EC07C`), Dark Umber (`#282828`).
  - `Nord`: Arctic Frost (`#88C0D0`), Snow Storm (`#ECEFF4`), Polar Night (`#2E3440`).
- **Keybinding:** `t` cycles through themes, updates `state.theme`, saves to `config.json`, and displays a brief status toast.

### Feature 2: Live Spectrum Visualizer
- **Module:** `src/ui/player_bar.rs`, `src/app/state.rs`, `src/events.rs`.
- **Compact Visualizer:** Rendered inside the player bar when track is `Playing`, animating 12 frequency bands using deterministic multi-frequency sine wave calculations driven by `current_time_secs`.
- **Fullscreen Visualizer (`v`):** Toggles a full visualizer view with responsive height bars and stereo channel separation. Freezes on `Pause`, clears on `Stop`.

### Feature 3: In-View Fuzzy Filtering
- **Module:** `src/ui/main_view.rs`, `src/app/state.rs`, `src/events.rs`.
- **Keybinding:** `f` enters filter input mode for the current view.
- **Behavior:**
  - Bottom input bar: `Filter: <query>`.
  - Dynamically filters the items displayed in `LibrarySongs`, `PlaylistDetail`, `RecentlyPlayed`, `LibraryAlbums`, `LibraryArtists`, and `Queue`.
  - `Enter` confirms the selection and exits filter mode.
  - `Esc` clears the filter and returns to the full list.
  - Arrow keys / `j` / `k` navigate the filtered list.

### Feature 4: Track Radio / Station
- **Module:** `src/api/client.rs`, `src/events.rs`, `src/app/state.rs`.
- **Keybinding:** `R` (Shift+r) on any selected song.
- **Behavior:**
  - In live mode: Calls Apple Music API to retrieve station / track recommendations based on the song.
  - In mock mode: Seeds a 15-track radio queue of similar tracks.
  - Automatically loads tracks into `state.queue`, starts playing the first track, switches to `ActiveView::Queue`, and sets status to `Started Station for '<Song>'`.

### Feature 5: Synced Side-by-Side Lyrics Panel
- **Module:** `src/api/lyrics.rs`, `src/ui/layout.rs`, `src/ui/lyrics.rs`, `src/app/state.rs`.
- **Keybinding:** `y` toggles the lyrics side panel.
- **Layout:** When enabled, the main content area splits into 2 columns:
  - Left column (60%): Main table view (songs, playlists, etc.).
  - Right column (40%): Dedicated scrolling lyrics panel.
- **Service:** Queries `https://lrclib.net/api/get` with track name, artist name, and duration.
- **Syncing:** Parses `[mm:ss.xx]` timestamps. Finds the active lyric line for `playback.current_time_secs`, vertically centers it in bold theme accent color, and dims past/future lines.

### Feature 6: MPRIS2 / D-Bus Desktop Integration
- **Module:** `src/playback/mpris.rs`, `src/main.rs`.
- **Crate:** `mpris-server` (with `tokio` feature).
- **Interface:** `org.mpris.MediaPlayer2.appleTUI`.
- **Actions:** Maps D-Bus calls for `Play`, `Pause`, `PlayPause`, `Next`, `Previous`, `Stop`, `Seek`, `SetPosition`, `Volume` directly into `PlaybackEngine` commands.
- **Signals:** Broadcasts metadata (`mpris:trackid`, `xesam:title`, `xesam:artist`, `xesam:album`, `mpris:length`) and `PlaybackStatus` on track changes or state transitions.
- **Resilience:** Gracefully disabled if D-Bus session is unavailable (e.g. headless environments).

---

## 3. Verification & Phasing
Each feature will be:
1. Implemented with unit/integration tests.
2. Verified with `cargo check`, `cargo test --all-targets`, and `cargo clippy --all-targets`.
3. Committed and pushed to `origin/main` individually before beginning the next feature.
