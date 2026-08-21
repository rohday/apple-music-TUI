# Apple Music TUI (`appleTUI`) Design Document

**Date:** 2026-08-21  
**Status:** Approved  
**Author:** Antigravity  

---

## 1. Objective & Scope

Build a lightweight, high-performance, and responsive Terminal User Interface (TUI) client for Apple Music on Linux. The application prioritizes speed, resource efficiency, and zero background resource consumption when not in use.

### Goals
- **Real-time audio streaming** on Linux with native audio output (PipeWire / PulseAudio).
- **Search & Catalog Exploration**: Search songs, albums, artists, and playlists with instant filtering.
- **Library Management**: Browse personal library tracks, albums, artists, recently played history, and playlists.
- **Playlist Management**: View playlist details, create new playlists, add songs to playlists, and remove tracks.
- **Full Playback Controls**: Play, pause, skip forward/backward, seek, adjust volume, toggle shuffle and repeat.
- **Self-contained Process Lifecycle**: Headless browser instance is only spawned when the TUI is running and is terminated immediately on exit (including SIGINT/SIGTERM handling) with no dangling processes.
- **Interactive First-Time Authentication**: Automated/assisted login flow capturing the necessary Developer Token and Music User Token (MUT) without requiring a paid $99/yr Apple Developer account.
- **High Performance & Low Latency**: Written in Rust using Ratatui, Tokio async runtime, and efficient event loop polling.

### Non-Goals
- Local audio file downloading or offline DRM stripping.
- Time-synced animated lyrics or heavy video playback.
- Running persistent background daemons when the TUI application is closed.

---

## 2. Architecture Overview

`appleTUI` is structured into modular async subsystems running on top of Tokio:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                appleTUI                                 │
│                                                                         │
│  ┌──────────────┐      ┌─────────────────┐      ┌────────────────────┐  │
│  │   UI Layer   │      │   API Client    │      │  Playback Engine   │  │
│  │  (Ratatui +  │◄────►│ (reqwest + REST │◄────►│ (chromiumoxide +   │  │
│  │  Crossterm)  │      │  Apple Music)   │      │  Headless Browser) │  │
│  └──────┬───────┘      └────────┬────────┘      └─────────┬──────────┘  │
│         │                       │                         │             │
│         ▼                       ▼                         ▼             │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │               App State & Action Dispatch Channel                 │  │
│  │            (Arc<Mutex<AppState>> / tokio::sync::mpsc)             │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ (Lifecycle Bound)
                                     ▼
                     [Headless Chromium / Brave Instance]
                                     │ (Audio Stream via EME / Widevine)
                                     ▼
                          [PipeWire / PulseAudio]
```

### Key Subsystems:
1. **Core / App State**: Manages current focus, selected views, navigation history, search query, active tracks, queue, volume, notifications, and modal popups.
2. **Auth & Config Manager**: Manages `~/.config/appletui/config.json` and `auth.json`. Detects browser binaries (`brave-browser`, `google-chrome`, `chromium`), launches login window on demand, extracts tokens, and checks token validity.
3. **Apple Music REST API Client**: Fast async HTTP client interacting with `https://api.music.apple.com/v1/` for storefront resolution, catalog searches, user library queries, and playlist modifications.
4. **Playback Engine & Browser Controller**: Manages child browser process lifecycle with strict process group isolation. Connects via Chrome DevTools Protocol (CDP), injects/controls MusicKit JS web player instance, evaluates playback commands (`play`, `pause`, `seekToTime`, `setQueue`), and polls playback status.
5. **TUI View & Input Handler**: Ratatui rendering engine, handling key events, terminal resizing, panel navigation, modals, and drawing frames at up to 60 FPS (with idle throttling).

---

## 3. Detailed Component Specifications

