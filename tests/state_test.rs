use apple_tui::api::models::Song;
use apple_tui::app::state::{ActiveView, AppState, FocusedPanel, ModalState};

#[test]
fn test_state_navigation_and_clamping() {
    let mut state = AppState::new();
    assert_eq!(state.active_view, ActiveView::LibrarySongs);
    assert_eq!(state.focused_panel, FocusedPanel::MainContent);

    state.songs = vec![
        Song {
            id: "1".to_string(),
            name: "Song A".to_string(),
            artist_name: "Artist A".to_string(),
            album_name: None,
            duration_in_millis: 180000,
            track_number: None,
            release_date: None,
            url: None,
            catalog_id: None,
        },
        Song {
            id: "2".to_string(),
            name: "Song B".to_string(),
            artist_name: "Artist B".to_string(),
            album_name: None,
            duration_in_millis: 210000,
            track_number: None,
            release_date: None,
            url: None,
            catalog_id: None,
        },
    ];

    state.focused_panel = FocusedPanel::MainContent;
    assert_eq!(state.selected_index, 0);

    state.move_selection_down();
    assert_eq!(state.selected_index, 1);

    // Clamp at end
    state.move_selection_down();
    assert_eq!(state.selected_index, 1);

    state.move_selection_up();
    assert_eq!(state.selected_index, 0);

    // Clamp at start
    state.move_selection_up();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_modal_toggling() {
    let mut state = AppState::new();
    assert_eq!(state.modal, ModalState::None);

    state.open_search();
    assert_eq!(state.modal, ModalState::Search);

    state.close_modal();
    assert_eq!(state.modal, ModalState::None);
}
