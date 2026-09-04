use crate::api::client::AppleMusicClient;
use crate::app::job::Job;
use crate::app::state::{ActiveView, AppState, FocusedPanel, ModalState};
use crate::playback::engine::PlaybackEngine;
use crate::playback::types::PlaybackCommand;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles a key press by mutating state and enqueuing background jobs.
/// This function never performs network I/O; jobs are executed by the main
/// loop (or by `run_pending_jobs` in tests), so the UI thread never blocks.
pub async fn handle_key_event(
    key: KeyEvent,
    state: &mut AppState,
    _client: &AppleMusicClient,
    playback: &PlaybackEngine,
) -> Result<()> {
    // 0. Handle in-view filter input mode
    if state.is_filtering {
        match key.code {
            KeyCode::Esc => {
                state.clear_filter();
                state.set_status("Filter cleared");
                return Ok(());
            }
            KeyCode::Enter => {
                state.is_filtering = false;
                state.set_status(format!("Filter applied: '{}'", state.filter_query));
                return Ok(());
            }
            KeyCode::Backspace => {
                state.filter_query.pop();
                state.selected_index = 0;
                return Ok(());
            }
            KeyCode::Up => {
                state.move_selection_up();
                return Ok(());
            }
            KeyCode::Down => {
                state.move_selection_down();
                maybe_fetch_next_songs_page(state);
                return Ok(());
            }
            KeyCode::Char(c) => {
                state.filter_query.push(c);
                state.selected_index = 0;
                return Ok(());
            }
            _ => return Ok(()),
        }
    }

    // 1. Handle Active Modals First
    match &mut state.modal {
        ModalState::Search => match key.code {
            KeyCode::Enter => {
                let query = state.text_input_buffer.trim().to_string();
                state.close_modal();
                if !query.is_empty() {
                    state.search_query = query.clone();
                    state.active_view = ActiveView::Search;
                    state.sidebar_index = 0;
                    state.selected_index = 0;
                    state.focused_panel = FocusedPanel::MainContent;
                    state.set_status(format!("Searching for '{}'...", query));
                    state.enqueue_job(Job::Search {
                        query,
                        storefront: state.storefront.clone(),
                    });
                }
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
                state.close_modal();
                if !name.is_empty() {
                    state.set_status(format!("Creating playlist '{}'...", name));
                    state.enqueue_job(Job::CreatePlaylist(name));
                }
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
        ModalState::AddToPlaylist { song } => {
            let song = song.clone();
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.add_to_playlist_index > 0 {
                        state.add_to_playlist_index -= 1;
                    }
                    return Ok(());
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !state.playlists.is_empty()
                        && state.add_to_playlist_index + 1 < state.playlists.len()
                    {
                        state.add_to_playlist_index += 1;
                    }
                    return Ok(());
                }
                KeyCode::Enter => {
                    if let Some(pl) = state.playlists.get(state.add_to_playlist_index) {
                        let job = Job::AddToPlaylist {
                            playlist_id: pl.id.clone(),
                            song_id: song.id.clone(),
                            song_name: song.name.clone(),
                            playlist_name: pl.name.clone(),
                        };
                        state.enqueue_job(job);
                    }
                    state.close_modal();
                    return Ok(());
                }
                KeyCode::Esc => {
                    state.close_modal();
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }
        ModalState::AuthPrompt => {
            if key.code == KeyCode::Char('l') || key.code == KeyCode::Char('L') {
                state.pending_login = true;
                state.close_modal();
                return Ok(());
            }
            if key.code == KeyCode::Esc
                || key.code == KeyCode::Char('q')
                || key.code == KeyCode::Enter
            {
                state.close_modal();
                return Ok(());
            }
        }
        ModalState::Help | ModalState::Notification(_) => {
            if key.code == KeyCode::Esc
                || key.code == KeyCode::Char('q')
                || key.code == KeyCode::Enter
            {
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
        KeyCode::Char('L') => {
            state.pending_login = true;
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
            let needs_queue_start = (state.playback.current_song.is_none()
                || state.playback.state == crate::playback::types::PlaybackState::Stopped)
                && state.get_selected_song().is_some();
            if needs_queue_start {
                start_playback_from_selection(state, playback).await?;
                return Ok(());
            }
            playback
                .send_command(PlaybackCommand::TogglePlayPause)
                .await?;
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
            playback
                .send_command(PlaybackCommand::SeekRelative(-10.0))
                .await?;
            return Ok(());
        }
        KeyCode::Char(']') => {
            playback
                .send_command(PlaybackCommand::SeekRelative(10.0))
                .await?;
            return Ok(());
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let new_vol = (state.playback.volume + 5).min(100);
            state.playback.volume = new_vol;
            playback
                .send_command(PlaybackCommand::SetVolume(new_vol))
                .await?;
            return Ok(());
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            let new_vol = state.playback.volume.saturating_sub(5);
            state.playback.volume = new_vol;
            playback
                .send_command(PlaybackCommand::SetVolume(new_vol))
                .await?;
            return Ok(());
        }
        KeyCode::F(5) => {
            state.cache.invalidate_all();
            refresh_view(state);
            return Ok(());
        }
        KeyCode::Char('s') => {
            playback
                .send_command(PlaybackCommand::ToggleShuffle)
                .await?;
            return Ok(());
        }
        KeyCode::Char('r') => {
            playback.send_command(PlaybackCommand::CycleRepeat).await?;
            return Ok(());
        }
        KeyCode::Char('t') => {
            let new_theme = state.cycle_theme();
            state.set_status(format!("Theme: {}", new_theme.display_name()));
            let mut cfg = crate::config::Config::load();
            cfg.theme = new_theme;
            let _ = cfg.save();
            return Ok(());
        }
        KeyCode::Char('v') => {
            state.show_visualizer = !state.show_visualizer;
            let status = if state.show_visualizer {
                "Visualizer: On"
            } else {
                "Visualizer: Off"
            };
            state.set_status(status);
            return Ok(());
        }
        KeyCode::Esc if state.show_visualizer => {
            state.show_visualizer = false;
            state.set_status("Visualizer: Off");
            return Ok(());
        }
        KeyCode::Char('f') => {
            state.is_filtering = true;
            state.set_status("Filter: Type query, Enter to apply, Esc to clear");
            return Ok(());
        }
        KeyCode::Esc if !state.filter_query.is_empty() => {
            state.clear_filter();
            state.set_status("Filter cleared");
            return Ok(());
        }
        KeyCode::Char('R') => {
            if let Some(song) = state.get_selected_song() {
                state.set_status(format!("Creating Station for '{}'...", song.name));
                state.enqueue_job(Job::Station {
                    song_id: song.id.clone(),
                    song_name: song.name.clone(),
                    storefront: state.storefront.clone(),
                });
            } else {
                state.set_status("Select a song to start a station");
            }
            return Ok(());
        }
        KeyCode::Esc if state.show_lyrics => {
            state.show_lyrics = false;
            state.set_status("Lyrics closed");
            return Ok(());
        }
        KeyCode::Char('y') => {
            state.toggle_lyrics();
            if state.show_lyrics {
                state.set_status("Lyrics panel opened");
                enqueue_lyrics_for_current_song(state);
            } else {
                state.set_status("Lyrics closed");
            }
            return Ok(());
        }
        KeyCode::Char('o') => {
            if state.playback.current_song.is_some() {
                state.show_now_playing = !state.show_now_playing;
            } else {
                state.set_status("Nothing playing");
            }
            return Ok(());
        }
        KeyCode::Esc if state.show_now_playing => {
            state.show_now_playing = false;
            return Ok(());
        }
        KeyCode::Char('A') => {
            if let Some(song) = state.get_selected_song() {
                state.queue.push(song.clone());
                playback
                    .send_command(PlaybackCommand::Enqueue(vec![song.clone()]))
                    .await?;
                state.set_status(format!("Added '{}' to queue", song.name));
            }
            return Ok(());
        }
        _ => {}
    }

    // 3. Navigation Controls
    match state.focused_panel {
        FocusedPanel::Sidebar => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_sidebar_up();
                enqueue_view_load(state, state.active_view);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_sidebar_down();
                enqueue_view_load(state, state.active_view);
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                if state.active_view == ActiveView::Search && state.search_results.songs.is_empty()
                {
                    state.open_search();
                    return Ok(());
                }
                state.focused_panel = FocusedPanel::MainContent;
                enqueue_view_load(state, state.active_view);
            }
            _ => {}
        },
        FocusedPanel::MainContent => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_selection_down();
                maybe_fetch_next_songs_page(state);
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                if state.active_view == ActiveView::PlaylistDetail {
                    let prev_view = ActiveView::all_sidebar_views()[state
                        .sidebar_index
                        .min(ActiveView::all_sidebar_views().len() - 1)];
                    state.active_view = prev_view;
                    state.active_playlist = None;
                } else {
                    state.focused_panel = FocusedPanel::Sidebar;
                }
            }
            KeyCode::Enter => match state.active_view {
                ActiveView::Search if state.search_results.songs.is_empty() => {
                    state.open_search();
                    return Ok(());
                }
                ActiveView::LibrarySongs
                | ActiveView::PlaylistDetail
                | ActiveView::RecentlyPlayed
                | ActiveView::Search
                | ActiveView::Queue => {
                    start_playback_from_selection(state, playback).await?;
                }
                ActiveView::Playlists => {
                    if let Some(pl) = state.playlists.get(state.selected_index).cloned() {
                        state.set_status(format!("Loading playlist '{}'...", pl.name));
                        let pl_id = pl.id.clone();
                        open_detail_view(state, pl);
                        state.enqueue_job(Job::PlaylistTracks { playlist_id: pl_id });
                    }
                }
                ActiveView::LibraryAlbums => {
                    if let Some(alb) = state.albums.get(state.selected_index).cloned() {
                        state.set_status(format!("Loading album '{}'...", alb.name));
                        let pl = crate::api::models::Playlist {
                            id: alb.id.clone(),
                            name: alb.name.clone(),
                            description: Some(alb.artist_name.clone()),
                            is_public: false,
                            track_count: alb.track_count,
                        };
                        open_detail_view(state, pl);
                        state.enqueue_job(Job::AlbumTracks { album_id: alb.id });
                    }
                }
                ActiveView::LibraryArtists => {
                    if let Some(art) = state.artists.get(state.selected_index).cloned() {
                        state.set_status(format!("Loading artist '{}'...", art.name));
                        let pl = crate::api::models::Playlist {
                            id: art.id.clone(),
                            name: art.name.clone(),
                            description: Some("Artist Tracks".to_string()),
                            is_public: false,
                            track_count: None,
                        };
                        open_detail_view(state, pl);
                        state.enqueue_job(Job::ArtistTracks { artist_id: art.id });
                    }
                }
            },
            KeyCode::Char('d') | KeyCode::Delete => match state.active_view {
                ActiveView::PlaylistDetail => {
                    if let Some(track) = state.playlist_tracks.get(state.selected_index).cloned() {
                        if let Some(pl_id) = state.active_playlist.as_ref().map(|p| p.id.clone()) {
                            state.set_status(format!("Removing '{}'...", track.name));
                            state.enqueue_job(Job::RemovePlaylistTrack {
                                playlist_id: pl_id,
                                track_id: track.id.clone(),
                                track_name: track.name.clone(),
                                index: state.selected_index,
                            });
                        }
                    }
                }
                ActiveView::Queue => {
                    let idx = state.selected_index;
                    if let Some(song) = state.remove_from_queue(idx) {
                        playback
                            .send_command(PlaybackCommand::RemoveFromQueue(idx))
                            .await?;
                        state.set_status(format!("Removed '{}' from queue", song.name));
                    }
                }
                _ => {}
            },
            KeyCode::Char('<') => {
                if state.active_view == ActiveView::Queue && state.selected_index > 0 {
                    let (idx, new_idx) = (state.selected_index, state.selected_index - 1);
                    if state.move_queue_item(idx, true) {
                        playback
                            .send_command(PlaybackCommand::MoveQueueItem(idx, new_idx))
                            .await?;
                        state.set_status("Moved up in queue");
                    }
                }
            }
            KeyCode::Char('>') => {
                if state.active_view == ActiveView::Queue {
                    let (idx, new_idx) = (state.selected_index, state.selected_index + 1);
                    if state.move_queue_item(idx, false) {
                        playback
                            .send_command(PlaybackCommand::MoveQueueItem(idx, new_idx))
                            .await?;
                        state.set_status("Moved down in queue");
                    }
                }
            }
            KeyCode::Char('a') => {
                if let Some(song) = state.get_selected_song() {
                    state.open_add_to_playlist(song);
                }
            }
            KeyCode::Char('c') if state.active_view == ActiveView::Playlists => {
                state.open_create_playlist();
            }
            _ => {}
        },
    }

    Ok(())
}