### 3.1 Authentication & Token Capture
- **Developer Token**: Extracted dynamically from Apple Music Web Player client scripts or static fallback, valid for catalog queries.
- **Music User Token (MUT)**: Extracted from user's `media-user-token` cookie after logging into Apple ID.
- **Storage**: Saved in user configuration directory: `$XDG_CONFIG_HOME/appletui/auth.json` (or `~/.config/appletui/auth.json`) with file mode `0600`.
- **Login Flow**:
  1. If no tokens exist or auth fails with HTTP 401, TUI prompts user to log in (`Press [L] to Login`).
  2. Spawns browser in visible mode pointing to `https://music.apple.com/login`.
  3. Uses CDP network/cookie listener to detect successful auth and extracts `media-user-token`.
  4. Automatically closes the login browser window, stores credentials, and initializes playback engine in headless mode.

### 3.2 Apple Music REST API Client
Base URL: `https://api.music.apple.com/v1`

Headers:
- `Authorization: Bearer <DEVELOPER_TOKEN>`
- `Music-User-Token: <USER_TOKEN>`
- `Origin: https://music.apple.com`
- `Referer: https://music.apple.com/`

Endpoints implemented:
- `GET /v1/me/storefront`: Fetches subscriber storefront (e.g. `us`, `in`, `gb`).
- `GET /v1/catalog/{storefront}/search`: Catalog search for songs, albums, artists, playlists.
- `GET /v1/me/library/songs`: Paginated library songs.
- `GET /v1/me/library/albums`: Paginated library albums.
- `GET /v1/me/library/artists`: Paginated library artists.
- `GET /v1/me/library/playlists`: User's created and saved playlists.
- `GET /v1/me/library/playlists/{id}/tracks`: Tracks inside a user playlist.
- `POST /v1/me/library/playlists`: Create new playlist.
- `POST /v1/me/library/playlists/{id}/tracks`: Append song to playlist.
- `DELETE /v1/me/library/playlists/{id}/tracks/{track_index}` or library remove: Remove track.
- `GET /v1/me/recent/played/tracks`: User's recently played track list.

### 3.3 Playback Engine (CDP + MusicKit JS)
- **Browser Binary Detection**: Probes `CHROME_BIN` environment variable, then searches system paths for `google-chrome-stable`, `google-chrome`, `chromium`, `chromium-browser`, `brave-browser`, `/usr/bin/brave-browser`.
- **Browser Launch Arguments**:
  - `--headless=new` (or `--headless` depending on version)
  - `--remote-debugging-port=0` (dynamic free port allocation)
  - `--autoplay-policy=no-user-gesture-required`
  - `--disable-gpu` (optional, based on config)
  - `--enable-widevine-cdm`
  - `--user-data-dir=<cache_dir>/appletui/chrome_profile`
- **Process Supervision**:
  - Spawns child process in isolated process group.
  - Registers `tokio::signal::unix` listeners for `SIGINT`, `SIGTERM`, `SIGHUP` and an explicit `Drop` guard on `PlaybackEngine` that terminates the child process and waits for exit.
- **MusicKit Control Script**:
  - Navigates to a lightweight wrapper page or `https://music.apple.com`.
  - Injects JS to control the global MusicKit instance:
    - `window.MusicKit.getInstance().play()`
    - `window.MusicKit.getInstance().pause()`
    - `window.MusicKit.getInstance().stop()`
    - `window.MusicKit.getInstance().seekToTime(seconds)`
    - `window.MusicKit.getInstance().volume = vol`
    - `window.MusicKit.getInstance().setQueue({ songs: [songId] })`
    - `window.MusicKit.getInstance().skipToNextItem()`
    - `window.MusicKit.getInstance().skipToPreviousItem()`
    - `window.MusicKit.getInstance().shuffleMode = mode`
    - `window.MusicKit.getInstance().repeatMode = mode`

### 3.4 Terminal User Interface (Ratatui)

