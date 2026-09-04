<div align="center">

# appleTUI

**A minimalist, high-contrast Apple Music TUI for Linux.**

Rust · Ratatui · Tokio · MIT

[![CI](https://github.com/rohday/apple-music-TUI/actions/workflows/ci.yml/badge.svg)](https://github.com/rohday/apple-music-TUI/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

appleTUI streams your Apple Music library straight into the terminal: full
library browsing, catalog search, playlists, radio stations, synced lyrics,
desktop media keys, and truecolor ASCII album art — all in a single static
binary with zero audio dependencies of its own.

```
+------------------------------------------------------------------------+
| AppleTUI [IN] ⠋      Filter: "weeknd"   Press '?' Help | '/' Search    |
+-------------------+----------------------------------------------------+
|  Search           |  #  Title               Artist            Duration |
|  Library          | -------------------------------------------------- |
|    Songs          |  >  A&W                 Lana Del Rey          7:13 |
|    Albums         |  2  Aashiqana           Chaar Diwaari         6:43 |
|    Artists        |  3  Achilles Come Down  Gang of Youths        7:02 |
|    Playlists      |  4  The Adults Are Talk The Strokes           5:09 |
+-------------------+----------------------------------------------------+
| ▀▀▄▄ [PLAY] A&W - Lana Del Rey              1:45 / 7:13                |
| ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░                               |
| [p]⏮  [Space]▶⏸  [n]⏭  [s]  [r]  [v]  [+/-] 80%                       |
+------------------------------------------------------------------------+
```

## Features

- **Full library access** — songs, albums, artists, playlists and recently
  played, with instant in-view fuzzy filtering (`f`)
- **Catalog search** — search millions of tracks (`/`)
- **Real playback** — streams via a headless Chromium instance running the
  Apple Music web player; audio plays through your normal desktop audio stack
- **Album art** — truecolor half-block cover art in the player bar and a Now
  Playing popup (`o`), with an on-disk cache
- **Synced lyrics** — side-by-side time-synced lyrics panel (`y`), fetched
  from LRCLIB
- **Radio stations** — start an endless station from any song (`R`)
- **Queue management** — append (`A`), remove (`d`), reorder (`<` / `>`)
- **Visualizer** — 60 FPS braille fluid ribbon and full-screen mode (`v`)
- **Theme engine** — five built-in themes, cycled with `t`, persisted across
  sessions
- **Desktop integration** — MPRIS2 D-Bus: media keys and `playerctl` work
- **Non-blocking UI** — all network work happens on background tasks; the
  interface stays responsive with a 60 FPS render loop while playing

## Dependencies

- **Linux** with **PipeWire** or **PulseAudio**
- **Chromium-based browser** (`brave-browser`, `google-chrome`, or `chromium`)
- **Rust** (`cargo` and `rustc` 1.80+)

## Install & Run

### 1. Build & Install Globally

```bash
git clone https://github.com/rohday/apple-music-TUI.git
cd appleTUI
cargo install --path .
```

> This installs the `appletui` binary into `~/.cargo/bin/`. Make sure
> `~/.cargo/bin` is in your `$PATH`.

*(Alternatively: `cargo build --release && ln -sf $(pwd)/target/release/appletui ~/.local/bin/appletui`)*

### 2. Login (first time only)

```bash
appletui --login
```

Sign in with your Apple ID in the browser window. Your session token is
captured, stored with `0600` permissions, and the window closes.

### 3. Launch

```bash
appletui
```

*(Or run `appletui --mock` to test without logging in.)*

## Keybindings

### Navigation
| Key | Action |
|---|---|
| `↑` / `k` · `↓` / `j` | Move selection |
| `←` / `h` · `→` / `l` | Focus sidebar / content |
| `Tab` | Toggle panel focus |
| `Enter` | Play / open |
| `Esc` | Back / close popup |
| `f` | Filter current view |
| `?` | Help overlay |

### Playback
| Key | Action |
|---|---|
| `Space` | Play / pause |
| `n` / `p` | Next / previous |
| `[` / `]` | Seek ∓10s |
| `+` / `-` | Volume |
| `s` / `r` | Shuffle / repeat |

### Actions
| Key | Action |
|---|---|
| `/` | Search catalog |
| `R` | Start radio station from selection |
| `y` | Synced lyrics panel |
| `o` | Now Playing popup |
| `A` | Add to queue |
| `d` / `<` / `>` | Queue: remove / move up / move down |
| `a` | Add to playlist |
| `c` | Create playlist (Playlists view) |
| `t` | Cycle theme |
| `v` | Visualizer |
| `F5` | Refresh (bypasses cache) |
| `q` / `Ctrl+C` | Quit |

## Configuration

Config lives at `~/.config/appletui/config.json`, auth tokens at
`~/.config/appletui/auth.json`, artwork cache at
`~/.config/appletui/art_cache/`.

```json
{
  "volume": 80,
  "storefront": "us",
  "browser_path": null,
  "mock_mode": false,
  "tick_rate_ms": 16,
  "theme": "AppleDark"
}
```

## Contributing

PRs and issues are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for the
development setup and conventions.

## License

[MIT](LICENSE)
