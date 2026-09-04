use crate::api::models::{Album, Artist, Playlist, Song};
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(600);

struct Cached<T> {
    data: Vec<T>,
    at: Instant,
}

impl<T> Cached<T> {
    fn new(data: Vec<T>) -> Self {
        Self {
            data,
            at: Instant::now(),
        }
    }

    fn fresh(&self) -> bool {
        self.at.elapsed() < CACHE_TTL
    }
}

/// In-memory TTL cache for library views. Keeps view switches instant while a
/// background refresh can still be requested separately.
#[derive(Default)]
pub struct DataCache {
    songs: Option<Cached<Song>>,
    albums: Option<Cached<Album>>,
    artists: Option<Cached<Artist>>,
    playlists: Option<Cached<Playlist>>,
    recent: Option<Cached<Song>>,
}

impl DataCache {
    pub fn get_songs(&self) -> Option<&[Song]> {
        self.songs
            .as_ref()
            .filter(|c| c.fresh())
            .map(|c| &c.data[..])
    }

    pub fn get_albums(&self) -> Option<&[Album]> {
        self.albums
            .as_ref()
            .filter(|c| c.fresh())
            .map(|c| &c.data[..])
    }

    pub fn get_artists(&self) -> Option<&[Artist]> {
        self.artists
            .as_ref()
            .filter(|c| c.fresh())
            .map(|c| &c.data[..])
    }

    pub fn get_playlists(&self) -> Option<&[Playlist]> {
        self.playlists
            .as_ref()
            .filter(|c| c.fresh())
            .map(|c| &c.data[..])
    }

    pub fn get_recent(&self) -> Option<&[Song]> {
        self.recent
            .as_ref()
            .filter(|c| c.fresh())
            .map(|c| &c.data[..])
    }

    pub fn insert_songs(&mut self, songs: &[Song]) {
        self.songs = Some(Cached::new(songs.to_vec()));
    }

    pub fn insert_albums(&mut self, albums: &[Album]) {
        self.albums = Some(Cached::new(albums.to_vec()));
    }

    pub fn insert_artists(&mut self, artists: &[Artist]) {
        self.artists = Some(Cached::new(artists.to_vec()));
    }

    pub fn insert_playlists(&mut self, playlists: &[Playlist]) {
        self.playlists = Some(Cached::new(playlists.to_vec()));
    }

    pub fn insert_recent(&mut self, recent: &[Song]) {
        self.recent = Some(Cached::new(recent.to_vec()));
    }

    pub fn invalidate_playlists(&mut self) {
        self.playlists = None;
    }

    pub fn invalidate_all(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cache_returns_none() {
        let cache = DataCache::default();
        assert!(cache.get_songs().is_none());
        assert!(cache.get_albums().is_none());
        assert!(cache.get_playlists().is_none());
    }

    #[test]
    fn insert_then_get_roundtrip() {
        let mut cache = DataCache::default();
        cache.insert_playlists(&[Playlist {
            id: "p1".into(),
            name: "Test".into(),
            description: None,
            is_public: false,
            track_count: None,
        }]);
        let playlists = cache.get_playlists().unwrap();
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name, "Test");
    }

    #[test]
    fn invalidate_playlists_only_clears_playlists() {
        let mut cache = DataCache::default();
        cache.insert_playlists(&[]);
        cache.insert_songs(&[]);
        cache.invalidate_playlists();
        assert!(cache.get_playlists().is_none());
        assert!(cache.get_songs().is_some());
    }
}
