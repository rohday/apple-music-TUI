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
        ModalState::Help | ModalState::Notification(_) | ModalState::AuthPrompt => {
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
        KeyCode::Char('R') | KeyCode::F(5) => {
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
                _ => {}
            },
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
