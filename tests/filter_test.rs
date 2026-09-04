use apple_tui::api::models::Song;
use apple_tui::app::state::{ActiveView, AppState};

#[test]
fn test_filter_matches_songs_and_resets() {
    let mut state = AppState::new();
    state.active_view = ActiveView::LibrarySongs;
    state.songs = vec![
        Song {
            id: "1".to_string(),
            name: "Blinding Lights".to_string(),
            artist_name: "The Weeknd".to_string(),
            album_name: Some("After Hours".to_string()),
            duration_in_millis: 200_000,
            track_number: Some(1),
            release_date: None,
            url: None,
            catalog_id: None,
            artwork_url: None,
        },
        Song {
            id: "2".to_string(),
            name: "Starboy".to_string(),
            artist_name: "The Weeknd".to_string(),
            album_name: Some("Starboy".to_string()),
            duration_in_millis: 230_000,
            track_number: Some(1),
            release_date: None,
            url: None,
            catalog_id: None,
            artwork_url: None,
        },
        Song {
            id: "3".to_string(),
            name: "Save Your Tears".to_string(),
            artist_name: "The Weeknd".to_string(),
            album_name: Some("After Hours".to_string()),
            duration_in_millis: 215_000,
            track_number: Some(2),
            release_date: None,
            url: None,
            catalog_id: None,
            artwork_url: None,
        },
    ];

    state.filter_query = "star".to_string();
    let filtered = state.filtered_songs();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Starboy");

    state.filter_query = "hours".to_string(); // Matches album name
    let filtered_album = state.filtered_songs();
    assert_eq!(filtered_album.len(), 2);

    state.clear_filter();
    assert_eq!(state.filter_query, "");
    assert_eq!(state.filtered_songs().len(), 3);
}