/// Switches to the detail (PlaylistDetail) view optimistically; tracks arrive
/// later via an effect matched on the active playlist id.
fn open_detail_view(state: &mut AppState, pl: crate::api::models::Playlist) {
    state.active_playlist = Some(pl);
    state.playlist_tracks = Vec::new();
    state.active_view = ActiveView::PlaylistDetail;
    state.selected_index = 0;
}

/// Starts playback of the current view's list at the selected index.
async fn start_playback_from_selection(
    state: &mut AppState,
    playback: &PlaybackEngine,
) -> Result<()> {
    if state.get_selected_song().is_none() {
        return Ok(());
    }
    let list = match state.active_view {
        ActiveView::LibrarySongs => state.songs.clone(),
        ActiveView::PlaylistDetail => state.playlist_tracks.clone(),
        ActiveView::RecentlyPlayed => state.recent_tracks.clone(),
        ActiveView::Search => state.search_results.songs.clone(),
        ActiveView::Queue => state.queue.clone(),
        _ => Vec::new(),
    };
    let index = state.selected_index.min(list.len().saturating_sub(1));
    playback
        .send_command(PlaybackCommand::SetQueueAndPlay(list, index))
        .await?;
    Ok(())
}

/// Enqueues the next page of library songs when the selection approaches the
/// end of the currently loaded list.
fn maybe_fetch_next_songs_page(state: &mut AppState) {
    if state.should_fetch_next_songs_page() {
        state.songs_loading_more = true;
        state.enqueue_job(Job::NextSongsPage {
            limit: 100,
            offset: state.songs_offset,
        });
    }
}

