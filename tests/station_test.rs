use apple_tui::api::client::AppleMusicClient;

#[tokio::test]
async fn test_create_station_for_song_mock() {
    let client = AppleMusicClient::new_mock();
    let station_tracks = client.create_station_for_song("1001", "us").await.unwrap();

    assert!(!station_tracks.is_empty());
    assert!(station_tracks.len() >= 5);
    // Tracks should have valid names and IDs
    for track in &station_tracks {
        assert!(!track.id.is_empty());
        assert!(!track.name.is_empty());
    }
}
