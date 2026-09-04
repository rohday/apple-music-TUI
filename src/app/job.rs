use crate::api::client::AppleMusicClient;
use crate::api::lyrics::LyricsData;
use crate::api::models::{Album, Artist, Playlist, SearchResults, Song};
use crate::app::state::{ActiveView, AppState};
use anyhow::Result;

/// Background work requested by the UI thread. Key handling never awaits the
/// network; it enqueues jobs, and the main loop executes them asynchronously.
#[derive(Debug, Clone)]
pub enum Job {
    Search {
        query: String,
        storefront: String,
    },
    LoadView(ActiveView),
    RefreshView(ActiveView),
    PlaylistTracks {
        playlist_id: String,
    },
    AlbumTracks {
        album_id: String,
    },
    ArtistTracks {
        artist_id: String,
    },
    Station {
        song_id: String,
        song_name: String,
        storefront: String,
    },
    Lyrics(Song),
    CreatePlaylist(String),
    AddToPlaylist {
        playlist_id: String,
        song_id: String,
        song_name: String,
        playlist_name: String,
    },
    RemovePlaylistTrack {
        playlist_id: String,
        track_id: String,
        track_name: String,
        index: usize,
    },
    NextSongsPage {
        limit: usize,
        offset: usize,
    },
    FetchArtwork {
        song_id: String,
        url: String,
    },
}

/// Result of a completed [`Job`], delivered back to the UI thread.
#[derive(Debug, Clone)]
pub enum Effect {
    SearchDone(SearchResults),
    SongsLoaded(Vec<Song>),
    AlbumsLoaded(Vec<Album>),
    ArtistsLoaded(Vec<Artist>),
    PlaylistsLoaded(Vec<Playlist>),
    RecentLoaded(Vec<Song>),
    SongsPageLoaded {
        offset: usize,
        songs: Vec<Song>,
        has_more: bool,
    },
    PlaylistTracksLoaded {
        playlist_id: String,
        tracks: Vec<Song>,
    },
    AlbumTracksLoaded {
        album_id: String,
        tracks: Vec<Song>,
    },
    ArtistTracksLoaded {
        artist_id: String,
        tracks: Vec<Song>,
    },
    StationLoaded {
        song_name: String,
        tracks: Vec<Song>,
    },
    LyricsLoaded {
        song_id: String,
        lyrics: LyricsData,
    },
    PlaylistCreated(Playlist),
    TracksAddedToPlaylist {
        song_name: String,
        playlist_name: String,
    },
    PlaylistTrackRemoved {
        index: usize,
        track_name: String,
    },
    ArtworkLoaded {
        song_id: String,
        bytes: Vec<u8>,
    },
    Error(String),
}

