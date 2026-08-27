use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::state::{ActiveView, AppState, FocusedPanel, ModalState};
use apple_tui::events::handle_key_event;
use apple_tui::playback::engine::PlaybackEngine;
use apple_tui::playback::types::{PlaybackCommand, PlaybackState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[tokio::test]
async fn test_end_to_end_mock_pipeline() {
    let client = AppleMusicClient::new_mock();
    let playback = PlaybackEngine::new(None, true).await.unwrap();

    let mut state = AppState::new();
    state.songs = client.get_library_songs(10, 0).await.unwrap();
    assert!(!state.songs.is_empty());

    let song_to_play = state.songs[0].clone();
    playback
        .send_command(PlaybackCommand::PlaySong(song_to_play.clone()))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let status = playback.get_current_status().await;
    assert_eq!(
        status.current_song.as_ref().map(|s| &s.id),
        Some(&song_to_play.id)
    );
}

#[tokio::test]
async fn test_event_handler_navigation_and_shortcuts() {
    let client = AppleMusicClient::new_mock();
    let playback = PlaybackEngine::new(None, true).await.unwrap();
    let mut state = AppState::new();

    // 1. Tab key to switch focus
    assert_eq!(state.focused_panel, FocusedPanel::MainContent);
    handle_key_event(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        &mut state,
        &client,
        &playback,
    )
    .await
    .unwrap();
    assert_eq!(state.focused_panel, FocusedPanel::Sidebar);

    // 2. '/' key to open search
    handle_key_event(
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
        &mut state,
        &client,
        &playback,
    )
    .await
    .unwrap();
    assert_eq!(state.modal, ModalState::Search);

    // 3. Search typing and enter
    handle_key_event(
        KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
        &mut state,
        &client,
        &playback,
    )
    .await
    .unwrap();
    handle_key_event(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        &mut state,
        &client,
        &playback,
    )
    .await
    .unwrap();
    assert_eq!(state.active_view, ActiveView::Search);
    assert!(!state.search_results.songs.is_empty());

    // 4. Enter on search song to play
    state.selected_index = 0;
    handle_key_event(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        &mut state,
        &client,
        &playback,
    )
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let status = playback.get_current_status().await;
    assert_eq!(status.state, PlaybackState::Playing);
}
