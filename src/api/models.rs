use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
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

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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

        Ok(Song {
            id: raw.id,
            name: attrs.name.unwrap_or_else(|| "Unknown Title".to_string()),
            artist_name: attrs
                .artist_name
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            album_name: attrs.album_name,
            duration_in_millis: attrs.duration_in_millis.unwrap_or(0),
            track_number: attrs.track_number,
            release_date: attrs.release_date,
            url: attrs.url,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist_name: String,
    pub track_count: Option<u32>,
    pub release_date: Option<String>,
}

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

impl<'de> Deserialize<'de> for Album {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
            artist_name: attrs
                .artist_name
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            track_count: attrs.track_count,
            release_date: attrs.release_date,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

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

impl<'de> Deserialize<'de> for Artist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub track_count: Option<u32>,
}

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

impl<'de> Deserialize<'de> for Playlist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
            name: attrs
                .name
                .unwrap_or_else(|| "Untitled Playlist".to_string()),
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
