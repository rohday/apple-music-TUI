# appleTUI Refinement: Async Foundation, Visual Overhaul & OSS Polish

**Date:** 2026-09-04
**Status:** Approved

## Goals

1. **Foundation** — eliminate UI freezes caused by inline network awaits in the
   key handler; add caching, pagination, queue management, and real error surfacing.
2. **Visual overhaul** — ASCII album art (half-block rendering, works in any
   terminal), player bar art thumbnail, now-playing popup, layout polish,
   unified animation cadence.
3. **OSS polish** — README, CONTRIBUTING, CHANGELOG, CI, version bump to 0.2.0.

Non-goals: theme engine v2 (keep existing 5 presets), real audio spectrum,
graphics-protocol album art, packaging.

## Architecture: async job pipeline

Current: `events::handle_key_event` awaits API calls inline → main loop blocked
during every fetch.

New model — **Jobs and Effects**:

- `src/app/job.rs` — `Job` enum describing background work:
  `Search(query, storefront)`, `LoadView(view)`, `LoadPlaylistTracks(id)`,
  `LoadAlbumTracks(id)`, `LoadArtistTracks(id)`, `CreateStation(song_id, storefront)`,
  `LoadLyrics(song)`, `RefreshView(view)`, `CreatePlaylist(name)`,
  `AddTracksToPlaylist(pl_id, song_id)`, `FetchArtwork(url, song_id)`.
- `execute_job(job, client) -> Effect` — pure-ish async fn; no state access.
- `Effect` enum: `SearchDone(SearchResults)`, `LibraryLoaded(view, items)`,
  `TracksLoaded(Vec<Song>)`, `StationLoaded(Vec<Song>)`, `LyricsLoaded(LyricsData)`,
  `PlaylistCreated(Playlist)`, `ArtworkLoaded(song_id, bytes)`, `Error(String)`.
- `apply_effect(state, effect)` — pure state mutation, fully unit-testable.
- `AppState` gains `pending_jobs: Vec<Job>`. Key handler enqueues jobs and sets
  `is_loading`; it never awaits network. Main loop drains `pending_jobs`,
  spawns each via `tokio::spawn(client-clone, tx)`; effects arrive over an
  `mpsc::channel<Effect>` selected in the main loop.
- Tests execute jobs directly (`execute_job` + `apply_effect`) — deterministic,
  no sleeps.

### Cache

`src/app/cache.rs` — in-memory `DataCache` with per-entry TTL (10 min) for
library songs/albums/artists/playlists/recent. View loads check cache first;
stale entries served immediately then refreshed in background. Library songs
also drive pagination.

### Pagination

Library songs fetched in pages of 100. `AppState` tracks `songs_has_more` +
`songs_loading_more`; when `selected_index >= len - 20`, next page job is
enqueued and appended. Filter/search unchanged.

### Queue management

- `A` — append selected song to queue (any song view).
- `d` — remove from queue (Queue view; keeps playlist-detail delete).
- `<` / `>` — move selected queue item down/up in play order.

### Error surfacing

All `let _ =` swallowed API results in the key path replaced with status
messages; background failures arrive as `Effect::Error` → status + `tracing::warn`.

## Visuals

### Album art

- `Song` gains `artwork_url: Option<String>` parsed from Apple Music
  `attributes.artwork.url` (mock: None → placeholder glyph).
- `src/ui/art.rs`:
  - `fetch_artwork_bytes(http, url)` with disk cache
    `~/.config/appletui/art_cache/<sha256(url)>.img` (0600 dir).
  - `to_half_block_lines(bytes, max_w, max_h) -> Vec<Vec<(fg, bg)>>` — decode
    via `image` crate (png/jpeg only), aspect-preserving thumbnail, each cell =
    `▀` with top pixel fg + bottom pixel bg (2× vertical resolution).
- Player bar: 3–4 cell wide art thumbnail left of track info; placeholder
  `♪` block while loading/missing.
- `o` — Now Playing popup: large art (~24 rows), track metadata, position bar.
- Artwork fetch is a Job → `Effect::ArtworkLoaded`; UI never blocks.

### Layout polish

- Sidebar: `SEARCH` / `LIBRARY` / `DISCOVERY` section labels; cleaner selection.
- Tables: dimmed duration column, `status_message` toast styled via theme.
- Consistent border language: focused pane uses `border_focused`, popups accent.

### Animation

- Single clock: `state.anim_time` drives shimmer + ribbon + visualizer (already
  true); remove dead `Theme::ACCENT`-style constants.
- Dynamic tick: 16 ms while playing/filtering/animating, 200 ms idle → lower
  CPU when paused.

## OSS polish

- README: badges, features, full keybindings, ASCII album art note, install,
  contributing pointer.
- CONTRIBUTING.md, CHANGELOG.md (0.2.0), `.github/workflows/ci.yml`
  (fmt, clippy `-D warnings`, tests, stable toolchain).
- Version → 0.2.0.

## Testing

- Unit: `apply_effect`, cache TTL, pagination trigger, queue ops, art
  half-block conversion (synthetic image), artwork URL parsing.
- Integration: existing suite updated to run jobs deterministically
  (`drain_jobs` helper in tests).
- UI render tests extended for art thumbnail + now-playing popup.
- Gates: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.

## Risks

- `image` dep adds compile weight — mitigate with png/jpeg-only features.
- Refactor touches main loop — mitigated by keeping key-handler signature and
  updating only integration tests that exercise network paths.
