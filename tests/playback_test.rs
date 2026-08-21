use apple_tui::api::models::Song;
use apple_tui::playback::engine::PlaybackEngine;
use apple_tui::playback::types::{PlaybackCommand, PlaybackState, PlaybackStatus, RepeatMode};

#[test]
fn test_playback_status_progress() {
    let status = PlaybackStatus {
        state: PlaybackState::Playing,
        current_time_secs: 60.0,
        duration_secs: 180.0,
        volume: 75,
        ..Default::default()
    };

    assert_eq!(status.progress_ratio(), 60.0 / 180.0);
    assert_eq!(status.formatted_position(), "1:00 / 3:00");
}

#[test]
fn test_repeat_mode_cycle() {
    let mode = RepeatMode::Off;
    let next = mode.cycle();
    assert_eq!(next, RepeatMode::All);
    let next2 = next.cycle();
    assert_eq!(next2, RepeatMode::One);
    let next3 = next2.cycle();
    assert_eq!(next3, RepeatMode::Off);
}

#[tokio::test]
async fn test_playback_engine_mock_commands() {
    let engine = PlaybackEngine::new(None, true).await.unwrap();

    let song = Song {
        id: "test_1".to_string(),
        name: "Test Track".to_string(),
        artist_name: "Test Artist".to_string(),
        album_name: Some("Test Album".to_string()),
        duration_in_millis: 180000,
        track_number: Some(1),
        release_date: None,
        url: None,
    };

    engine.send_command(PlaybackCommand::PlaySong(song.clone())).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let status = engine.get_current_status().await;
    assert_eq!(status.state, PlaybackState::Playing);
    assert_eq!(status.current_song.as_ref().map(|s| &s.id), Some(&song.id));

    engine.send_command(PlaybackCommand::TogglePlayPause).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let status = engine.get_current_status().await;
    assert_eq!(status.state, PlaybackState::Paused);

    engine.send_command(PlaybackCommand::SetVolume(90)).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let status = engine.get_current_status().await;
    assert_eq!(status.volume, 90);
}