/// Requests a view's data: served instantly from cache when fresh, otherwise
/// enqueued as a background job.
fn enqueue_view_load(state: &mut AppState, view: ActiveView) {
    if !state.is_authenticated {
        return;
    }
    match view {
        ActiveView::LibrarySongs if state.songs.is_empty() => {
            if let Some(cached) = state.cache.get_songs() {
                state.songs = cached.to_vec();
                state.songs_offset = state.songs.len();
                state.songs_has_more = state.songs.len() >= 100;
            } else {
                state.enqueue_job(Job::LoadView(view));
            }
        }
        ActiveView::LibraryAlbums if state.albums.is_empty() => {
            if let Some(cached) = state.cache.get_albums() {
                state.albums = cached.to_vec();
            } else {
                state.enqueue_job(Job::LoadView(view));
            }
        }
        ActiveView::LibraryArtists if state.artists.is_empty() => {
            if let Some(cached) = state.cache.get_artists() {
                state.artists = cached.to_vec();
            } else {
                state.enqueue_job(Job::LoadView(view));
            }
        }
        ActiveView::Playlists if state.playlists.is_empty() => {
            if let Some(cached) = state.cache.get_playlists() {
                state.playlists = cached.to_vec();
            } else {
                state.enqueue_job(Job::LoadView(view));
            }
        }
        ActiveView::RecentlyPlayed if state.recent_tracks.is_empty() => {
            if let Some(cached) = state.cache.get_recent() {
                state.recent_tracks = cached.to_vec();
            } else {
                state.enqueue_job(Job::LoadView(view));
            }
        }
        _ => {}
    }
}

