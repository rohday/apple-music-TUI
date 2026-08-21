# appleTUI — Apple Music TUI for Linux

A fast, lightweight, and responsive Terminal User Interface (TUI) client for Apple Music on Linux. Built with Rust, Ratatui, Tokio, and Chrome DevTools Protocol.

```
+------------------------------------------------------------------------+
| appleTUI v0.1.0                     [AUTH]              Storefront: IN |
+-------------------+----------------------------------------------------+
|  > Search         |  #  Title               Artist            Duration |
|    Library Songs  | -------------------------------------------------- |
|    Albums         |  >  A&W                 Lana Del Rey          7:13 |
|    Artists        |  2  Aashiqana           Chaar Diwaari         6:43 |
|    Playlists      |  3  Achilles Come Down  Gang of Youths        7:02 |
|    Recently Played|  4  The Adults Are Talk The Strokes           5:09 |
+-------------------+----------------------------------------------------+
| [PLAY] A&W - Lana Del Rey                       1:45 / 7:13            |
| [p] |<<  [Space] >/||  [n] >>|  [s] Shuffle: Off  [r] Repeat: Off  Vol:80% |
| ========================----------------------------                   |
+------------------------------------------------------------------------+
```

---

## Features

- **Real-Time Streaming**: Plays Apple Music streams directly through PipeWire / PulseAudio on Linux.
- **Pure ASCII Aesthetic**: Clean, high-contrast monospace UI with bold red accents (`cmus`/`ncspot` feel). Zero emoji alignment bugs.
- **Search & Catalog Exploration**: Search songs, albums, artists, and playlists with instant keyboard filtering.
- **Library Management**: Browse personal library tracks, albums, artists, recently played history, and playlists.
- **Playlist Management**: View playlist tracks, create new playlists (`c`), add tracks to playlists (`a`).
- **Complete Playback Controls**: Play/pause, track skip, 10s seek, shuffle toggle, repeat modes, and volume control.
- **Clean Process Lifecycle**: Spawns headless browser engine on startup and terminates it cleanly on exit. Zero orphan processes.
- **Mock Mode**: Built-in offline mock mode (`--mock`) for testing and previewing without logging in.

---

## Installation & Requirements

### Requirements
- **Linux** (x86_64 or ARM64) with **PipeWire** or **PulseAudio**
- **Chromium-based browser** installed (`brave-browser`, `google-chrome`, or `chromium`)
- **Rust Toolchain** (1.80+ or latest stable)

### Build from Source
```bash
git clone https://github.com/user/appleTUI.git
cd appleTUI
cargo build --release
```
The compiled binary will be located at `target/release/apple-tui`.

---

## Usage

### Run with Offline / Mock Data
```bash
./target/release/apple-tui --mock
```

### Authentication & First Launch

#### Option A: Interactive Browser Sign-In
Launch the interactive login window:
```bash
./target/release/apple-tui --login
```
This opens a browser window to `https://music.apple.com/login`. Once you sign in with your Apple ID, `appleTUI` automatically captures your session cookie (`media-user-token`), saves it securely to `~/.config/appletui/auth.json` (mode `0600`), and closes the browser window.

#### Option B: Manual Token Input
If you prefer extracting the token from your browser manually:
```bash
./target/release/apple-tui --set-user-token "<YOUR_MEDIA_USER_TOKEN>"
```

### Standard Launch
```bash
./target/release/apple-tui
```

---

## Keybindings

### Navigation
| Key | Action |
|---|---|
| `↑` / `k` | Move selection Up |
| `↓` / `j` | Move selection Down |
| `←` / `h` | Focus Sidebar / Go Back |
| `→` / `l` | Focus Main Panel / Drill into item |
| `Tab` | Toggle focus between Sidebar and Main Content |
| `Enter` | Play selected track / Open playlist / Select item |
| `Esc` | Close modal / Clear search / Go back |

### Playback Controls
| Key | Action |
|---|---|
| `Space` | Toggle Play / Pause |
| `n` | Next Track |
| `p` | Previous Track |
| `[` / `]` | Seek -10s / +10s |
| `+` / `=` | Increase Volume (+5%) |
| `-` / `_` | Decrease Volume (-5%) |
| `s` | Toggle Shuffle (On / Off) |
| `r` | Cycle Repeat (Off / All / One) |

### Actions & Modals
| Key | Action |
|---|---|
| `/` | Open Search Prompt |
| `R` / `F5` | Refresh Library / Playlists from Apple Music |
| `c` | Create New Playlist (in Playlists view) |
| `a` | Add selected track to a Playlist |
| `?` | Toggle Help & Keybinding Overlay |
| `q` / `Ctrl+C` | Clean exit application |

---

## Configuration & Storage

Configuration files are stored in `~/.config/appletui/`:
- `config.json`: Volume, default storefront, browser path, tick rate.
- `auth.json`: Developer and User tokens (permission `0600`).

---

## Architecture

```
+--------------------------------------------------------+
|                        appleTUI                        |
|                                                        |
|  +--------------+    +--------------+   +------------+ |
|  |   UI Layer   |    |  API Client  |   |  Playback  | |
|  |  (Ratatui +  |<-->| (reqwest REST|<->|   Engine   | |
|  |  Crossterm)  |    | Apple Music) |   | (CDP / JS) | |
|  +------+-------+    +------+-------+   +-----+------+ |
|         |                   |                 |        |
|         v                   v                 v        |
|  +--------------------------------------------------+  |
|  |              AppState & Event Channel            |  |
|  +--------------------------------------------------+  |
+---------------------------+----------------------------+
                            | (Lifecycle bound)
                            v
              [Headless Chromium / Brave]
                            | (Widevine EME Audio)
                            v
                  [PipeWire / PulseAudio]
```

---

## License

MIT OR Apache-2.0
