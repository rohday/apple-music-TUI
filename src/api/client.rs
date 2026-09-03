use crate::api::models::{
    Album, Artist, Playlist, RawCatalogResponse, RawListResponse, SearchResults, Song,
};
use crate::config::DEFAULT_FALLBACK_DEVELOPER_TOKEN;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

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

        #[derive(Default, serde::Deserialize)]
        struct StorefrontItem {
            id: String,
        }
        let list: RawListResponse<StorefrontItem> = resp.json().await?;
        Ok(list
            .data
            .into_iter()
            .next()
            .map(|s| s.id)
            .unwrap_or_else(|| "us".to_string()))
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

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

        let raw: RawCatalogResponse = resp
            .json()
            .await
            .context("Failed to parse catalog search JSON")?;
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
            .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await
            .context("Failed to fetch library songs")?;

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

        let list: RawListResponse<Song> =
            resp.json().await.context("Failed to parse library songs")?;
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
            .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await?;

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

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
            .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await?;

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

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

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

        let list: RawListResponse<Playlist> =
            resp.json().await.context("Failed to parse playlists")?;
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

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

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
        list.data
            .into_iter()
            .next()
            .context("No playlist returned in response")
    }

    pub async fn add_tracks_to_playlist(
        &self,
        playlist_id: &str,
        track_ids: &[&str],
    ) -> Result<()> {
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

    pub async fn delete_playlist_track(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<()> {
        if self.mock_mode {
            return Ok(());
        }

        let url = format!("{}/me/library/playlists/{}/tracks/{}", BASE_URL, playlist_id, track_id);
        let resp = self
            .client
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to delete track from playlist")?;

        if !resp.status().is_success() {
            bail!("API error deleting track: status {}", resp.status());
        }

        Ok(())
    }

    pub async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Song>> {
        if self.mock_mode {
            return Ok(mock_library_songs()
                .into_iter()
                .take(4)
                .map(|mut s| {
                    s.album_name = Some("Selected Album".to_string());
                    s
                })
                .collect());
        }

        let url = format!("{}/me/library/albums/{}/tracks", BASE_URL, album_id);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch album tracks")?;

        if !resp.status().is_success() {
            bail!("API error fetching album tracks: status {}", resp.status());
        }

        let list: RawListResponse<Song> = resp.json().await?;
        Ok(list.data)
    }

    pub async fn get_artist_tracks(&self, artist_id: &str) -> Result<Vec<Song>> {
        if self.mock_mode {
            return Ok(mock_library_songs()
                .into_iter()
                .take(6)
                .map(|mut s| {
                    s.artist_name = "Selected Artist".to_string();
                    s
                })
                .collect());
        }

        let url = format!("{}/me/library/artists/{}/tracks", BASE_URL, artist_id);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to fetch artist tracks")?;

        if !resp.status().is_success() {
            bail!("API error fetching artist tracks: status {}", resp.status());
        }

        let list: RawListResponse<Song> = resp.json().await?;
        Ok(list.data)
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

        if !resp.status().is_success() {
            bail!("Apple Music API error: status {}", resp.status());
        }

        let list: RawListResponse<Song> = resp.json().await?;
        Ok(list.data)
    }

    pub async fn create_station_for_song(&self, song_id: &str, storefront: &str) -> Result<Vec<Song>> {
        if self.mock_mode {
            let base = mock_library_songs();
            let station: Vec<Song> = base
                .into_iter()
                .cycle()
                .take(10)
                .enumerate()
                .map(|(i, mut s)| {
                    s.id = format!("station_{song_id}_{i}");
                    s.name = format!("{} (Station Radio {})", s.name, i + 1);
                    s
                })
                .collect();
            return Ok(station);
        }

        let url = format!("{}/catalog/{}/songs/{}/station", BASE_URL, storefront, song_id);
        let resp = self.client.get(&url).headers(self.auth_headers()).send().await;

        if let Ok(resp) = resp {
            if resp.status().is_success() {
                if let Ok(list) = resp.json::<RawListResponse<Song>>().await {
                    if !list.data.is_empty() {
                        return Ok(list.data);
                    }
                }
            }
        }

        self.get_recent_tracks().await
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
                catalog_id: None,
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
                catalog_id: None,
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
                catalog_id: None,
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
        playlists: vec![Playlist {
            id: "4001".to_string(),
            name: "The Weeknd Essentials".to_string(),
            description: Some("The essential tracks from Abel Tesfaye.".to_string()),
            is_public: true,
            track_count: Some(25),
        }],
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
            catalog_id: None,
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
            catalog_id: None,
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
            catalog_id: None,
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
            catalog_id: None,
        });
    }
    songs
}
