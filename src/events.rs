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
                if !state.playlists.is_empty()
                    && state.add_to_playlist_index + 1 < state.playlists.len()
                {
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
            if state.playback.current_song.is_none() || state.playback.state == crate::playback::types::PlaybackState::Stopped {
                if let Some(song) = state.get_selected_song() {
                    let list = match state.active_view {
                        ActiveView::LibrarySongs => state.songs.clone(),
                        ActiveView::PlaylistDetail => state.playlist_tracks.clone(),
                        ActiveView::RecentlyPlayed => state.recent_tracks.clone(),
                        ActiveView::Search => state.search_results.songs.clone(),
                        ActiveView::Queue => state.queue.clone(),
                        _ => vec![song.clone()],
                    };
                    playback
                        .send_command(PlaybackCommand::SetQueueAndPlay(
                            list,
                            state.selected_index,
                        ))
                        .await?;
                    return Ok(());
                }
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
            state.set_status("Refreshing data from Apple Music...");
            match state.active_view {
                ActiveView::LibrarySongs => {
                    if let Ok(songs) = client.get_library_songs(100, 0).await {
                        state.songs = songs;
                        state.set_status("Refreshed Library Songs");
                    }
                }
                ActiveView::LibraryAlbums => {
                    if let Ok(albums) = client.get_library_albums(100, 0).await {
                        state.albums = albums;
                        state.set_status("Refreshed Albums");
                    }
                }
                ActiveView::LibraryArtists => {
                    if let Ok(artists) = client.get_library_artists(100, 0).await {
                        state.artists = artists;
                        state.set_status("Refreshed Artists");
                    }
                }
                ActiveView::Playlists => {
                    if let Ok(playlists) = client.get_library_playlists().await {
                        state.playlists = playlists;
                        state.set_status("Refreshed Playlists");
                    }
                }
                ActiveView::RecentlyPlayed => {
                    if let Ok(recent) = client.get_recent_tracks().await {
                        state.recent_tracks = recent;
                        state.set_status("Refreshed Recently Played");
                    }
                }
                _ => {}
            }
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
                "Visualizer: Enabled (Press 'v' or 'Esc' to exit)"
            } else {
                "Visualizer: Disabled"
            };
            state.set_status(status);
            return Ok(());
        }
        KeyCode::Esc if state.show_visualizer => {
            state.show_visualizer = false;
            state.set_status("Visualizer: Disabled");
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
                match client.create_station_for_song(&song.id, &state.storefront).await {
                    Ok(station_tracks) if !station_tracks.is_empty() => {
                        state.queue = station_tracks.clone();
                        state.active_view = ActiveView::Queue;
                        state.selected_index = 0;
                        playback
                            .send_command(PlaybackCommand::SetQueueAndPlay(station_tracks, 0))
                            .await?;
                        state.set_status(format!("📻 Playing Station for '{}'", song.name));
                    }
                    _ => {
                        state.set_status("Failed to create station");
                    }
                }
            } else {
                state.set_status("Select a song to start a station");
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
                    let prev_view = ActiveView::all_sidebar_views()[state.sidebar_index.min(ActiveView::all_sidebar_views().len() - 1)];
                    state.active_view = prev_view;
                    state.active_playlist = None;
                } else {
                    state.focused_panel = FocusedPanel::Sidebar;
                }
            }
            KeyCode::Enter => match state.active_view {
                ActiveView::LibrarySongs
                | ActiveView::PlaylistDetail
                | ActiveView::RecentlyPlayed
                | ActiveView::Search
                | ActiveView::Queue => {
                    if let Some(song) = state.get_selected_song() {
                        let list = match state.active_view {
                            ActiveView::LibrarySongs => state.songs.clone(),
                            ActiveView::PlaylistDetail => state.playlist_tracks.clone(),
                            ActiveView::RecentlyPlayed => state.recent_tracks.clone(),
                            ActiveView::Search => state.search_results.songs.clone(),
                            ActiveView::Queue => state.queue.clone(),
                            _ => vec![song.clone()],
                        };
                        playback
                            .send_command(PlaybackCommand::SetQueueAndPlay(
                                list,
                                state.selected_index,
                            ))
                            .await?;
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
                ActiveView::LibraryAlbums => {
                    if let Some(alb) = state.albums.get(state.selected_index).cloned() {
                        state.set_status(format!("Loading album '{}'...", alb.name));
                        if let Ok(tracks) = client.get_album_tracks(&alb.id).await {
                            state.playlist_tracks = tracks;
                            state.active_playlist = Some(crate::api::models::Playlist {
                                id: alb.id.clone(),
                                name: alb.name.clone(),
                                description: Some(alb.artist_name.clone()),
                                is_public: false,
                                track_count: alb.track_count,
                            });
                            state.active_view = ActiveView::PlaylistDetail;
                            state.selected_index = 0;
                        }
                    }
                }
                ActiveView::LibraryArtists => {
                    if let Some(art) = state.artists.get(state.selected_index).cloned() {
                        state.set_status(format!("Loading artist '{}'...", art.name));
                        if let Ok(tracks) = client.get_artist_tracks(&art.id).await {
                            let track_count = Some(tracks.len() as u32);
                            state.playlist_tracks = tracks;
                            state.active_playlist = Some(crate::api::models::Playlist {
                                id: art.id.clone(),
                                name: art.name.clone(),
                                description: Some("Artist Tracks".to_string()),
                                is_public: false,
                                track_count,
                            });
                            state.active_view = ActiveView::PlaylistDetail;
                            state.selected_index = 0;
                        }
                    }
                }
            },
            KeyCode::Char('d') | KeyCode::Delete if state.active_view == ActiveView::PlaylistDetail => {
                if let Some(track) = state.playlist_tracks.get(state.selected_index).cloned() {
                    if let Some(pl_id) = state.active_playlist.as_ref().map(|p| p.id.clone()) {
                        state.set_status(format!("Removing '{}'...", track.name));
                        if let Err(e) = client.delete_playlist_track(&pl_id, &track.id).await {
                            state.set_status(format!("Failed to delete track: {e}"));
                        } else {
                            state.playlist_tracks.remove(state.selected_index);
                            if state.selected_index >= state.playlist_tracks.len() && !state.playlist_tracks.is_empty() {
                                state.selected_index = state.playlist_tracks.len() - 1;
                            }
                            state.set_status(format!("Removed '{}' from playlist", track.name));
                        }
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

async fn load_view_data(state: &mut AppState, client: &AppleMusicClient) -> Result<()> {
    if !state.is_authenticated && !client.is_mock() {
        return Ok(());
    }

    match state.active_view {
        ActiveView::LibrarySongs if state.songs.is_empty() => {
            if let Ok(songs) = client.get_library_songs(100, 0).await {
                state.songs = songs;
            }
        }
        ActiveView::LibraryAlbums if state.albums.is_empty() => {
            if let Ok(albums) = client.get_library_albums(100, 0).await {
                state.albums = albums;
            }
        }
        ActiveView::LibraryArtists if state.artists.is_empty() => {
            if let Ok(artists) = client.get_library_artists(100, 0).await {
                state.artists = artists;
            }
        }
        ActiveView::Playlists if state.playlists.is_empty() => {
            if let Ok(playlists) = client.get_library_playlists().await {
                state.playlists = playlists;
            }
        }
        ActiveView::RecentlyPlayed if state.recent_tracks.is_empty() => {
            if let Ok(recent) = client.get_recent_tracks().await {
                state.recent_tracks = recent;
            }
        }
        _ => {}
    }
    Ok(())
}
