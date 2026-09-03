use apple_tui::api::models::{Playlist, Song};
use apple_tui::app::state::{ActiveView, AppState, FocusedPanel, ModalState};
use apple_tui::playback::types::{PlaybackState, PlaybackStatus, RepeatMode};
use apple_tui::ui::draw;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_ui_draw_default_state() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new();
    state.status_message = Some("Testing UI rendering".to_string());

    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 120);
    assert_eq!(buffer.area.height, 40);
}

#[test]
fn test_ui_draw_with_songs_and_playback() {
    let backend = TestBackend::new(140, 45);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new();
    let song = Song {
        id: "s1".to_string(),
        name: "Blinding Lights".to_string(),
        artist_name: "The Weeknd".to_string(),
        album_name: Some("After Hours".to_string()),
        duration_in_millis: 200040,
        track_number: Some(9),
        release_date: Some("2019-11-29".to_string()),
        url: None,
        catalog_id: None,
    };
    state.songs = vec![song.clone()];
    state.focused_panel = FocusedPanel::MainContent;
    state.active_view = ActiveView::LibrarySongs;
    state.playback = PlaybackStatus {
        state: PlaybackState::Playing,
        current_time_secs: 45.0,
        duration_secs: 200.0,
        current_song: Some(song),
        volume: 85,
        shuffle: true,
        repeat: RepeatMode::All,
    };

    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 140);
}

#[test]
fn test_ui_draw_modals() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new();
    state.modal = ModalState::Search;
    state.text_input_buffer = "Starboy".to_string();

    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    state.modal = ModalState::Help;
    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    state.modal = ModalState::CreatePlaylist;
    state.text_input_buffer = "My Chill Hits".to_string();
    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    state.playlists = vec![Playlist {
        id: "p1".to_string(),
        name: "My Chill Hits".to_string(),
        description: None,
        is_public: false,
        track_count: Some(5),
    }];
    state.modal = ModalState::AddToPlaylist {
        song: Song {
            id: "s1".to_string(),
            name: "Song 1".to_string(),
            artist_name: "Artist 1".to_string(),
            album_name: None,
            duration_in_millis: 180000,
            track_number: None,
            release_date: None,
            url: None,
            catalog_id: None,
        },
    };
    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();
}

#[test]
fn test_ui_draw_scrolled_table_renders_selected_row() {
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new();
    state.songs = (0..50)
        .map(|i| Song {
            id: format!("song_{i}"),
            name: format!("UniqueSongName_{i}"),
            artist_name: "Artist".to_string(),
            album_name: Some("Album".to_string()),
            duration_in_millis: 180_000,
            track_number: Some(i + 1),
            release_date: None,
            url: None,
            catalog_id: None,
        })
        .collect();

    state.focused_panel = FocusedPanel::MainContent;
    state.active_view = ActiveView::LibrarySongs;
    state.selected_index = 35;

    terminal
        .draw(|f| {
            draw(f, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    // In a 20-line terminal with header and footer, only ~12 rows fit.
    // With windowed scrolling, row 35 MUST be rendered inside the buffer!
    let mut rendered_text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            rendered_text.push_str(buffer[(x, y)].symbol());
        }
        rendered_text.push('\n');
    }

    assert!(
        rendered_text.contains("UniqueSongName_35"),
        "The selected row (index 35) must be visible in the rendered buffer when scrolled!"
    );
}