**Layout Structure:**
```
┌────────────────────────────────────────────────────────────────────────┐
│ appleTUI v0.1.0                     [Status: Connected] Storefront: US │
├───────────────────┬────────────────────────────────────────────────────┤
│ 1. Search         │ Title                      Artist          Duration│
│ 2. Library Songs  │ ────────────────────────────────────────────────── │
│ 3. Albums         │ ▶ Starboy                  The Weeknd          3:50│
│ 4. Artists        │   Blinding Lights          The Weeknd          3:20│
│ 5. Playlists      │   Save Your Tears          The Weeknd          3:35│
│    - Synthwave    │   Die For You              The Weeknd          4:20│
│    - Workout      │   After Hours              The Weeknd          6:01│
│ 6. Recent Played  │                                                    │
├───────────────────┴────────────────────────────────────────────────────┤
│ ▶ Starboy · The Weeknd — Starboy                       1:45 / 3:50     │
│ [⏮ Prev] [▶/⏸ Space] [⏭ Next]  [🔀 Shuffle: Off] [🔁 Repeat: Off]  Vol: 80% │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━●────────────────────────────                │
└────────────────────────────────────────────────────────────────────────┘
```

**Panels:**
- **Sidebar (Left, ~22% width)**: Primary navigation tree (Search, Library Songs, Albums, Artists, Playlists, Recently Played).
- **Main Content (Center/Right, ~78% width)**: Active list / table view with column formatting, headers, active selection pointer, and playback indicator (`▶`).
- **Bottom Player Bar (Bottom, 4 lines)**: Now playing track title, artist, album, elapsed & total time gauge bar, playback state, shuffle/repeat indicators, volume level.

**Modals & Overlays:**
- **Search Bar Popup**: Activated via `/`, debounced query execution.
- **Playlist Creator Popup**: Activated via `c` when on playlists view.
- **Add to Playlist Selector**: Activated via `a` on any selected track.
- **Help / Keybinding Sheet**: Activated via `?`.
- **Auth / Login Notification Dialog**: Displayed when authentication is required.

---

## 4. Keybindings Map

| Key | Action |
|---|---|
| `↑` / `k` | Navigate Up |
| `↓` / `j` | Navigate Down |
| `←` / `h` | Switch focus to Sidebar / Go back |
| `→` / `l` | Switch focus to Main View / Drill into item |
| `Tab` | Toggle focus between Sidebar and Main Content |
| `Enter` | Play selected song / Open album or playlist / Confirm modal |
| `Space` | Toggle Play / Pause |
| `n` | Next Track |
| `p` | Previous Track |
| `[` / `]` | Seek -10s / +10s |
| `+` / `=` | Increase Volume (5%) |
| `-` / `_` | Decrease Volume (5%) |
| `s` | Toggle Shuffle |
| `r` | Cycle Repeat (Off / All / One) |
| `/` | Open Search Dialog |
| `a` | Add selected track to playlist |
| `c` | Create new playlist (when in Playlists view) |
| `d` / `Delete` | Remove track from playlist |
| `Esc` | Close modal / Clear search / Go back |
| `?` | Toggle Help Overlay |
| `q` / `Ctrl+C` | Clean exit application |

---

## 5. Error Handling & Resilience
- **Network Outages**: Retries failed REST requests up to 3 times with exponential backoff; displays inline status messages without crashing the UI.
- **Browser Process Monitoring**: If the child headless browser process exits unexpectedly, the application notifies the user and offers automatic restart.
- **Terminal State Restoration**: Panic hooks and standard exit paths ensure `crossterm` disables raw mode, leaves the alternate screen, and restores terminal cursor visibility.
- **Graceful Process Termination**: Ensures browser process and all child threads/sockets are dropped cleanly upon exit.

---

## 6. Testing & Quality Assurance Plan
1. **Unit Testing**:
   - Configuration and Auth deserialization/serialization with permission checks.
   - REST API URL generation and query parameter builders.
   - Response JSON parsing for tracks, albums, artists, playlists, storefronts.
   - State reducer tests for navigation, selection bounds, queue management, volume capping.
2. **Mock Server & Headless Simulation**:
   - Mock HTTP server to test API client error handling, pagination, and token headers.
   - Playback engine abstraction with mock driver for automated non-browser CI environments.
3. **End-to-End Verification**:
   - Build clean release binary `cargo build --release`.
   - Run complete suite with `cargo test --all-targets`.
   - Validate binary execution, CLI help, config initialization, and graceful termination.
