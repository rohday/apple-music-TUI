use apple_tui::api::models::Song;
use apple_tui::playback::mpris::MprisPlayer;
use apple_tui::playback::types::{PlaybackCommand, PlaybackState, PlaybackStatus};
use mpris_server::{PlaybackStatus as MprisPlaybackStatus, PlayerInterface, RootInterface};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::test]
async fn test_mpris_root_interface() {
    let (cmd_tx, _cmd_rx) = mpsc::channel(10);
    let status = Arc::new(Mutex::new(PlaybackStatus::default()));
    let player = MprisPlayer { cmd_tx, status };

    assert_eq!(player.identity().await.unwrap(), "AppleTUI");
    assert_eq!(player.desktop_entry().await.unwrap(), "appletui");
    assert!(player.can_quit().await.unwrap());
}

#[tokio::test]
async fn test_mpris_playback_status_and_metadata() {
    let (cmd_tx, _cmd_rx) = mpsc::channel(10);
    let status = Arc::new(Mutex::new(PlaybackStatus::default()));
    let player = MprisPlayer {
        cmd_tx,
        status: status.clone(),
    };

    assert_eq!(
        player.playback_status().await.unwrap(),
        MprisPlaybackStatus::Stopped
    );

    // Update status to playing a track
    {
        let mut st = status.lock().await;
        st.state = PlaybackState::Playing;
        st.current_song = Some(Song {
            id: "mpris_test_song".to_string(),
            name: "Save Your Tears".to_string(),
            artist_name: "The Weeknd".to_string(),
            album_name: Some("After Hours".to_string()),
            duration_in_millis: 215_000,
            track_number: Some(1),
            release_date: None,
            url: None,
            catalog_id: None,
            artwork_url: None,
        });
        st.volume = 90;
    }

    assert_eq!(
        player.playback_status().await.unwrap(),
        MprisPlaybackStatus::Playing
    );

    let meta = player.metadata().await.unwrap();
    assert_eq!(meta.title(), Some("Save Your Tears"));
    assert_eq!(meta.album(), Some("After Hours"));
    assert_eq!(meta.artist(), Some(vec!["The Weeknd".to_string()]));

    let vol = player.volume().await.unwrap();
    assert!((vol - 0.90).abs() < 0.01);
}

#[tokio::test]
async fn test_mpris_commands_forwarded() {
    let (cmd_tx, mut cmd_rx) = mpsc::channel(10);
    let status = Arc::new(Mutex::new(PlaybackStatus::default()));
    let player = MprisPlayer { cmd_tx, status };

    player.play_pause().await.unwrap();
    match cmd_rx.recv().await {
        Some(PlaybackCommand::TogglePlayPause) => {}
        other => panic!("Expected TogglePlayPause, got {:?}", other),
    }

    player.next().await.unwrap();
    match cmd_rx.recv().await {
        Some(PlaybackCommand::Next) => {}
        other => panic!("Expected Next, got {:?}", other),
    }

    player.previous().await.unwrap();
    match cmd_rx.recv().await {
        Some(PlaybackCommand::Previous) => {}
        other => panic!("Expected Previous, got {:?}", other),
    }
}
