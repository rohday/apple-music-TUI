# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-09-04

### Added
- Album art: truecolor half-block cover art in the player bar, with on-disk
  artwork cache
- Now Playing popup (`o`) with large cover art, track info and progress
- Queue management: add to queue (`A`), remove (`d`), reorder (`<` / `>`)
- Library songs pagination: fetches ahead automatically beyond the first 100
- Per-view TTL cache for library data for instant view switching
- Sidebar section headers (Search / Library / Playback)
- Loading spinner in the header during background fetches
- CONTRIBUTING.md, CHANGELOG.md and GitHub Actions CI (fmt, clippy, tests)

### Changed
- **Non-blocking UI**: all network requests moved off the key handler into
  background tokio tasks via a Job/Effect pipeline; the interface no longer
  freezes during searches, library loads, or lyric fetches
- Dynamic render tick: 60 FPS while playing, low-power 250 ms when idle
- API errors are surfaced in the status bar instead of being silently
  discarded

### Removed
- Dead hardcoded theme color constants

## [0.1.0] - 2026-08-21

### Added
- Initial release: library browsing, catalog search, playlist CRUD, playback
  via headless Chromium + MusicKit, theme engine (5 presets), braille
  visualizer, synced lyrics via LRCLIB, radio stations, in-view filtering,
  MPRIS2 desktop integration
