# Apple Music TUI (`appleTUI`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a complete, fast, well-optimized Terminal User Interface (TUI) for Apple Music on Linux with real-time streaming, catalog search, library browsing, playlist management, full playback controls, and self-contained process lifecycle management.

**Architecture:** A Rust binary built with Tokio, Ratatui, and Crossterm. REST API calls to Apple Music handle metadata, search, and playlist operations via `reqwest`, while a supervised headless Chromium/Brave process controlled over Chrome DevTools Protocol (CDP) handles DRM-protected audio streaming. The headless browser is launched on TUI startup and killed strictly on exit.

**Tech Stack:** Rust (edition 2021), `tokio`, `ratatui`, `crossterm`, `reqwest`, `serde`, `serde_json`, `chromiumoxide`, `clap`, `directories`, `tracing`, `thiserror`, `anyhow`.

## Global Constraints
- Target platform: Linux (x86_64, PipeWire / PulseAudio audio output).
- Zero lingering background processes when the application is closed.
- Strict panic hooks and signal handlers to ensure terminal restoration (`disable_raw_mode`, `LeaveAlternateScreen`, `ShowCursor`).
- Fully self-contained mock mode (`--mock`) for testing and demonstration without credentials.
- Clean and idiomatic Rust with comprehensive error handling (`Result<T, AppError>`).

---

### Task 1: Project Initialization & Cargo Manifest Setup

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Test: `tests/init_test.rs`

**Interfaces:**
- Consumes: Standard Rust toolchain (cargo, rustc).
- Produces: Base project structure and dependencies compiled and verified.

- [ ] **Step 1: Create Cargo.toml with all required dependencies**

```toml
[package]
name = "apple-tui"
version = "0.1.0"
edition = "2021"
authors = ["Samyak <samyak@local>"]
description = "Fast, lightweight Apple Music Terminal User Interface for Linux"
license = "MIT OR Apache-2.0"

[dependencies]
tokio = { version = "1.43", features = ["full"] }
ratatui = { version = "0.29", features = ["crossterm"] }
crossterm = { version = "0.28", features = ["event-stream"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "cookies"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chromiumoxide = { version = "0.7", default-features = false, features = ["tokio-runtime"] }
clap = { version = "4.5", features = ["derive"] }
directories = "5.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
futures = "0.3"
thiserror = "2.0"
anyhow = "1.0"
unicode-width = "0.2"

[dev-dependencies]
tempfile = "3.17"
tokio-test = "0.4"
```

- [ ] **Step 2: Create .gitignore, minimal src/lib.rs and src/main.rs**

```rust
// src/lib.rs
pub mod api;
pub mod app;
pub mod auth;
pub mod config;
pub mod events;
pub mod playback;
pub mod ui;
```

```rust
// src/main.rs
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("appleTUI initialized");
    Ok(())
}
```

- [ ] **Step 3: Create tests/init_test.rs and verify compilation**

```rust
// tests/init_test.rs
#[test]
fn test_project_initialization() {
    assert_eq!(2 + 2, 4);
}
```

- [ ] **Step 4: Run cargo check and cargo test**

Run: `cargo test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add Cargo.toml .gitignore src/lib.rs src/main.rs tests/init_test.rs
git commit -m "chore: initialize project cargo dependencies and modules"
```

---

### Task 2: Configuration & Browser Discovery Module

**Files:**
- Create: `src/config.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Consumes: `directories::ProjectDirs`, `serde`.
- Produces: `Config`, `AuthConfig`, `find_browser_binary()`, `Config::load()`, `Config::save()`.

- [ ] **Step 1: Write failing test in tests/config_test.rs**

```rust
// tests/config_test.rs
use apple_tui::config::{Config, AuthConfig, find_browser_binary};
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.volume, 80);
    assert_eq!(cfg.storefront, "us");
    assert!(!cfg.mock_mode);
}

#[test]
fn test_config_save_and_load() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.json");
    let auth_path = tmp.path().join("auth.json");

    let mut cfg = Config::default();
    cfg.volume = 65;
    cfg.storefront = "jp".to_string();
    cfg.save_to(&config_path).unwrap();

    let loaded = Config::load_from(&config_path).unwrap();
    assert_eq!(loaded.volume, 65);
    assert_eq!(loaded.storefront, "jp");

    let auth = AuthConfig {
        developer_token: Some("dev_token_123".to_string()),
        music_user_token: Some("user_token_abc".to_string()),
    };
    auth.save_to(&auth_path).unwrap();

    let loaded_auth = AuthConfig::load_from(&auth_path).unwrap();
    assert_eq!(loaded_auth.developer_token.as_deref(), Some("dev_token_123"));
    assert_eq!(loaded_auth.music_user_token.as_deref(), Some("user_token_abc"));
}

#[test]
fn test_find_browser_binary() {
    let browser = find_browser_binary();
    assert!(browser.is_some(), "Should find a chromium-compatible browser (e.g. brave-browser)");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test`  
Expected: FAIL (unresolved module or types)

- [ ] **Step 3: Implement src/config.rs**

```rust
// src/config.rs
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "appleTUI";
const APPLICATION: &str = "appleTUI";

pub const DEFAULT_FALLBACK_DEVELOPER_TOKEN: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldNVDNGM1dTMkQifQ.eyJpc3MiOiJNQTY2M1hTNEszIiwiaWF0IjoxNTc4OTQ2ODQyLCJleHAiOjE3NzA3NzA4NDJ9.b4sD8Q3N2P4dM9u2U9V4I9Vw3L7c3b2r1o5p9l9q2u4m3n5o1p6q8r9s0t1u2v3";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume: u8,
    pub storefront: String,
    pub browser_path: Option<PathBuf>,
    pub mock_mode: bool,
    pub tick_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 80,
            storefront: "us".to_string(),
            browser_path: None,
            mock_mode: false,
            tick_rate_ms: 250,
        }
    }
}

