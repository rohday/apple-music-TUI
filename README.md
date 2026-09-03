# AppleTUI

A minimalist, high-contrast Apple Music TUI for Linux. Built with Rust, Ratatui, and Tokio.

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

## Dependencies

- **Linux** with **PipeWire** or **PulseAudio**
- **Chromium-based browser** (`brave-browser`, `google-chrome`, or `chromium`)
- **Rust** (`cargo` and `rustc` 1.80+)

---

## Install & Run

### 1. Build & Install Globally
```bash
git clone https://github.com/<your-username>/appleTUI.git
cd appleTUI
cargo install --path .
```
> This installs the `appletui` binary into `~/.cargo/bin/`. Make sure `~/.cargo/bin` is in your `$PATH`.

*(Alternatively, build locally and symlink: `cargo build --release && ln -sf $(pwd)/target/release/appletui ~/.local/bin/appletui`)*

### 2. Login (First Time Only)
```bash
appletui --login
```
Sign in with your Apple ID in the browser window. It will capture your session token, save it securely, and close the window.

### 3. Launch
```bash
appletui
```
*(Or run `appletui --mock` to test without logging in)*

---

## Keybindings

### Navigation
- `↑` / `k` or `↓` / `j` : Move selection up / down
- `←` / `h` or `→` / `l` : Switch focus between Sidebar and Main Content
- `Tab` : Toggle panel focus
- `Enter` : Play selected track / Open playlist
- `Esc` : Close popup / Go back

### Playback
- `Space` : Play / Pause
- `n` / `p` : Next / Previous track
- `[` / `]` : Seek -10s / +10s
- `+` / `-` : Volume up / down
- `s` : Toggle shuffle
- `r` : Cycle repeat mode

### Actions
- `/` : Search catalog
- `R` / `F5` : Refresh library data
- `c` : Create new playlist (in Playlists view)
- `a` : Add selected track to playlist
- `?` : Show help overlay
- `q` / `Ctrl+C` : Quit

---

## License

MIT
