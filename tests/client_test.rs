use apple_tui::api::client::AppleMusicClient;

#[tokio::test]
async fn test_client_creation_and_mock_mode() {
    let client = AppleMusicClient::new_mock();
    assert!(client.is_mock());

    let results = client.search_catalog("the weeknd", "us").await.unwrap();
    assert!(!results.songs.is_empty());
    assert_eq!(results.songs[0].artist_name, "The Weeknd");

    let playlists = client.get_library_playlists().await.unwrap();
    assert!(!playlists.is_empty());

    let songs = client.get_library_songs(10, 0).await.unwrap();
    assert!(!songs.is_empty());

    let albums = client.get_library_albums(10, 0).await.unwrap();
    assert!(!albums.is_empty());

    let artists = client.get_library_artists(10, 0).await.unwrap();
    assert!(!artists.is_empty());

    let recent = client.get_recent_tracks().await.unwrap();
    assert!(!recent.is_empty());

    let pl_tracks = client.get_playlist_tracks("pl1").await.unwrap();
    assert!(!pl_tracks.is_empty());

    let new_pl = client
        .create_playlist("My Test Playlist", Some("Description"))
        .await
        .unwrap();
    assert_eq!(new_pl.name, "My Test Playlist");

    let add_res = client
        .add_tracks_to_playlist(&new_pl.id, &["s1", "s2"])
        .await;
    assert!(add_res.is_ok());
}