/// Executes a job against the API client. Contains no state access, so it can
/// run on any spawned task.
pub async fn execute_job(job: Job, client: &AppleMusicClient) -> Effect {
    match job {
        Job::Search { query, storefront } => match client.search_catalog(&query, &storefront).await
        {
            Ok(results) => Effect::SearchDone(results),
            Err(e) => Effect::Error(format!("Search failed: {e}")),
        },
        Job::LoadView(view) | Job::RefreshView(view) => run_view_load(view, client).await,
        Job::PlaylistTracks { playlist_id } => match client.get_playlist_tracks(&playlist_id).await
        {
            Ok(tracks) => Effect::PlaylistTracksLoaded {
                playlist_id,
                tracks,
            },
            Err(e) => Effect::Error(format!("Failed to load playlist: {e}")),
        },
        Job::AlbumTracks { album_id } => match client.get_album_tracks(&album_id).await {
            Ok(tracks) => Effect::AlbumTracksLoaded { album_id, tracks },
            Err(e) => Effect::Error(format!("Failed to load album: {e}")),
        },
        Job::ArtistTracks { artist_id } => match client.get_artist_tracks(&artist_id).await {
            Ok(tracks) => Effect::ArtistTracksLoaded { artist_id, tracks },
            Err(e) => Effect::Error(format!("Failed to load artist: {e}")),
        },
        Job::Station {
            song_id,
            song_name,
            storefront,
        } => match client.create_station_for_song(&song_id, &storefront).await {
            Ok(tracks) if !tracks.is_empty() => Effect::StationLoaded { song_name, tracks },
            _ => Effect::Error("Failed to create station".to_string()),
        },
        Job::Lyrics(song) => {
            let duration = Some((song.duration_in_millis / 1000) as u32);
            match crate::api::lyrics::fetch_lyrics(
                client.http_client(),
                &song.name,
                &song.artist_name,
                duration,
                client.is_mock(),
            )
            .await
            {
                Ok(lyrics) => Effect::LyricsLoaded {
                    song_id: song.id,
                    lyrics,
                },
                Err(e) => Effect::Error(format!("Lyrics failed: {e}")),
            }
        }
        Job::CreatePlaylist(name) => match client.create_playlist(&name, None).await {
            Ok(playlist) => Effect::PlaylistCreated(playlist),
            Err(e) => Effect::Error(format!("Create playlist failed: {e}")),
        },
        Job::AddToPlaylist {
            playlist_id,
            song_id,
            song_name,
            playlist_name,
        } => match client
            .add_tracks_to_playlist(&playlist_id, &[&song_id])
            .await
        {
            Ok(()) => Effect::TracksAddedToPlaylist {
                song_name,
                playlist_name,
            },
            Err(e) => Effect::Error(format!("Add to playlist failed: {e}")),
        },
        Job::RemovePlaylistTrack {
            playlist_id,
            track_id,
            track_name,
            index,
        } => match client.delete_playlist_track(&playlist_id, &track_id).await {
            Ok(()) => Effect::PlaylistTrackRemoved { index, track_name },
            Err(e) => Effect::Error(format!("Failed to delete track: {e}")),
        },
        Job::NextSongsPage { limit, offset } => match client.get_library_songs(limit, offset).await
        {
            Ok(songs) => {
                let has_more = songs.len() >= limit;
                Effect::SongsPageLoaded {
                    offset,
                    songs,
                    has_more,
                }
            }
            Err(e) => Effect::Error(format!("Failed to load more songs: {e}")),
        },
        Job::FetchArtwork { song_id, url } => {
            match crate::ui::art::fetch_artwork_bytes(client.http_client(), &url).await {
                Ok(bytes) => Effect::ArtworkLoaded { song_id, bytes },
                Err(e) => {
                    tracing::debug!("Artwork fetch failed for {song_id}: {e}");
                    Effect::Error(format!("Artwork failed: {e}"))
                }
            }
        }
    }
}

async fn run_view_load(view: ActiveView, client: &AppleMusicClient) -> Effect {
    match view {
        ActiveView::LibrarySongs => match client.get_library_songs(100, 0).await {
            Ok(songs) => Effect::SongsLoaded(songs),
            Err(e) => Effect::Error(format!("Failed to load songs: {e}")),
        },
        ActiveView::LibraryAlbums => match client.get_library_albums(100, 0).await {
            Ok(albums) => Effect::AlbumsLoaded(albums),
            Err(e) => Effect::Error(format!("Failed to load albums: {e}")),
        },
        ActiveView::LibraryArtists => match client.get_library_artists(100, 0).await {
            Ok(artists) => Effect::ArtistsLoaded(artists),
            Err(e) => Effect::Error(format!("Failed to load artists: {e}")),
        },
        ActiveView::Playlists => match client.get_library_playlists().await {
            Ok(playlists) => Effect::PlaylistsLoaded(playlists),
            Err(e) => Effect::Error(format!("Failed to load playlists: {e}")),
        },
        ActiveView::RecentlyPlayed => match client.get_recent_tracks().await {
            Ok(recent) => Effect::RecentLoaded(recent),
            Err(e) => Effect::Error(format!("Failed to load recently played: {e}")),
        },
        _ => Effect::Error(format!("Nothing to load for view {view:?}")),
    }
}

