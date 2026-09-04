use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::job::{apply_effect, run_pending_jobs, Effect, Job};
use apple_tui::app::state::{ActiveView, AppState};
use apple_tui::playback::types::PlaybackStatus;

fn mock_song(id: &str) -> apple_tui::api::models::Song {
    apple_tui::api::models::Song {
        id: id.to_string(),
        name: format!("Song {id}"),
        artist_name: "Artist".to_string(),
        album_name: None,
        duration_in_millis: 200_000,
        track_number: None,
        release_date: None,
        url: None,
        catalog_id: None,
        artwork_url: None,
    }
}

#[test]
fn queue_remove_and_move() {
    let mut state = AppState::new();
    state.queue = vec![mock_song("s1"), mock_song("s2"), mock_song("s3")];
    state.selected_index = 1;

    // Move down: s2 swaps with s3
    assert!(state.move_queue_item(1, false));
    assert_eq!(state.queue[2].id, "s2");
    assert_eq!(state.selected_index, 2);

    // Move up twice: s2 goes to front
    assert!(state.move_queue_item(2, true));
    assert!(state.move_queue_item(1, true));
    assert_eq!(state.queue[0].id, "s2");
    assert_eq!(state.selected_index, 0);

    // Can't move above top
    assert!(!state.move_queue_item(0, true));

    // Remove at selection adjusts selection
    state.selected_index = 0;
    let removed = state.remove_from_queue(0).unwrap();
    assert_eq!(removed.id, "s2");
    assert_eq!(state.queue.len(), 2);
}

#[test]
fn station_effect_starts_queue_playback() {
    let mut state = AppState::new();
    let tracks = vec![mock_song("a"), mock_song("b")];
    apply_effect(
        &mut state,
        Effect::StationLoaded {
            song_name: "Test".to_string(),
            tracks,
        },
    );
    assert_eq!(state.active_view, ActiveView::Queue);
    assert_eq!(state.queue.len(), 2);
    assert_eq!(
        state.pending_playback,
        Some(apple_tui::app::state::PendingPlayback::QueueStart(0))
    );
}

#[test]
fn stale_playlist_tracks_ignored() {
    let mut state = AppState::new();
    state.active_playlist = Some(apple_tui::api::models::Playlist {
        id: "pl_current".to_string(),
        name: "Current".to_string(),
        description: None,
        is_public: false,
        track_count: None,
    });
    apply_effect(
        &mut state,
        Effect::PlaylistTracksLoaded {
            playlist_id: "pl_old".to_string(),
            tracks: vec![mock_song("x")],
        },
    );
    assert!(state.playlist_tracks.is_empty());
}

#[tokio::test]
async fn playlist_detail_drilldown_via_jobs() {
    let client = AppleMusicClient::new_mock();
    let mut state = AppState::new();
    state.is_authenticated = true;
    state.playlists = client.get_library_playlists().await.unwrap();

    state.active_view = ActiveView::Playlists;
    state.selected_index = 0;
    let pl = state.playlists[0].clone();
    state.active_playlist = Some(pl.clone());
    state.active_view = ActiveView::PlaylistDetail;
    state.enqueue_job(Job::PlaylistTracks { playlist_id: pl.id });

    run_pending_jobs(&mut state, &client).await;
    assert!(!state.playlist_tracks.is_empty());
    assert!(!state.is_loading);
}

#[test]
fn songs_page_effect_appends_only_in_order() {
    let mut state = AppState::new();
    state.songs = vec![mock_song("s1")];
    state.songs_offset = 1;

    // Out-of-order page arrives: ignored
    apply_effect(
        &mut state,
        Effect::SongsPageLoaded {
            offset: 5,
            songs: vec![mock_song("late")],
            has_more: false,
        },
    );
    assert_eq!(state.songs.len(), 1);

    // In-order page appends and advances the offset
    apply_effect(
        &mut state,
        Effect::SongsPageLoaded {
            offset: 1,
            songs: vec![mock_song("s2")],
            has_more: true,
        },
    );
    assert_eq!(state.songs.len(), 2);
    assert_eq!(state.songs_offset, 2);
    assert!(state.songs_has_more);
    assert!(!state.songs_loading_more);
}

#[test]
fn artwork_effect_decodes_into_cache() {
    // 1x1 png, red pixel
    let png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut state = AppState::new();
    state.artwork_loading.insert("song1".to_string());
    apply_effect(
        &mut state,
        Effect::ArtworkLoaded {
            song_id: "song1".to_string(),
            bytes: png,
        },
    );
    assert!(state.artwork.contains_key("song1"));
    assert!(!state.artwork_loading.contains("song1"));
}

#[test]
fn playback_status_defaults_allow_progress() {
    let status = PlaybackStatus::default();
    assert_eq!(status.progress_ratio(), 0.0);
    assert_eq!(status.formatted_position(), "0:00 / 0:00");
}