/// F5 handler: re-fetches the active view bypassing the cache.
fn refresh_view(state: &mut AppState) {
    let view = state.active_view;
    state.set_status("Refreshing data from Apple Music...");
    match view {
        ActiveView::LibrarySongs => {
            state.songs.clear();
            state.songs_offset = 0;
            state.songs_has_more = false;
            state.enqueue_job(Job::LoadView(view));
        }
        ActiveView::LibraryAlbums => {
            state.albums.clear();
            state.enqueue_job(Job::LoadView(view));
        }
        ActiveView::LibraryArtists => {
            state.artists.clear();
            state.enqueue_job(Job::LoadView(view));
        }
        ActiveView::Playlists => {
            state.playlists.clear();
            state.enqueue_job(Job::LoadView(view));
        }
        ActiveView::RecentlyPlayed => {
            state.recent_tracks.clear();
            state.enqueue_job(Job::LoadView(view));
        }
        _ => {
            state.set_status("Nothing to refresh in this view");
        }
    }
}

/// Requests lyrics for the currently playing song, unless already loaded or
/// loading. Replaces the old blocking `load_lyrics_for_current_song`.
pub fn enqueue_lyrics_for_current_song(state: &mut AppState) {
    if let Some(song) = state.playback.current_song.clone() {
        if state.lyrics_song_id.as_deref() == Some(&song.id) || state.lyrics_loading {
            return;
        }
        state.lyrics_loading = true;
        state.lyrics = None;
        state.enqueue_job(Job::Lyrics(song));
    } else {
        state.lyrics = None;
        state.lyrics_song_id = None;
    }
}

/// Requests cover art for the currently playing song once per session.
pub fn enqueue_artwork_for_current_song(state: &mut AppState) {
    if let Some(song) = state.playback.current_song.clone() {
        if state.artwork.contains_key(&song.id) || state.artwork_loading.contains(&song.id) {
            return;
        }
        if let Some(url) = song.resolved_artwork_url() {
            state.artwork_loading.insert(song.id.clone());
            state.enqueue_job(Job::FetchArtwork {
                song_id: song.id,
                url,
            });
        }
    }
}