/// Applies an effect to the shared state. Pure state mutation.
pub fn apply_effect(state: &mut AppState, effect: Effect) {
    state.is_loading = false;
    match effect {
        Effect::SearchDone(results) => {
            let count = results.songs.len();
            state.search_results = results;
            state.active_view = ActiveView::Search;
            state.sync_sidebar_to_view();
            state.selected_index = 0;
            state.set_status(format!("Found {count} songs"));
        }
        Effect::SongsLoaded(songs) => {
            state.songs = songs;
            state.cache.insert_songs(&state.songs);
            state.songs_offset = state.songs.len();
            state.songs_has_more = state.songs.len() >= 100;
            state.set_status("Library songs loaded");
        }
        Effect::AlbumsLoaded(albums) => {
            state.albums = albums;
            state.cache.insert_albums(&state.albums);
            state.set_status("Albums loaded");
        }
        Effect::ArtistsLoaded(artists) => {
            state.artists = artists;
            state.cache.insert_artists(&state.artists);
            state.set_status("Artists loaded");
        }
        Effect::PlaylistsLoaded(playlists) => {
            state.playlists = playlists;
            state.cache.insert_playlists(&state.playlists);
            state.set_status("Playlists loaded");
        }
        Effect::RecentLoaded(recent) => {
            state.recent_tracks = recent;
            state.cache.insert_recent(&state.recent_tracks);
            state.set_status("Recently played loaded");
        }
        Effect::SongsPageLoaded {
            offset,
            songs,
            has_more,
        } => {
            state.songs_loading_more = false;
            if offset == state.songs_offset {
                state.songs.extend(songs);
                state.songs_offset = state.songs.len();
                state.songs_has_more = has_more;
            }
        }
        Effect::PlaylistTracksLoaded {
            playlist_id,
            tracks,
        } => {
            if state.active_playlist.as_ref().map(|p| p.id.as_str()) == Some(playlist_id.as_str()) {
                state.playlist_tracks = tracks;
                state.set_status("Playlist tracks loaded");
            }
        }
        Effect::AlbumTracksLoaded { album_id, tracks } => {
            if state.active_playlist.as_ref().map(|p| p.id.as_str()) == Some(album_id.as_str()) {
                state.playlist_tracks = tracks;
                state.set_status("Album tracks loaded");
            }
        }
        Effect::ArtistTracksLoaded { artist_id, tracks } => {
            if state.active_playlist.as_ref().map(|p| p.id.as_str()) == Some(artist_id.as_str()) {
                state.playlist_tracks = tracks;
                state.set_status("Artist tracks loaded");
            }
        }
        Effect::StationLoaded { song_name, tracks } => {
            state.queue = tracks.clone();
            state.active_view = ActiveView::Queue;
            state.sync_sidebar_to_view();
            state.selected_index = 0;
            state.pending_playback = Some(crate::app::state::PendingPlayback::QueueStart(0));
            state.set_status(format!("Station started for '{song_name}'"));
        }
        Effect::LyricsLoaded { song_id, lyrics } => {
            state.lyrics = Some(lyrics);
            state.lyrics_song_id = Some(song_id);
            state.lyrics_loading = false;
        }
        Effect::PlaylistCreated(playlist) => {
            state.playlists.push(playlist.clone());
            state.cache.invalidate_playlists();
            state.set_status(format!("Created playlist '{}'", playlist.name));
        }
        Effect::TracksAddedToPlaylist {
            song_name,
            playlist_name,
        } => {
            state.set_status(format!("Added '{song_name}' to '{playlist_name}'"));
        }
        Effect::PlaylistTrackRemoved { index, track_name } => {
            if state.active_view == ActiveView::PlaylistDetail
                && index < state.playlist_tracks.len()
            {
                state.playlist_tracks.remove(index);
                if state.selected_index >= state.playlist_tracks.len()
                    && !state.playlist_tracks.is_empty()
                {
                    state.selected_index = state.playlist_tracks.len() - 1;
                }
            }
            state.set_status(format!("Removed '{track_name}' from playlist"));
        }
        Effect::ArtworkLoaded { song_id, bytes } => match image::load_from_memory(&bytes) {
            Ok(img) => {
                state.artwork_loading.remove(&song_id);
                state.artwork.insert(song_id, img.to_rgb8());
            }
            Err(e) => {
                state.artwork_loading.remove(&song_id);
                tracing::debug!("Artwork decode failed: {e}");
            }
        },
        Effect::Error(msg) => {
            tracing::warn!("Background job failed: {msg}");
            state.set_status(msg);
        }
    }
}

/// Test helper: runs all pending jobs synchronously and applies their effects.
pub async fn run_pending_jobs(state: &mut AppState, client: &AppleMusicClient) {
    let jobs = std::mem::take(&mut state.pending_jobs);
    for job in jobs {
        let effect = execute_job(job, client).await;
        apply_effect(state, effect);
    }
}

/// Executes a single job, ignoring errors (used for fire-and-forget writes).
pub async fn run_job_quietly(job: Job, client: &AppleMusicClient) -> Result<()> {
    let effect = execute_job(job, client).await;
    if matches!(effect, Effect::Error(_)) {
        anyhow::bail!("job failed");
    }
    Ok(())
}