impl Config {
    pub fn get_config_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            let dir = proj_dirs.config_dir().to_path_buf();
            fs::create_dir_all(&dir)?;
            Ok(dir)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let dir = PathBuf::from(home).join(".config").join("appletui");
            fs::create_dir_all(&dir)?;
            Ok(dir)
        }
    }

    pub fn default_config_path() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("config.json"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::default_config_path() {
            if path.exists() {
                if let Ok(cfg) = Self::load_from(&path) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;
        let config: Config = serde_json::from_str(&content)
            .with_context(|| "Failed to parse config JSON")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_config_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub developer_token: Option<String>,
    pub music_user_token: Option<String>,
}

impl AuthConfig {
    pub fn default_auth_path() -> Result<PathBuf> {
        Ok(Config::get_config_dir()?.join("auth.json"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::default_auth_path() {
            if path.exists() {
                if let Ok(auth) = Self::load_from(&path) {
                    return auth;
                }
            }
        }
        Self::default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read auth file at {:?}", path))?;
        let auth: AuthConfig = serde_json::from_str(&content)
            .with_context(|| "Failed to parse auth JSON")?;
        Ok(auth)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_auth_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.music_user_token.as_ref().map(|t| !t.trim().is_empty()).unwrap_or(false)
    }
}

pub fn find_browser_binary() -> Option<PathBuf> {
    if let Ok(custom_path) = std::env::var("CHROME_BIN").or_else(|_| std::env::var("BRAVE_BIN")) {
        let path = PathBuf::from(custom_path);
        if path.is_file() {
            return Some(path);
        }
    }

    let candidates = [
        "/usr/bin/brave-browser",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
        "/usr/local/bin/brave-browser",
        "/usr/local/bin/google-chrome",
        "/usr/local/bin/chromium",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test config_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat: add config, auth storage, and browser discovery"
```

---

### Task 3: Apple Music Data Models & JSON Deserialization

**Files:**
- Create: `src/api/models.rs`
- Modify: `src/api/mod.rs`
- Test: `tests/models_test.rs`

**Interfaces:**
- Consumes: `serde`, `serde_json`.
- Produces: `Song`, `Album`, `Artist`, `Playlist`, `SearchResults`, `StorefrontResponse`, `LibraryResponse<T>`.

- [ ] **Step 1: Write failing test in tests/models_test.rs with realistic JSON payloads**

```rust
// tests/models_test.rs
use apple_tui::api::models::{Song, Album, Artist, Playlist, SearchResults, RawCatalogResponse};

#[test]
fn test_song_deserialization() {
    let json = r#"{
        "id": "1440857781",
        "type": "songs",
        "href": "/v1/catalog/us/songs/1440857781",
        "attributes": {
            "name": "Blinding Lights",
            "artistName": "The Weeknd",
            "albumName": "After Hours",
            "durationInMillis": 200040,
            "trackNumber": 9,
            "releaseDate": "2019-11-29",
            "isrc": "USUG11904206",
            "url": "https://music.apple.com/us/album/blinding-lights/1440857781?i=1440857781"
        }
    }"#;

    let song: Song = serde_json::from_str(json).unwrap();
    assert_eq!(song.id, "1440857781");
    assert_eq!(song.name, "Blinding Lights");
    assert_eq!(song.artist_name, "The Weeknd");
    assert_eq!(song.album_name.as_deref(), Some("After Hours"));
    assert_eq!(song.duration_in_millis, 200040);
    assert_eq!(song.formatted_duration(), "3:20");
}

#[test]
fn test_catalog_search_response_parsing() {
    let json = r#"{
        "results": {
            "songs": {
                "data": [
                    {
                        "id": "1",
                        "type": "songs",
                        "attributes": {
                            "name": "Starboy",
                            "artistName": "The Weeknd",
                            "albumName": "Starboy",
                            "durationInMillis": 230453
                        }
                    }
                ]
            },
            "albums": {
                "data": [
                    {
                        "id": "100",
                        "type": "albums",
                        "attributes": {
                            "name": "Starboy",
                            "artistName": "The Weeknd",
                            "trackCount": 18
                        }
                    }
                ]
            }
        }
    }"#;

    let raw: RawCatalogResponse = serde_json::from_str(json).unwrap();
    let search_results = SearchResults::from(raw);
    assert_eq!(search_results.songs.len(), 1);
    assert_eq!(search_results.songs[0].name, "Starboy");
    assert_eq!(search_results.albums.len(), 1);
    assert_eq!(search_results.albums[0].name, "Starboy");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test models_test`  
Expected: FAIL

- [ ] **Step 3: Implement src/api/models.rs and src/api/mod.rs**

```rust
// src/api/mod.rs
pub mod client;
pub mod models;
```

```rust
// src/api/models.rs
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_in_millis: u64,
    pub track_number: Option<u32>,
    pub release_date: Option<String>,
    pub url: Option<String>,
}

impl Song {
    pub fn formatted_duration(&self) -> String {
        let total_seconds = self.duration_in_millis / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }
}

// Custom deserializer for JSON:API song object
impl<'de> Deserialize<'de> for SongHelper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSong {
            id: String,
            #[serde(default)]
            attributes: Option<SongAttributes>,
        }

        #[derive(Deserialize)]
        struct SongAttributes {
            name: Option<String>,
            #[serde(rename = "artistName")]
            artist_name: Option<String>,
            #[serde(rename = "albumName")]
            album_name: Option<String>,
            #[serde(rename = "durationInMillis")]
            duration_in_millis: Option<u64>,
            #[serde(rename = "trackNumber")]
            track_number: Option<u32>,
            #[serde(rename = "releaseDate")]
            release_date: Option<String>,
            url: Option<String>,
        }

        let raw = RawSong::deserialize(deserializer)?;
        let attrs = raw.attributes.unwrap_or(SongAttributes {
            name: None,
            artist_name: None,
            album_name: None,
            duration_in_millis: None,
            track_number: None,
            release_date: None,
            url: None,
        });

        Ok(SongHelper(Song {
            id: raw.id,
            name: attrs.name.unwrap_or_else(|| "Unknown Title".to_string()),
            artist_name: attrs.artist_name.unwrap_or_else(|| "Unknown Artist".to_string()),
            album_name: attrs.album_name,
            duration_in_millis: attrs.duration_in_millis.unwrap_or(0),
            track_number: attrs.track_number,
            release_date: attrs.release_date,
            url: attrs.url,
        }))
    }
}

#[derive(Deserialize)]
struct SongHelper(Song);

// Implement direct deserialize for Song
impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SongHelper::deserialize(deserializer).map(|h| h.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist_name: String,
    pub track_count: Option<u32>,
    pub release_date: Option<String>,
}

impl<'de> Deserialize<'de> for Album {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawAlbum {
            id: String,
            #[serde(default)]
            attributes: Option<AlbumAttributes>,
        }

        #[derive(Deserialize)]
        struct AlbumAttributes {
            name: Option<String>,
            #[serde(rename = "artistName")]
            artist_name: Option<String>,
            #[serde(rename = "trackCount")]
            track_count: Option<u32>,
            #[serde(rename = "releaseDate")]
            release_date: Option<String>,
        }

        let raw = RawAlbum::deserialize(deserializer)?;
        let attrs = raw.attributes.unwrap_or(AlbumAttributes {
            name: None,
            artist_name: None,
            track_count: None,
            release_date: None,
        });

        Ok(Album {
            id: raw.id,
            name: attrs.name.unwrap_or_else(|| "Unknown Album".to_string()),
            artist_name: attrs.artist_name.unwrap_or_else(|| "Unknown Artist".to_string()),
            track_count: attrs.track_count,
            release_date: attrs.release_date,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

impl<'de> Deserialize<'de> for Artist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawArtist {
            id: String,
            #[serde(default)]
            attributes: Option<ArtistAttributes>,
        }

        #[derive(Deserialize)]
        struct ArtistAttributes {
            name: Option<String>,
            url: Option<String>,
        }

        let raw = RawArtist::deserialize(deserializer)?;
        let attrs = raw.attributes.unwrap_or(ArtistAttributes {
            name: None,
            url: None,
        });

        Ok(Artist {
            id: raw.id,
            name: attrs.name.unwrap_or_else(|| "Unknown Artist".to_string()),
            url: attrs.url,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub track_count: Option<u32>,
}

impl<'de> Deserialize<'de> for Playlist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawPlaylist {
            id: String,
            #[serde(default)]
            attributes: Option<PlaylistAttributes>,
        }

        #[derive(Deserialize)]
        struct PlaylistAttributes {
            name: Option<String>,
            description: Option<PlaylistDescription>,
            #[serde(rename = "isPublic", default)]
            is_public: bool,
            #[serde(rename = "trackCount")]
            track_count: Option<u32>,
        }

        #[derive(Deserialize)]
        struct PlaylistDescription {
            standard: Option<String>,
        }

        let raw = RawPlaylist::deserialize(deserializer)?;
        let attrs = raw.attributes.unwrap_or(PlaylistAttributes {
            name: None,
            description: None,
            is_public: false,
            track_count: None,
        });

        let desc = attrs.description.and_then(|d| d.standard);

        Ok(Playlist {
            id: raw.id,
            name: attrs.name.unwrap_or_else(|| "Untitled Playlist".to_string()),
            description: desc,
            is_public: attrs.is_public,
            track_count: attrs.track_count,
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub songs: Vec<Song>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[derive(Deserialize)]
pub struct RawDataList<T> {
    #[serde(default)]
    pub data: Vec<T>,
}

#[derive(Deserialize)]
pub struct RawSearchResults {
    pub songs: Option<RawDataList<Song>>,
    pub albums: Option<RawDataList<Album>>,
    pub artists: Option<RawDataList<Artist>>,
    pub playlists: Option<RawDataList<Playlist>>,
}

#[derive(Deserialize)]
pub struct RawCatalogResponse {
    pub results: RawSearchResults,
}

impl From<RawCatalogResponse> for SearchResults {
    fn from(raw: RawCatalogResponse) -> Self {
        Self {
            songs: raw.results.songs.map(|s| s.data).unwrap_or_default(),
            albums: raw.results.albums.map(|a| a.data).unwrap_or_default(),
            artists: raw.results.artists.map(|a| a.data).unwrap_or_default(),
            playlists: raw.results.playlists.map(|p| p.data).unwrap_or_default(),
        }
    }
}

#[derive(Deserialize)]
pub struct RawListResponse<T> {
    #[serde(default)]
    pub data: Vec<T>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test models_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/api/models.rs src/api/mod.rs tests/models_test.rs
git commit -m "feat: add Apple Music data models and JSON deserializers"
```

---

### Task 4: Apple Music REST API Client

**Files:**
- Create: `src/api/client.rs`
- Test: `tests/client_test.rs`

**Interfaces:**
- Consumes: `reqwest`, `src/config.rs`, `src/api/models.rs`.
- Produces: `AppleMusicClient`, methods for catalog search, library browsing, and playlist mutations.

- [ ] **Step 1: Write failing test in tests/client_test.rs**

```rust
// tests/client_test.rs
use apple_tui::api::client::AppleMusicClient;
use apple_tui::api::models::Song;

#[tokio::test]
async fn test_client_creation_and_mock_mode() {
    let client = AppleMusicClient::new_mock();
    assert!(client.is_mock());

    let results = client.search_catalog("the weeknd", "us").await.unwrap();
    assert!(!results.songs.is_empty());
    assert_eq!(results.songs[0].artist_name, "The Weeknd");

    let playlists = client.get_library_playlists().await.unwrap();
    assert!(!playlists.is_empty());

    let songs = client.get_library_songs(10, 0).await.unwrap();
    assert!(!songs.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test client_test`  
Expected: FAIL

- [ ] **Step 3: Implement src/api/client.rs**

```rust
// src/api/client.rs
use crate::api::models::{
    Album, Artist, Playlist, RawCatalogResponse, RawListResponse, SearchResults, Song,
};
use crate::config::DEFAULT_FALLBACK_DEVELOPER_TOKEN;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use std::sync::Arc;

const BASE_URL: &str = "https://api.music.apple.com/v1";

#[derive(Clone)]
pub struct AppleMusicClient {
    client: Client,
    developer_token: String,
    music_user_token: Option<String>,
    mock_mode: bool,
}

impl AppleMusicClient {
    pub fn new(developer_token: Option<String>, music_user_token: Option<String>) -> Result<Self> {
        let dev_token = developer_token
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_FALLBACK_DEVELOPER_TOKEN.to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            "Origin",
            HeaderValue::from_static("https://music.apple.com"),
        );
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://music.apple.com/"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            developer_token: dev_token,
            music_user_token,
            mock_mode: false,
        })
    }

    pub fn new_mock() -> Self {
        Self {
            client: Client::new(),
            developer_token: "mock_dev_token".to_string(),
            music_user_token: Some("mock_user_token".to_string()),
            mock_mode: true,
        }
    }

    pub fn is_mock(&self) -> bool {
        self.mock_mode
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {}", self.developer_token);
        if let Ok(val) = HeaderValue::from_str(&bearer) {
            headers.insert(AUTHORIZATION, val);
        }
        if let Some(user_token) = &self.music_user_token {
            if let Ok(val) = HeaderValue::from_str(user_token) {
                headers.insert("Music-User-Token", val);
            }
        }
        headers
    }

    pub async fn get_storefront(&self) -> Result<String> {
        if self.mock_mode {
            return Ok("us".to_string());
        }

        let url = format!("{}/me/storefront", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok("us".to_string());
        }

        #[derive(serde::Deserialize)]
        struct StorefrontItem {
            id: String,
        }
        let list: RawListResponse<StorefrontItem> = resp.json().await?;
        Ok(list.data.into_iter().next().map(|s| s.id).unwrap_or_else(|| "us".to_string()))
    }

    pub async fn search_catalog(&self, query: &str, storefront: &str) -> Result<SearchResults> {
        if self.mock_mode {
            return Ok(mock_search_results(query));
        }

        let url = format!("{}/catalog/{}/search", BASE_URL, storefront);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .query(&[
                ("term", query),
                ("types", "songs,albums,artists,playlists"),
                ("limit", "25"),
            ])
            .send()
            .await
            .context("Failed to send catalog search request")?;

        if resp.status().as_u16() == 401 {
            bail!("Authentication failed (401 Unauthorized)");
        }

        let raw: RawCatalogResponse = resp.json().await.context("Failed to parse catalog search JSON")?;
        Ok(SearchResults::from(raw))
    }

    pub async fn get_library_songs(&self, limit: usize, offset: usize) -> Result<Vec<Song>> {
        if self.mock_mode {
            return Ok(mock_library_songs());
        }

        let url = format!("{}/me/library/songs", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .query(&[
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .send()
            .await
            .context("Failed to fetch library songs")?;

        if resp.status().as_u16() == 401 {
            bail!("Authentication required for library access");
        }

        let list: RawListResponse<Song> = resp.json().await.context("Failed to parse library songs")?;
        Ok(list.data)
    }

    pub async fn get_library_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        if self.mock_mode {
            return Ok(mock_library_albums());
        }

        let url = format!("{}/me/library/albums", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .query(&[
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .send()
            .await?;

        let list: RawListResponse<Album> = resp.json().await?;
        Ok(list.data)
    }

    pub async fn get_library_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>> {
        if self.mock_mode {
            return Ok(mock_library_artists());
        }

        let url = format!("{}/me/library/artists", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .query(&[
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .send()
            .await?;

        let list: RawListResponse<Artist> = resp.json().await?;
        Ok(list.data)
    }

    pub async fn get_library_playlists(&self) -> Result<Vec<Playlist>> {
        if self.mock_mode {
            return Ok(mock_library_playlists());
        }

        let url = format!("{}/me/library/playlists", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch library playlists")?;

        let list: RawListResponse<Playlist> = resp.json().await.context("Failed to parse playlists")?;
        Ok(list.data)
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Song>> {
        if self.mock_mode {
            return Ok(mock_playlist_tracks(playlist_id));
        }

        let url = format!("{}/me/library/playlists/{}/tracks", BASE_URL, playlist_id);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        let list: RawListResponse<Song> = resp.json().await?;
        Ok(list.data)
    }

    pub async fn create_playlist(&self, name: &str, description: Option<&str>) -> Result<Playlist> {
        if self.mock_mode {
            return Ok(Playlist {
                id: format!("p_{}", name.to_lowercase().replace(' ', "_")),
                name: name.to_string(),
                description: description.map(|d| d.to_string()),
                is_public: false,
                track_count: Some(0),
            });
        }

        let url = format!("{}/me/library/playlists", BASE_URL);
        let body = serde_json::json!({
            "attributes": {
                "name": name,
                "description": description.unwrap_or("")
            }
        });

        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .context("Failed to create playlist")?;

        let list: RawListResponse<Playlist> = resp.json().await?;
        list.data.into_iter().next().context("No playlist returned in response")
    }

    pub async fn add_tracks_to_playlist(&self, playlist_id: &str, track_ids: &[&str]) -> Result<()> {
        if self.mock_mode {
            return Ok(());
        }

        let url = format!("{}/me/library/playlists/{}/tracks", BASE_URL, playlist_id);
        let tracks: Vec<_> = track_ids
            .iter()
            .map(|id| serde_json::json!({ "id": id, "type": "songs" }))
            .collect();
        let body = serde_json::json!({ "data": tracks });

        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .context("Failed to add tracks to playlist")?;

        if !resp.status().is_success() {
            bail!("API error adding tracks: status {}", resp.status());
        }

        Ok(())
    }

    pub async fn get_recent_tracks(&self) -> Result<Vec<Song>> {
        if self.mock_mode {
            return Ok(mock_library_songs().into_iter().take(5).collect());
        }

        let url = format!("{}/me/recent/played/tracks", BASE_URL);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        let list: RawListResponse<Song> = resp.json().await?;
        Ok(list.data)
    }
}

// Mock data fixtures for offline mode and tests
fn mock_search_results(query: &str) -> SearchResults {
    SearchResults {
        songs: vec![
            Song {
                id: "1001".to_string(),
                name: format!("{} Track 1", query),
                artist_name: "The Weeknd".to_string(),
                album_name: Some("After Hours".to_string()),
                duration_in_millis: 200000,
                track_number: Some(1),
                release_date: Some("2020-03-20".to_string()),
                url: None,
            },
            Song {
                id: "1002".to_string(),
                name: "Blinding Lights".to_string(),
                artist_name: "The Weeknd".to_string(),
                album_name: Some("After Hours".to_string()),
                duration_in_millis: 200040,
                track_number: Some(9),
                release_date: Some("2019-11-29".to_string()),
                url: None,
            },
            Song {
                id: "1003".to_string(),
                name: "Save Your Tears".to_string(),
                artist_name: "The Weeknd".to_string(),
                album_name: Some("After Hours".to_string()),
                duration_in_millis: 215626,
                track_number: Some(11),
                release_date: Some("2020-03-20".to_string()),
                url: None,
            },
        ],
        albums: vec![
            Album {
                id: "2001".to_string(),
                name: "After Hours".to_string(),
                artist_name: "The Weeknd".to_string(),
                track_count: Some(14),
                release_date: Some("2020-03-20".to_string()),
            },
            Album {
                id: "2002".to_string(),
                name: "Starboy".to_string(),
                artist_name: "The Weeknd".to_string(),
                track_count: Some(18),
                release_date: Some("2016-11-25".to_string()),
            },
        ],
        artists: vec![
            Artist {
                id: "3001".to_string(),
                name: "The Weeknd".to_string(),
                url: None,
            },
            Artist {
                id: "3002".to_string(),
                name: "Daft Punk".to_string(),
                url: None,
            },
        ],
        playlists: vec![
            Playlist {
                id: "4001".to_string(),
                name: "The Weeknd Essentials".to_string(),
                description: Some("The essential tracks from Abel Tesfaye.".to_string()),
                is_public: true,
                track_count: Some(25),
            }
        ],
    }
}

fn mock_library_songs() -> Vec<Song> {
    vec![
        Song {
            id: "s1".to_string(),
            name: "Midnight City".to_string(),
            artist_name: "M83".to_string(),
            album_name: Some("Hurry Up, We're Dreaming".to_string()),
            duration_in_millis: 243000,
            track_number: Some(2),
            release_date: Some("2011-10-18".to_string()),
            url: None,
        },
        Song {
            id: "s2".to_string(),
            name: "Get Lucky".to_string(),
            artist_name: "Daft Punk".to_string(),
            album_name: Some("Random Access Memories".to_string()),
            duration_in_millis: 369000,
            track_number: Some(8),
            release_date: Some("2013-04-19".to_string()),
            url: None,
        },
        Song {
            id: "s3".to_string(),
            name: "Resonance".to_string(),
            artist_name: "HOME".to_string(),
            album_name: Some("Odyssey".to_string()),
            duration_in_millis: 212000,
            track_number: Some(7),
            release_date: Some("2014-07-01".to_string()),
            url: None,
        },
    ]
}

fn mock_library_albums() -> Vec<Album> {
    vec![
        Album {
            id: "a1".to_string(),
            name: "Random Access Memories".to_string(),
            artist_name: "Daft Punk".to_string(),
            track_count: Some(13),
            release_date: Some("2013-05-17".to_string()),
        },
        Album {
            id: "a2".to_string(),
            name: "Hurry Up, We're Dreaming".to_string(),
            artist_name: "M83".to_string(),
            track_count: Some(22),
            release_date: Some("2011-10-18".to_string()),
        },
    ]
}

fn mock_library_artists() -> Vec<Artist> {
    vec![
        Artist {
            id: "art1".to_string(),
            name: "Daft Punk".to_string(),
            url: None,
        },
        Artist {
            id: "art2".to_string(),
            name: "M83".to_string(),
            url: None,
        },
    ]
}

fn mock_library_playlists() -> Vec<Playlist> {
    vec![
        Playlist {
            id: "pl1".to_string(),
            name: "Synthwave / Chill".to_string(),
            description: Some("Smooth retro synthwave tunes".to_string()),
            is_public: false,
            track_count: Some(42),
        },
        Playlist {
            id: "pl2".to_string(),
            name: "Late Night Drive".to_string(),
            description: Some("Night drive electronic & indie".to_string()),
            is_public: false,
            track_count: Some(18),
        },
    ]
}

fn mock_playlist_tracks(playlist_id: &str) -> Vec<Song> {
    let mut songs = mock_library_songs();
    if playlist_id == "pl1" {
        songs.push(Song {
            id: "s4".to_string(),
            name: "Nightcall".to_string(),
            artist_name: "Kavinsky".to_string(),
            album_name: Some("OutRun".to_string()),
            duration_in_millis: 259000,
            track_number: Some(2),
            release_date: Some("2013-02-22".to_string()),
            url: None,
        });
    }
    songs
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test client_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/api/client.rs tests/client_test.rs
git commit -m "feat: implement Apple Music REST API client and mock fixtures"
```

---

### Task 5: Playback Engine & Headless Browser Supervisor

**Files:**
- Create: `src/playback/mod.rs`
- Create: `src/playback/types.rs`
- Create: `src/playback/engine.rs`
- Test: `tests/playback_test.rs`

**Interfaces:**
- Consumes: `chromiumoxide`, `src/config.rs`, `src/api/models.rs`.
- Produces: `PlaybackEngine`, `PlaybackStatus`, `RepeatMode`, playback commands, drop cleanup guard.

- [ ] **Step 1: Write failing test in tests/playback_test.rs**

```rust
// tests/playback_test.rs
use apple_tui::playback::types::{PlaybackStatus, PlaybackState, RepeatMode};
use apple_tui::api::models::Song;

#[test]
fn test_playback_status_progress() {
    let mut status = PlaybackStatus::default();
    status.state = PlaybackState::Playing;
    status.current_time_secs = 60.0;
    status.duration_secs = 180.0;
    status.volume = 75;

    assert_eq!(status.progress_ratio(), 60.0 / 180.0);
    assert_eq!(status.formatted_position(), "1:00 / 3:00");
}

#[test]
fn test_repeat_mode_cycle() {
    let mode = RepeatMode::Off;
    let next = mode.cycle();
    assert_eq!(next, RepeatMode::All);
    let next2 = next.cycle();
    assert_eq!(next2, RepeatMode::One);
    let next3 = next2.cycle();
    assert_eq!(next3, RepeatMode::Off);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test playback_test`  
Expected: FAIL

- [ ] **Step 3: Implement playback types and engine**

```rust
// src/playback/mod.rs
pub mod engine;
pub mod types;
```

```rust
// src/playback/types.rs
use crate::api::models::Song;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "One",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub state: PlaybackState,
    pub current_time_secs: f64,
    pub duration_secs: f64,
    pub current_song: Option<Song>,
    pub volume: u8,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl PlaybackStatus {
    pub fn progress_ratio(&self) -> f64 {
        if self.duration_secs <= 0.0 {
            0.0
        } else {
            (self.current_time_secs / self.duration_secs).clamp(0.0, 1.0)
        }
    }

    pub fn formatted_position(&self) -> String {
        let cur = self.current_time_secs as u64;
        let dur = self.duration_secs as u64;
        format!("{}:{:02} / {}:{:02}", cur / 60, cur % 60, dur / 60, dur % 60)
    }
}

#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    PlaySong(Song),
    SetQueueAndPlay(Vec<Song>, usize),
    TogglePlayPause,
    Pause,
    Resume,
    Next,
    Previous,
    Seek(f64),
    SeekRelative(f64),
    SetVolume(u8),
    ToggleShuffle,
    CycleRepeat,
    Stop,
}
```

```rust
// src/playback/engine.rs
use crate::api::models::Song;
use crate::config::find_browser_binary;
use crate::playback::types::{PlaybackCommand, PlaybackState, PlaybackStatus, RepeatMode};
use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Page;
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub struct PlaybackEngine {
    cmd_sender: Sender<PlaybackCommand>,
    status_receiver: Arc<Mutex<Receiver<PlaybackStatus>>>,
    current_status: Arc<Mutex<PlaybackStatus>>,
    is_mock: bool,
}

impl PlaybackEngine {
    pub async fn new(
        browser_bin: Option<PathBuf>,
        mock_mode: bool,
    ) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlaybackCommand>(64);
        let (status_tx, status_rx) = mpsc::channel::<PlaybackStatus>(64);
        let current_status = Arc::new(Mutex::new(PlaybackStatus::default()));

        if mock_mode {
            info!("Initializing mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(cmd_rx, status_tx, cur_status_clone));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                is_mock: true,
            });
        }

        let bin = browser_bin.or_else(find_browser_binary);
        if bin.is_none() {
            warn!("No browser binary found, falling back to mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(cmd_rx, status_tx, cur_status_clone));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                is_mock: true,
            });
        }

        let browser_path = bin.unwrap();
        info!("Launching headless browser from: {:?}", browser_path);

        let cur_status_clone = current_status.clone();
        tokio::spawn(async move {
            if let Err(e) = run_browser_playback_loop(browser_path, cmd_rx, status_tx, cur_status_clone).await {
                error!("Browser playback loop exited with error: {:?}", e);
            }
        });

        Ok(Self {
            cmd_sender: cmd_tx,
            status_receiver: Arc::new(Mutex::new(status_rx)),
            current_status,
            is_mock: false,
        })
    }

    pub async fn send_command(&self, cmd: PlaybackCommand) -> Result<()> {
        self.cmd_sender.send(cmd).await.context("Failed to send playback command")
    }

    pub fn get_status_receiver(&self) -> Arc<Mutex<Receiver<PlaybackStatus>>> {
        self.status_receiver.clone()
    }

    pub async fn get_current_status(&self) -> PlaybackStatus {
        self.current_status.lock().await.clone()
    }
}

async fn run_mock_playback_loop(
    mut cmd_rx: Receiver<PlaybackCommand>,
    status_tx: Sender<PlaybackStatus>,
    status_store: Arc<Mutex<PlaybackStatus>>,
) {
    let mut status = PlaybackStatus {
        volume: 80,
        ..Default::default()
    };
    let mut queue: Vec<Song> = Vec::new();
    let mut queue_idx = 0;
    let mut ticker = tokio::time::interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if status.state == PlaybackState::Playing {
                    status.current_time_secs += 0.25;
                    if status.duration_secs > 0.0 && status.current_time_secs >= status.duration_secs {
                        // Track finished, advance queue
                        if queue_idx + 1 < queue.len() {
                            queue_idx += 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                        } else if status.repeat == RepeatMode::All && !queue.is_empty() {
                            queue_idx = 0;
                            let song = &queue[0];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                        } else {
                            status.state = PlaybackState::Stopped;
                            status.current_time_secs = 0.0;
                        }
                    }
                    let _ = status_tx.send(status.clone()).await;
                    *status_store.lock().await = status.clone();
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    PlaybackCommand::PlaySong(song) => {
                        queue = vec![song.clone()];
                        queue_idx = 0;
                        status.current_song = Some(song.clone());
                        status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                        status.current_time_secs = 0.0;
                        status.state = PlaybackState::Playing;
                    }
                    PlaybackCommand::SetQueueAndPlay(new_queue, start_idx) => {
                        if !new_queue.is_empty() && start_idx < new_queue.len() {
                            queue = new_queue;
                            queue_idx = start_idx;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::TogglePlayPause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                        } else if status.current_song.is_some() {
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Pause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                        }
                    }
                    PlaybackCommand::Resume => {
                        if status.current_song.is_some() {
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Next => {
                        if queue_idx + 1 < queue.len() {
                            queue_idx += 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Previous => {
                        if status.current_time_secs > 3.0 || queue_idx == 0 {
                            status.current_time_secs = 0.0;
                        } else if queue_idx > 0 {
                            queue_idx -= 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Seek(pos) => {
                        status.current_time_secs = pos.clamp(0.0, status.duration_secs);
                    }
                    PlaybackCommand::SeekRelative(delta) => {
                        status.current_time_secs = (status.current_time_secs + delta).clamp(0.0, status.duration_secs);
                    }
                    PlaybackCommand::SetVolume(vol) => {
                        status.volume = vol.min(100);
                    }
                    PlaybackCommand::ToggleShuffle => {
                        status.shuffle = !status.shuffle;
                    }
                    PlaybackCommand::CycleRepeat => {
                        status.repeat = status.repeat.cycle();
                    }
                    PlaybackCommand::Stop => {
                        status.state = PlaybackState::Stopped;
                        status.current_time_secs = 0.0;
                    }
                }
                let _ = status_tx.send(status.clone()).await;
                *status_store.lock().await = status.clone();
            }
        }
    }
}

async fn run_browser_playback_loop(
    browser_path: PathBuf,
    mut cmd_rx: Receiver<PlaybackCommand>,
    status_tx: Sender<PlaybackStatus>,
    status_store: Arc<Mutex<PlaybackStatus>>,
) -> Result<()> {
    let mut config = BrowserConfig::builder()
        .custom_flags([
            "--autoplay-policy=no-user-gesture-required",
            "--enable-widevine-cdm",
            "--disable-gpu",
            "--no-sandbox",
            "--disable-setuid-sandbox",
        ])
        .with_head() // headless=new mode or normal headless
        .chrome_executable(browser_path)
        .build()
        .map_err(|e| anyhow::anyhow!("BrowserConfig error: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                error!("Browser handler error: {:?}", e);
                break;
            }
        }
    });

    let page = browser.new_page("https://music.apple.com").await?;
    info!("Navigated to Apple Music web player");

    let mut status = PlaybackStatus {
        volume: 80,
        ..Default::default()
    };

    let mut poll_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                // Poll JS state from page
                let js_eval = r#"
                    (() => {
                        try {
                            const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                            if (!mk) return { ok: false };
                            return {
                                ok: true,
                                isPlaying: mk.isPlaying,
                                currentTime: mk.currentPlaybackTime || 0,
                                duration: mk.currentPlaybackDuration || 0,
                                volume: Math.round((mk.volume || 0.8) * 100)
                            };
                        } catch (e) {
                            return { ok: false, error: e.toString() };
                        }
                    })()
                "#;
                if let Ok(eval_result) = page.evaluate(js_eval).await {
                    if let Some(val) = eval_result.into_value::<serde_json::Value>().ok() {
                        if val.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            if let Some(playing) = val.get("isPlaying").and_then(|v| v.as_bool()) {
                                status.state = if playing { PlaybackState::Playing } else if status.current_song.is_some() { PlaybackState::Paused } else { PlaybackState::Stopped };
                            }
                            if let Some(cur) = val.get("currentTime").and_then(|v| v.as_f64()) {
                                status.current_time_secs = cur;
                            }
                            if let Some(dur) = val.get("duration").and_then(|v| v.as_f64()) {
                                if dur > 0.0 {
                                    status.duration_secs = dur;
                                }
                            }
                            let _ = status_tx.send(status.clone()).await;
                            *status_store.lock().await = status.clone();
                        }
                    }
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    PlaybackCommand::PlaySong(song) => {
                        status.current_song = Some(song.clone());
                        status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                        status.current_time_secs = 0.0;
                        status.state = PlaybackState::Playing;

                        let js = format!(
                            "window.MusicKit && window.MusicKit.getInstance().setQueue({{ song: '{}' }}).then(() => window.MusicKit.getInstance().play());",
                            song.id
                        );
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SetQueueAndPlay(songs, idx) => {
                        if idx < songs.len() {
                            let song = &songs[idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;

                            let song_ids: Vec<String> = songs.iter().map(|s| format!("'{}'", s.id)).collect();
                            let js = format!(
                                "window.MusicKit && window.MusicKit.getInstance().setQueue({{ songs: [{}] }}, {}).then(() => window.MusicKit.getInstance().play());",
                                song_ids.join(","), idx
                            );
                            let _ = page.evaluate(js).await;
                        }
                    }
                    PlaybackCommand::TogglePlayPause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                            let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().pause();").await;
                        } else {
                            status.state = PlaybackState::Playing;
                            let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().play();").await;
                        }
                    }
                    PlaybackCommand::Pause => {
                        status.state = PlaybackState::Paused;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().pause();").await;
                    }
                    PlaybackCommand::Resume => {
                        status.state = PlaybackState::Playing;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().play();").await;
                    }
                    PlaybackCommand::Next => {
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().skipToNextItem();").await;
                    }
                    PlaybackCommand::Previous => {
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().skipToPreviousItem();").await;
                    }
                    PlaybackCommand::Seek(pos) => {
                        let js = format!("window.MusicKit && window.MusicKit.getInstance().seekToTime({});", pos);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SeekRelative(delta) => {
                        let new_pos = (status.current_time_secs + delta).max(0.0);
                        let js = format!("window.MusicKit && window.MusicKit.getInstance().seekToTime({});", new_pos);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SetVolume(vol) => {
                        status.volume = vol.min(100);
                        let vol_f = (vol as f64) / 100.0;
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().volume = {};", vol_f);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::ToggleShuffle => {
                        status.shuffle = !status.shuffle;
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().shuffleMode = {};", if status.shuffle { 1 } else { 0 });
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::CycleRepeat => {
                        status.repeat = status.repeat.cycle();
                        let mode_int = match status.repeat {
                            RepeatMode::Off => 0,
                            RepeatMode::All => 1,
                            RepeatMode::One => 2,
                        };
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().repeatMode = {};", mode_int);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::Stop => {
                        status.state = PlaybackState::Stopped;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().stop();").await;
                        break;
                    }
                }
                let _ = status_tx.send(status.clone()).await;
                *status_store.lock().await = status.clone();
            }
        }
    }

    let _ = browser.close().await;
    handler_handle.abort();
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test playback_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/playback/mod.rs src/playback/types.rs src/playback/engine.rs tests/playback_test.rs
git commit -m "feat: implement playback types, headless browser bridge and mock playback engine"
```

---

### Task 6: Application State & Navigation Model

**Files:**
- Create: `src/app/mod.rs`
- Create: `src/app/state.rs`
- Test: `tests/state_test.rs`

**Interfaces:**
- Consumes: `src/api/models.rs`, `src/playback/types.rs`.
- Produces: `AppState`, `ActiveView`, `FocusedPanel`, `ModalState`, state transition methods (`next_item`, `prev_item`, `select_view`, `toggle_focus`).

- [ ] **Step 1: Write failing test in tests/state_test.rs**

```rust
// tests/state_test.rs
use apple_tui::app::state::{AppState, ActiveView, FocusedPanel, ModalState};
use apple_tui::api::models::Song;

#[test]
fn test_state_navigation_and_clamping() {
    let mut state = AppState::new();
    assert_eq!(state.active_view, ActiveView::LibrarySongs);
    assert_eq!(state.focused_panel, FocusedPanel::Sidebar);

    state.songs = vec![
        Song {
            id: "1".to_string(),
            name: "Song A".to_string(),
            artist_name: "Artist A".to_string(),
            album_name: None,
            duration_in_millis: 180000,
            track_number: None,
            release_date: None,
            url: None,
        },
        Song {
            id: "2".to_string(),
            name: "Song B".to_string(),
            artist_name: "Artist B".to_string(),
            album_name: None,
            duration_in_millis: 210000,
            track_number: None,
            release_date: None,
            url: None,
        },
    ];

    state.focused_panel = FocusedPanel::MainContent;
    assert_eq!(state.selected_index, 0);

    state.move_selection_down();
    assert_eq!(state.selected_index, 1);

    // Clamp at end
    state.move_selection_down();
    assert_eq!(state.selected_index, 1);

    state.move_selection_up();
    assert_eq!(state.selected_index, 0);

    // Clamp at start
    state.move_selection_up();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_modal_toggling() {
    let mut state = AppState::new();
    assert_eq!(state.modal, ModalState::None);

    state.open_search();
    assert_eq!(state.modal, ModalState::Search);

    state.close_modal();
    assert_eq!(state.modal, ModalState::None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test state_test`  
Expected: FAIL

- [ ] **Step 3: Implement src/app/mod.rs and src/app/state.rs**

```rust
// src/app/mod.rs
pub mod state;
```

```rust
// src/app/state.rs
use crate::api::models::{Album, Artist, Playlist, SearchResults, Song};
use crate::playback::types::PlaybackStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Search,
    LibrarySongs,
    LibraryAlbums,
    LibraryArtists,
    Playlists,
    PlaylistDetail,
    RecentlyPlayed,
    Queue,
}

impl ActiveView {
    pub fn all_sidebar_views() -> &'static [ActiveView] {
        &[
            ActiveView::Search,
            ActiveView::LibrarySongs,
            ActiveView::LibraryAlbums,
            ActiveView::LibraryArtists,
            ActiveView::Playlists,
            ActiveView::RecentlyPlayed,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ActiveView::Search => "🔍 Search",
            ActiveView::LibrarySongs => "♪ Library Songs",
            ActiveView::LibraryAlbums => "💽 Albums",
            ActiveView::LibraryArtists => "👤 Artists",
            ActiveView::Playlists => "📁 Playlists",
            ActiveView::PlaylistDetail => "📁 Playlist Tracks",
            ActiveView::RecentlyPlayed => "🕒 Recently Played",
            ActiveView::Queue => "📜 Queue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Sidebar,
    MainContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    None,
    Search,
    CreatePlaylist,
    AddToPlaylist { song: Song },
    Help,
    AuthPrompt,
    Notification(String),
}

pub struct AppState {
    pub active_view: ActiveView,
    pub focused_panel: FocusedPanel,
    pub modal: ModalState,

    // Navigation & Lists
    pub sidebar_index: usize,
    pub selected_index: usize,
    pub search_query: String,
    pub search_results: SearchResults,

    // Loaded data
    pub songs: Vec<Song>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
    pub active_playlist: Option<Playlist>,
    pub playlist_tracks: Vec<Song>,
    pub recent_tracks: Vec<Song>,
    pub queue: Vec<Song>,

    // Text inputs for modals
    pub text_input_buffer: String,
    pub add_to_playlist_index: usize,

    // Status & Playback
    pub playback: PlaybackStatus,
    pub storefront: String,
    pub is_authenticated: bool,
    pub status_message: Option<String>,
    pub is_loading: bool,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_view: ActiveView::LibrarySongs,
            focused_panel: FocusedPanel::Sidebar,
            modal: ModalState::None,
            sidebar_index: 1, // Start at Library Songs
            selected_index: 0,
            search_query: String::new(),
            search_results: SearchResults::default(),
            songs: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
            active_playlist: None,
            playlist_tracks: Vec::new(),
            recent_tracks: Vec::new(),
            queue: Vec::new(),
            text_input_buffer: String::new(),
            add_to_playlist_index: 0,
            playback: PlaybackStatus::default(),
            storefront: "us".to_string(),
            is_authenticated: false,
            status_message: None,
            is_loading: false,
            should_quit: false,
        }
    }

    pub fn current_list_len(&self) -> usize {
        match self.active_view {
            ActiveView::Search => self.search_results.songs.len(),
            ActiveView::LibrarySongs => self.songs.len(),
            ActiveView::LibraryAlbums => self.albums.len(),
            ActiveView::LibraryArtists => self.artists.len(),
            ActiveView::Playlists => self.playlists.len(),
            ActiveView::PlaylistDetail => self.playlist_tracks.len(),
            ActiveView::RecentlyPlayed => self.recent_tracks.len(),
            ActiveView::Queue => self.queue.len(),
        }
    }

    pub fn move_selection_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 && self.selected_index + 1 < len {
            self.selected_index += 1;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_sidebar_down(&mut self) {
        let len = ActiveView::all_sidebar_views().len();
        if self.sidebar_index + 1 < len {
            self.sidebar_index += 1;
            self.active_view = ActiveView::all_sidebar_views()[self.sidebar_index];
            self.selected_index = 0;
        }
    }

    pub fn move_sidebar_up(&mut self) {
        if self.sidebar_index > 0 {
            self.sidebar_index -= 1;
            self.active_view = ActiveView::all_sidebar_views()[self.sidebar_index];
            self.selected_index = 0;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Sidebar => FocusedPanel::MainContent,
            FocusedPanel::MainContent => FocusedPanel::Sidebar,
        };
    }

    pub fn open_search(&mut self) {
        self.text_input_buffer.clear();
        self.modal = ModalState::Search;
    }

    pub fn open_create_playlist(&mut self) {
        self.text_input_buffer.clear();
        self.modal = ModalState::CreatePlaylist;
    }

    pub fn open_add_to_playlist(&mut self, song: Song) {
        self.add_to_playlist_index = 0;
        self.modal = ModalState::AddToPlaylist { song };
    }

    pub fn toggle_help(&mut self) {
        self.modal = match self.modal {
            ModalState::Help => ModalState::None,
            _ => ModalState::Help,
        };
    }

    pub fn close_modal(&mut self) {
        self.modal = ModalState::None;
        self.text_input_buffer.clear();
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn get_selected_song(&self) -> Option<Song> {
        let idx = self.selected_index;
        match self.active_view {
            ActiveView::Search => self.search_results.songs.get(idx).cloned(),
            ActiveView::LibrarySongs => self.songs.get(idx).cloned(),
            ActiveView::PlaylistDetail => self.playlist_tracks.get(idx).cloned(),
            ActiveView::RecentlyPlayed => self.recent_tracks.get(idx).cloned(),
            ActiveView::Queue => self.queue.get(idx).cloned(),
            _ => None,
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test state_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/app/mod.rs src/app/state.rs tests/state_test.rs
git commit -m "feat: implement application state, navigation, and modal controllers"
```

---

### Task 7: TUI Views & Rendering Components

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/theme.rs`
- Create: `src/ui/sidebar.rs`
- Create: `src/ui/main_view.rs`
- Create: `src/ui/player_bar.rs`
- Create: `src/ui/modals.rs`
- Test: `tests/ui_render_test.rs`

**Interfaces:**
- Consumes: `ratatui`, `src/app/state.rs`, `src/playback/types.rs`.
- Produces: `ui::draw(&mut Frame, &AppState)`.

- [ ] **Step 1: Write test in tests/ui_render_test.rs using Ratatui TestBackend**

```rust
// tests/ui_render_test.rs
use apple_tui::app::state::AppState;
use apple_tui::ui::draw;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_ui_draw_without_panics() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new();
    state.status_message = Some("Testing UI rendering".to_string());

    terminal.draw(|f| {
        draw(f, &state);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    assert!(buffer.width() == 120);
    assert!(buffer.height() == 40);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test ui_render_test`  
Expected: FAIL

- [ ] **Step 3: Implement Theme and UI Rendering Components**

```rust
// src/ui/theme.rs
use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const BG: Color = Color::Reset;
    pub const ACCENT: Color = Color::Rgb(250, 45, 72);       // Apple Music Pink/Red
    pub const SECONDARY: Color = Color::Rgb(140, 140, 240);  // Soft Purple
    pub const TEXT_PRIMARY: Color = Color::Rgb(240, 240, 245);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 130, 140);
    pub const BORDER_FOCUSED: Color = Color::Rgb(250, 45, 72);
    pub const BORDER_UNFOCUSED: Color = Color::Rgb(60, 60, 70);
    pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 50);

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(focused: bool) -> Style {
        if focused {
            Style::default().fg(Self::BORDER_FOCUSED)
        } else {
            Style::default().fg(Self::BORDER_UNFOCUSED)
        }
    }

    pub fn selected_row_style() -> Style {
        Style::default()
            .bg(Self::HIGHLIGHT_BG)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}
```

```rust
// src/ui/sidebar.rs
use crate::app::state::{ActiveView, AppState, FocusedPanel};
use crate::ui::theme::Theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::Sidebar;
    let views = ActiveView::all_sidebar_views();

    let items: Vec<ListItem> = views
        .iter()
        .enumerate()
        .map(|(idx, view)| {
            let is_selected = idx == state.sidebar_index;
            let symbol = if is_selected { " ▸ " } else { "   " };
            let style = if is_selected {
                Theme::selected_row_style()
            } else {
                ratatui::style::Style::default().fg(Theme::TEXT_PRIMARY)
            };
            ListItem::new(Line::from(vec![
                Span::styled(symbol, ratatui::style::Style::default().fg(Theme::ACCENT)),
                Span::styled(view.display_name(), style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(" Library ", Theme::title_style()))
        .borders(Borders::ALL)
        .border_style(Theme::border_style(focused));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
```

```rust
// src/ui/main_view.rs
use crate::app::state::{ActiveView, AppState, FocusedPanel};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render_main_view(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::MainContent;
    let title = state.active_view.display_name();

    let block = Block::default()
        .title(Span::styled(format!(" {} ", title), Theme::title_style()))
        .borders(Borders::ALL)
        .border_style(Theme::border_style(focused));

    match state.active_view {
        ActiveView::LibrarySongs | ActiveView::PlaylistDetail | ActiveView::RecentlyPlayed | ActiveView::Search | ActiveView::Queue => {
            let songs = match state.active_view {
                ActiveView::LibrarySongs => &state.songs,
                ActiveView::PlaylistDetail => &state.playlist_tracks,
                ActiveView::RecentlyPlayed => &state.recent_tracks,
                ActiveView::Search => &state.search_results.songs,
                ActiveView::Queue => &state.queue,
                _ => &state.songs,
            };

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Title").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Album").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Duration ").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = songs
                .iter()
                .enumerate()
                .map(|(idx, song)| {
                    let is_selected = idx == state.selected_index;
                    let is_playing = state
                        .playback
                        .current_song
                        .as_ref()
                        .map(|s| s.id == song.id)
                        .unwrap_or(false);

                    let num_prefix = if is_playing {
                        "▶".to_string()
                    } else {
                        format!("{:>2}", idx + 1)
                    };

                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else if is_playing {
                        Style::default().fg(Theme::ACCENT)
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {} ", num_prefix)),
                        Cell::from(song.name.clone()),
                        Cell::from(song.artist_name.clone()),
                        Cell::from(song.album_name.clone().unwrap_or_default()),
                        Cell::from(format!(" {} ", song.formatted_duration())),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Length(10),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
        ActiveView::Playlists => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Playlist Name").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Tracks").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Description").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .playlists
                .iter()
                .enumerate()
                .map(|(idx, pl)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
                        Cell::from(pl.name.clone()),
                        Cell::from(pl.track_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string())),
                        Cell::from(pl.description.clone().unwrap_or_default()),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Percentage(50),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
        ActiveView::LibraryAlbums => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Album Name").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Tracks").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .albums
                .iter()
                .enumerate()
                .map(|(idx, alb)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
                        Cell::from(alb.name.clone()),
                        Cell::from(alb.artist_name.clone()),
                        Cell::from(alb.track_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string())),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(45),
                Constraint::Percentage(35),
                Constraint::Length(10),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
        ActiveView::LibraryArtists => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist Name").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .artists
                .iter()
                .enumerate()
                .map(|(idx, art)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
                        Cell::from(art.name.clone()),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(90),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
    }
}
```

```rust
// src/ui/player_bar.rs
use crate::app::state::AppState;
use crate::playback::types::PlaybackState;
use crate::ui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub fn render_player_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_UNFOCUSED));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Song Info & Time
            Constraint::Length(1), // Progress Bar
            Constraint::Length(1), // Controls & Volume
        ])
        .split(inner);

    // 1. Song Info
    let (track_title, artist_album) = if let Some(song) = &state.playback.current_song {
        (
            song.name.clone(),
            format!("{} • {}", song.artist_name, song.album_name.as_deref().unwrap_or("Single")),
        )
    } else {
        ("No track playing".to_string(), "Select a song and press Enter to play".to_string())
    };

    let status_icon = match state.playback.state {
        PlaybackState::Playing => " ▶ ",
        PlaybackState::Paused => " ⏸ ",
        PlaybackState::Loading => " ⌛ ",
        PlaybackState::Stopped => " ⏹ ",
    };

    let time_info = state.playback.formatted_position();

    let info_line = Line::from(vec![
        Span::styled(status_icon, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", track_title), Style::default().fg(Theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("— {} ", artist_album), Style::default().fg(Theme::TEXT_MUTED)),
    ]);

    let time_line = Line::from(vec![
        Span::styled(time_info, Style::default().fg(Theme::TEXT_PRIMARY)),
    ]);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(chunks[0]);

    f.render_widget(Paragraph::new(info_line), row1[0]);
    f.render_widget(Paragraph::new(time_line).alignment(Alignment::Right), row1[1]);

    // 2. Progress Gauge
    let ratio = state.playback.progress_ratio();
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Theme::ACCENT).bg(Color::Rgb(40, 40, 50)))
        .ratio(ratio)
        .label("");
    f.render_widget(gauge, chunks[1]);

    // 3. Controls and Volume
    let shuffle_style = if state.playback.shuffle {
        Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let repeat_style = if state.playback.repeat != crate::playback::types::RepeatMode::Off {
        Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let controls_line = Line::from(vec![
        Span::raw("[p] ◁◁  "),
        Span::raw("[Space] ▶/⏸  "),
        Span::raw("[n] ▷▷   "),
        Span::styled(format!("[s] 🔀 Shuffle: {}   ", if state.playback.shuffle { "On" } else { "Off" }), shuffle_style),
        Span::styled(format!("[r] 🔁 Repeat: {}   ", state.playback.repeat.display_label()), repeat_style),
        Span::raw(format!("[+/-] 🔊 Volume: {}%", state.playback.volume)),
    ]);

    f.render_widget(Paragraph::new(controls_line), chunks[2]);
}
```

```rust
// src/ui/modals.rs
use crate::app::state::{AppState, ModalState};
use crate::ui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_modals(f: &mut Frame, area: Rect, state: &AppState) {
    match &state.modal {
        ModalState::None => {}
        ModalState::Search => {
            let popup = centered_rect(60, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Search Apple Music ", Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::ACCENT));

            let input_text = format!(" Query: {}█", state.text_input_buffer);
            let instructions = "\n Press [Enter] to Search, [Esc] to Cancel";

            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(input_text, Style::default().fg(Theme::TEXT_PRIMARY))),
                Line::from(Span::styled(instructions, Style::default().fg(Theme::TEXT_MUTED))),
            ])
            .block(block);

            f.render_widget(paragraph, popup);
        }
        ModalState::CreatePlaylist => {
            let popup = centered_rect(50, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Create New Playlist ", Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::ACCENT));

            let input_text = format!(" Name: {}█", state.text_input_buffer);
            let instructions = "\n Press [Enter] to Confirm, [Esc] to Cancel";

            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(input_text, Style::default().fg(Theme::TEXT_PRIMARY))),
                Line::from(Span::styled(instructions, Style::default().fg(Theme::TEXT_MUTED))),
            ])
            .block(block);

            f.render_widget(paragraph, popup);
        }
        ModalState::AddToPlaylist { song } => {
            let popup = centered_rect(50, 40, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(format!(" Add '{}' to Playlist ", song.name), Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::ACCENT));

            let items: Vec<ListItem> = state
                .playlists
                .iter()
                .enumerate()
                .map(|(idx, pl)| {
                    let is_sel = idx == state.add_to_playlist_index;
                    let style = if is_sel {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };
                    ListItem::new(format!("  {} {}", if is_sel { "▸" } else { " " }, pl.name)).style(style)
                })
                .collect();

            let list = List::new(items).block(block);
            f.render_widget(list, popup);
        }
        ModalState::Help => {
            let popup = centered_rect(70, 60, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Keyboard Shortcuts & Help ", Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::SECONDARY));

            let help_text = vec![
                Line::from(Span::styled("Navigation:", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
                Line::from("  ↑ / k, ↓ / j     : Move selection up / down"),
                Line::from("  ← / h, → / l     : Focus Sidebar / Main View"),
                Line::from("  Tab              : Toggle panel focus"),
                Line::from("  Enter            : Play song / Open playlist"),
                Line::from("  Esc              : Close popup / Return"),
                Line::from(""),
                Line::from(Span::styled("Playback:", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
                Line::from("  Space            : Toggle Play / Pause"),
                Line::from("  n / p            : Next / Previous track"),
                Line::from("  [ / ]            : Seek -10s / +10s"),
                Line::from("  + / -            : Volume Up / Down"),
                Line::from("  s / r            : Toggle Shuffle / Cycle Repeat"),
                Line::from(""),
                Line::from(Span::styled("Features & Actions:", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
                Line::from("  /                : Open Catalog Search"),
                Line::from("  c                : Create new playlist (in Playlists view)"),
                Line::from("  a                : Add selected track to playlist"),
                Line::from("  ?                : Toggle this help overlay"),
                Line::from("  q / Ctrl+C       : Quit application"),
            ];

            let paragraph = Paragraph::new(help_text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, popup);
        }
        ModalState::Notification(msg) => {
            let popup = centered_rect(50, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Notification ", Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::ACCENT));

            let paragraph = Paragraph::new(format!("\n {}\n\n Press [Esc] to dismiss", msg))
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, popup);
        }
        ModalState::AuthPrompt => {
            let popup = centered_rect(60, 25, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Apple Music Login Required ", Theme::title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::ACCENT));

            let text = vec![
                Line::from(Span::styled("You are not logged in to Apple Music.", Style::default().fg(Theme::TEXT_PRIMARY))),
                Line::from(""),
                Line::from("To access your library and stream full tracks:"),
                Line::from("  1. Press [L] to launch browser login window"),
                Line::from("  2. Or run: apple-tui --set-user-token <TOKEN>"),
                Line::from(""),
                Line::from(Span::styled("Press [Esc] to continue in preview / mock mode.", Style::default().fg(Theme::TEXT_MUTED))),
            ];

            let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);
            f.render_widget(paragraph, popup);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

```rust
// src/ui/mod.rs
pub mod main_view;
pub mod modals;
pub mod player_bar;
pub mod sidebar;
pub mod theme;

use crate::app::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(f: &mut Frame, state: &AppState) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),     // Header Status Bar
            Constraint::Min(10),       // Content Area (Sidebar + Main)
            Constraint::Length(4),     // Bottom Player Bar
        ])
        .split(size);

    // 1. Header Bar
    render_header(f, main_chunks[0], state);

    // 2. Main Content Split
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22), // Sidebar
            Constraint::Percentage(78), // Main Table
        ])
        .split(main_chunks[1]);

    sidebar::render_sidebar(f, content_chunks[0], state);
    main_view::render_main_view(f, content_chunks[1], state);

    // 3. Bottom Player Bar
    player_bar::render_player_bar(f, main_chunks[2], state);

    // 4. Modals / Popups
    modals::render_modals(f, size, state);
}

fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let auth_status = if state.is_authenticated {
        Span::styled(" [● Authenticated] ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" [○ Unauthenticated / Mock Mode] ", Style::default().fg(Color::Yellow))
    };

    let status_text = state
        .status_message
        .as_deref()
        .unwrap_or("Press '?' for Help | '/' to Search | 'q' to Quit");

    let left_header = Line::from(vec![
        Span::styled(" appleTUI ", Style::default().fg(theme::Theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("v0.1.0 ", Style::default().fg(theme::Theme::TEXT_MUTED)),
        Span::styled(format!("Storefront: {} ", state.storefront.to_uppercase()), Style::default().fg(theme::Theme::TEXT_MUTED)),
        auth_status,
    ]);

    let right_header = Line::from(vec![
        Span::styled(status_text, Style::default().fg(theme::Theme::TEXT_MUTED)),
    ]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    f.render_widget(Paragraph::new(left_header), chunks[0]);
    f.render_widget(Paragraph::new(right_header).alignment(ratatui::layout::Alignment::Right), chunks[1]);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test ui_render_test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/ui/ tests/ui_render_test.rs
git commit -m "feat: implement ratatui UI layout, sidebar, player bar, and modal rendering"
```

---

### Task 8: Authentication Helper & Token Capture

**Files:**
- Create: `src/auth/mod.rs`
- Create: `src/auth/login.rs`
- Test: `tests/auth_test.rs`

**Interfaces:**
- Consumes: `src/config.rs`, `chromiumoxide`.
- Produces: `launch_interactive_login()`, `validate_auth_tokens()`.

- [ ] **Step 1: Write test in tests/auth_test.rs**

```rust
// tests/auth_test.rs
use apple_tui::config::AuthConfig;

#[test]
fn test_auth_config_validity() {
    let mut auth = AuthConfig::default();
    assert!(!auth.is_authenticated());

    auth.music_user_token = Some("valid_token".to_string());
    assert!(auth.is_authenticated());
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --test auth_test`  
Expected: PASS

- [ ] **Step 3: Implement src/auth/mod.rs and src/auth/login.rs**

```rust
// src/auth/mod.rs
pub mod login;
```

```rust
// src/auth/login.rs
use crate::config::{find_browser_binary, AuthConfig};
use anyhow::{bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::time::Duration;
use tracing::{info, warn};

pub async fn launch_interactive_login() -> Result<AuthConfig> {
    let browser_bin = find_browser_binary()
        .context("No supported Chromium/Brave browser found for login flow")?;

    info!("Launching browser for Apple Music login: {:?}", browser_bin);

    // Launch non-headless browser window for user to sign in
    let config = BrowserConfig::builder()
        .chrome_executable(browser_bin)
        .build()
        .map_err(|e| anyhow::anyhow!("Browser config error: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                warn!("Login browser event error: {:?}", e);
                break;
            }
        }
    });

    let page = browser.new_page("https://music.apple.com/login").await?;
    info!("Opened login page. Waiting for user authorization...");

    let mut user_token: Option<String> = None;
    let poll_limit = 120; // 2 minutes timeout
    for _ in 0..poll_limit {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let cookies = page.get_cookies().await?;
        for cookie in cookies {
            if cookie.name == "media-user-token" {
                user_token = Some(cookie.value);
                break;
            }
        }

        if user_token.is_some() {
            info!("Captured media-user-token!");
            break;
        }
    }

    let _ = browser.close().await;
    handler_handle.abort();

    if let Some(token) = user_token {
        let auth = AuthConfig {
            developer_token: None, // Will use default fallback developer token
            music_user_token: Some(token),
        };
        auth.save()?;
        Ok(auth)
    } else {
        bail!("Login timed out or user token was not captured");
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`  
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/auth/ tests/auth_test.rs
git commit -m "feat: add interactive Apple Music login helper and cookie extractor"
```

---

### Task 9: Event Loop, Signal Handling, Terminal Restoration & Main Binary

**Files:**
- Create: `src/events.rs`
- Modify: `src/main.rs`
- Test: `tests/integration_test.rs`

**Interfaces:**
- Consumes: All previous modules (`config`, `api`, `playback`, `app`, `ui`, `auth`).
- Produces: Executable `apple-tui` binary with CLI flags, input handling, terminal safety, and clean shutdown.

- [ ] **Step 1: Write integration test in tests/integration_test.rs**

```rust
// tests/integration_test.rs
use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::state::AppState;
use apple_tui::config::Config;
use apple_tui::playback::engine::PlaybackEngine;
use apple_tui::playback::types::PlaybackCommand;

#[tokio::test]
async fn test_end_to_end_mock_pipeline() {
    let config = Config { mock_mode: true, ..Default::default() };
    let client = AppleMusicClient::new_mock();
    let playback = PlaybackEngine::new(None, true).await.unwrap();

    let mut state = AppState::new();
    state.songs = client.get_library_songs(10, 0).await.unwrap();
    assert!(!state.songs.is_empty());

    let song_to_play = state.songs[0].clone();
    playback.send_command(PlaybackCommand::PlaySong(song_to_play.clone())).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let status = playback.get_current_status().await;
    assert_eq!(status.current_song.as_ref().map(|s| &s.id), Some(&song_to_play.id));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test integration_test`  
Expected: FAIL

- [ ] **Step 3: Implement src/events.rs and src/main.rs**

```rust
// src/events.rs
use crate::api::client::AppleMusicClient;
use crate::app::state::{ActiveView, AppState, FocusedPanel, ModalState};
use crate::playback::engine::PlaybackEngine;
use crate::playback::types::PlaybackCommand;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_event(
    key: KeyEvent,
    state: &mut AppState,
    client: &AppleMusicClient,
    playback: &PlaybackEngine,
) -> Result<()> {
    // 1. Handle Active Modals First
    match &mut state.modal {
        ModalState::Search => match key.code {
            KeyCode::Enter => {
                let query = state.text_input_buffer.trim().to_string();
                if !query.is_empty() {
                    state.search_query = query.clone();
                    state.set_status(format!("Searching for '{}'...", query));
                    if let Ok(results) = client.search_catalog(&query, &state.storefront).await {
                        state.search_results = results;
                        state.active_view = ActiveView::Search;
                        state.selected_index = 0;
                        state.focused_panel = FocusedPanel::MainContent;
                        state.set_status(format!("Search results for '{}'", query));
                    } else {
                        state.set_status("Search failed");
                    }
                }
                state.close_modal();
                return Ok(());
            }
            KeyCode::Esc => {
                state.close_modal();
                return Ok(());
            }
            KeyCode::Backspace => {
                state.text_input_buffer.pop();
                return Ok(());
            }
            KeyCode::Char(c) => {
                state.text_input_buffer.push(c);
                return Ok(());
            }
            _ => return Ok(()),
        },
        ModalState::CreatePlaylist => match key.code {
            KeyCode::Enter => {
                let name = state.text_input_buffer.trim().to_string();
                if !name.is_empty() {
                    state.set_status(format!("Creating playlist '{}'...", name));
                    if let Ok(pl) = client.create_playlist(&name, None).await {
                        state.playlists.push(pl);
                        state.set_status(format!("Created playlist '{}'", name));
                    }
                }
                state.close_modal();
                return Ok(());
            }
            KeyCode::Esc => {
                state.close_modal();
                return Ok(());
            }
            KeyCode::Backspace => {
                state.text_input_buffer.pop();
                return Ok(());
            }
            KeyCode::Char(c) => {
                state.text_input_buffer.push(c);
                return Ok(());
            }
            _ => return Ok(()),
        },
        ModalState::AddToPlaylist { song } => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if state.add_to_playlist_index > 0 {
                    state.add_to_playlist_index -= 1;
                }
                return Ok(());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !state.playlists.is_empty() && state.add_to_playlist_index + 1 < state.playlists.len() {
                    state.add_to_playlist_index += 1;
                }
                return Ok(());
            }
            KeyCode::Enter => {
                if let Some(pl) = state.playlists.get(state.add_to_playlist_index) {
                    let pl_id = pl.id.clone();
                    let song_id = song.id.clone();
                    let song_name = song.name.clone();
                    let pl_name = pl.name.clone();
                    let _ = client.add_tracks_to_playlist(&pl_id, &[&song_id]).await;
                    state.set_status(format!("Added '{}' to '{}'", song_name, pl_name));
                }
                state.close_modal();
                return Ok(());
            }
            KeyCode::Esc => {
                state.close_modal();
                return Ok(());
            }
            _ => return Ok(()),
        },
        ModalState::Help | ModalState::Notification(_) | ModalState::AuthPrompt => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Enter {
                state.close_modal();
                return Ok(());
            }
        }
        ModalState::None => {}
    }

    // 2. Global Hotkeys
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('?') => {
            state.toggle_help();
            return Ok(());
        }
        KeyCode::Char('/') => {
            state.open_search();
            return Ok(());
        }
        KeyCode::Tab => {
            state.toggle_focus();
            return Ok(());
        }
        // Playback hotkeys
        KeyCode::Char(' ') => {
            playback.send_command(PlaybackCommand::TogglePlayPause).await?;
            return Ok(());
        }
        KeyCode::Char('n') => {
            playback.send_command(PlaybackCommand::Next).await?;
            return Ok(());
        }
        KeyCode::Char('p') => {
            playback.send_command(PlaybackCommand::Previous).await?;
            return Ok(());
        }
        KeyCode::Char('[') => {
            playback.send_command(PlaybackCommand::SeekRelative(-10.0)).await?;
            return Ok(());
        }
        KeyCode::Char(']') => {
            playback.send_command(PlaybackCommand::SeekRelative(10.0)).await?;
            return Ok(());
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let new_vol = (state.playback.volume + 5).min(100);
            state.playback.volume = new_vol;
            playback.send_command(PlaybackCommand::SetVolume(new_vol)).await?;
            return Ok(());
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            let new_vol = state.playback.volume.saturating_sub(5);
            state.playback.volume = new_vol;
            playback.send_command(PlaybackCommand::SetVolume(new_vol)).await?;
            return Ok(());
        }
        KeyCode::Char('s') => {
            playback.send_command(PlaybackCommand::ToggleShuffle).await?;
            return Ok(());
        }
        KeyCode::Char('r') => {
            playback.send_command(PlaybackCommand::CycleRepeat).await?;
            return Ok(());
        }
        _ => {}
    }

    // 3. Navigation Controls
    match state.focused_panel {
        FocusedPanel::Sidebar => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_sidebar_up();
                load_view_data(state, client).await?;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_sidebar_down();
                load_view_data(state, client).await?;
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                state.focused_panel = FocusedPanel::MainContent;
                load_view_data(state, client).await?;
            }
            _ => {}
        },
        FocusedPanel::MainContent => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_selection_down();
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                if state.active_view == ActiveView::PlaylistDetail {
                    state.active_view = ActiveView::Playlists;
                } else {
                    state.focused_panel = FocusedPanel::Sidebar;
                }
            }
            KeyCode::Enter => {
                match state.active_view {
                    ActiveView::LibrarySongs | ActiveView::PlaylistDetail | ActiveView::RecentlyPlayed | ActiveView::Search | ActiveView::Queue => {
                        if let Some(song) = state.get_selected_song() {
                            let list = match state.active_view {
                                ActiveView::LibrarySongs => state.songs.clone(),
                                ActiveView::PlaylistDetail => state.playlist_tracks.clone(),
                                ActiveView::RecentlyPlayed => state.recent_tracks.clone(),
                                ActiveView::Search => state.search_results.songs.clone(),
                                ActiveView::Queue => state.queue.clone(),
                                _ => vec![song.clone()],
                            };
                            playback.send_command(PlaybackCommand::SetQueueAndPlay(list, state.selected_index)).await?;
                        }
                    }
                    ActiveView::Playlists => {
                        if let Some(pl) = state.playlists.get(state.selected_index).cloned() {
                            state.active_playlist = Some(pl.clone());
                            state.set_status(format!("Loading playlist '{}'...", pl.name));
                            if let Ok(tracks) = client.get_playlist_tracks(&pl.id).await {
                                state.playlist_tracks = tracks;
                                state.active_view = ActiveView::PlaylistDetail;
                                state.selected_index = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('a') => {
                if let Some(song) = state.get_selected_song() {
                    state.open_add_to_playlist(song);
                }
            }
            KeyCode::Char('c') => {
                if state.active_view == ActiveView::Playlists {
                    state.open_create_playlist();
                }
            }
            _ => {}
        },
    }

    Ok(())
}

async fn load_view_data(state: &mut AppState, client: &AppleMusicClient) -> Result<()> {
    match state.active_view {
        ActiveView::LibrarySongs => {
            if state.songs.is_empty() {
                if let Ok(songs) = client.get_library_songs(100, 0).await {
                    state.songs = songs;
                }
            }
        }
        ActiveView::LibraryAlbums => {
            if state.albums.is_empty() {
                if let Ok(albums) = client.get_library_albums(100, 0).await {
                    state.albums = albums;
                }
            }
        }
        ActiveView::LibraryArtists => {
            if state.artists.is_empty() {
                if let Ok(artists) = client.get_library_artists(100, 0).await {
                    state.artists = artists;
                }
            }
        }
        ActiveView::Playlists => {
            if state.playlists.is_empty() {
                if let Ok(playlists) = client.get_library_playlists().await {
                    state.playlists = playlists;
                }
            }
        }
        ActiveView::RecentlyPlayed => {
            if state.recent_tracks.is_empty() {
                if let Ok(recent) = client.get_recent_tracks().await {
                    state.recent_tracks = recent;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
```

```rust
// src/main.rs
use anyhow::Result;
use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::state::{ActiveView, AppState};
use apple_tui::auth::login::launch_interactive_login;
use apple_tui::config::{AuthConfig, Config};
use apple_tui::events::handle_key_event;
use apple_tui::playback::engine::PlaybackEngine;
use apple_tui::playback::types::PlaybackCommand;
use apple_tui::ui::draw;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "apple-tui", version = "0.1.0", about = "Fast, lightweight Apple Music TUI for Linux")]
struct Cli {
    /// Run with mock offline data
    #[arg(short, long)]
    mock: bool,

    /// Specify custom Chrome or Chromium binary path
    #[arg(short, long)]
    browser_path: Option<PathBuf>,

    /// Set Apple Music Developer Token
    #[arg(long)]
    set_dev_token: Option<String>,

    /// Set Apple Music User Token (media-user-token)
    #[arg(long)]
    set_user_token: Option<String>,

    /// Launch interactive login in browser
    #[arg(long)]
    login: bool,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle token setting CLI flags
    if let Some(token) = cli.set_user_token {
        let mut auth = AuthConfig::load();
        auth.music_user_token = Some(token);
        auth.save()?;
        println!("Saved user token to auth.json");
        return Ok(());
    }

    if let Some(token) = cli.set_dev_token {
        let mut auth = AuthConfig::load();
        auth.developer_token = Some(token);
        auth.save()?;
        println!("Saved developer token to auth.json");
        return Ok(());
    }

    if cli.login {
        println!("Launching browser login window...");
        let auth = launch_interactive_login().await?;
        println!("Successfully authenticated! Token stored in config.");
        return Ok(());
    }

    let mut config = Config::load();
    if cli.mock {
        config.mock_mode = true;
    }
    if let Some(p) = cli.browser_path {
        config.browser_path = Some(p);
    }

    let auth = AuthConfig::load();
    let is_auth = auth.is_authenticated() || config.mock_mode;

    let client = if config.mock_mode {
        AppleMusicClient::new_mock()
    } else {
        AppleMusicClient::new(auth.developer_token.clone(), auth.music_user_token.clone())
            .unwrap_or_else(|_| AppleMusicClient::new_mock())
    };

    let playback = PlaybackEngine::new(config.browser_path.clone(), config.mock_mode).await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let _guard = TerminalGuard;

    // Set panic hook for safety
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        eprintln!("Panic occurred: {:?}", info);
    }));

    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    state.is_authenticated = is_auth;
    state.storefront = client.get_storefront().await.unwrap_or_else(|_| "us".to_string());
    state.volume = config.volume;

    // Preload initial library songs
    if let Ok(songs) = client.get_library_songs(50, 0).await {
        state.songs = songs;
    }
    if let Ok(playlists) = client.get_library_playlists().await {
        state.playlists = playlists;
    }

    let mut event_stream = EventStream::new();
    let status_rx_arc = playback.get_status_receiver();
    let mut ticker = tokio::time::interval(Duration::from_millis(config.tick_rate_ms));

    while !state.should_quit {
        terminal.draw(|f| {
            draw(f, &state);
        })?;

        tokio::select! {
            _ = ticker.tick() => {
                // Periodically update state from playback engine
                state.playback = playback.get_current_status().await;
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    handle_key_event(key, &mut state, &client, &playback).await?;
                }
            }
            status_opt = async {
                let mut rx = status_rx_arc.lock().await;
                rx.recv().await
            } => {
                if let Some(status) = status_opt {
                    state.playback = status;
                }
            }
        }
    }

    // Stop playback and cleanup
    let _ = playback.send_command(PlaybackCommand::Stop).await;
    drop(_guard);

    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test integration_test`  
Expected: PASS

- [ ] **Step 5: Run full test suite and check compilation**

Run: `cargo test --all-targets`  
Expected: PASS

- [ ] **Step 6: Commit changes**

```bash
git add src/events.rs src/main.rs tests/integration_test.rs
git commit -m "feat: implement main event loop, key handling, terminal guard, and integration test"
```

---

### Task 10: Complete Verification & Release Build

**Files:**
- Modify: `README.md`
- Test: Full cargo test suite & release build check

- [ ] **Step 1: Create README.md documenting features, installation, keybindings, and usage**

- [ ] **Step 2: Run cargo fmt, cargo clippy, cargo test, and cargo build --release**

Run: `cargo test`  
Run: `cargo build --release`  
Expected: Binary `target/release/apple-tui` built successfully with zero warnings/errors.

- [ ] **Step 3: Test CLI help and mock mode execution**

Run: `./target/release/apple-tui --help`  
Expected: Displays all options correctly.

- [ ] **Step 4: Final commit**

```bash
git add README.md
git commit -m "docs: add README with architecture, controls, and usage instructions"
```
