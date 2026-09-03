use crate::api::models::{Album, Artist, Playlist, SearchResults, Song};
use crate::playback::types::PlaybackStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Search,
    LibrarySongs,
    LibraryAlbums,
    LibraryArtists,
    Playlists,
    PlaylistDetail,
    RecentlyPlayed,
    Queue,
}

impl ActiveView {
    pub fn all_sidebar_views() -> &'static [ActiveView] {
        &[
            ActiveView::Search,
            ActiveView::LibrarySongs,
            ActiveView::LibraryAlbums,
            ActiveView::LibraryArtists,
            ActiveView::Playlists,
            ActiveView::RecentlyPlayed,
            ActiveView::Queue,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ActiveView::Search => "Search",
            ActiveView::LibrarySongs => "Library Songs",
            ActiveView::LibraryAlbums => "Albums",
            ActiveView::LibraryArtists => "Artists",
            ActiveView::Playlists => "Playlists",
            ActiveView::PlaylistDetail => "Playlist Tracks",
            ActiveView::RecentlyPlayed => "Recently Played",
            ActiveView::Queue => "Queue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Sidebar,
    MainContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    None,
    Search,
    CreatePlaylist,
    AddToPlaylist { song: Song },
    Help,
    AuthPrompt,
    Notification(String),
}

pub struct AppState {
    pub active_view: ActiveView,
    pub focused_panel: FocusedPanel,
    pub modal: ModalState,

    // Navigation & Lists
    pub sidebar_index: usize,
    pub selected_index: usize,
    pub search_query: String,
    pub search_results: SearchResults,

    // Loaded data
    pub songs: Vec<Song>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
    pub active_playlist: Option<Playlist>,
    pub playlist_tracks: Vec<Song>,
    pub recent_tracks: Vec<Song>,
    pub queue: Vec<Song>,

    // Text inputs for modals
    pub text_input_buffer: String,
    pub add_to_playlist_index: usize,

    // Status & Playback
    pub playback: PlaybackStatus,
    pub storefront: String,
    pub is_authenticated: bool,
    pub status_message: Option<String>,
    pub is_loading: bool,
    pub should_quit: bool,
    pub volume: u8,
    pub pending_login: bool,
    pub theme: crate::ui::theme::ThemePreset,
    pub show_visualizer: bool,
    pub filter_query: String,
    pub is_filtering: bool,
    pub show_lyrics: bool,
    pub lyrics: Option<crate::api::lyrics::LyricsData>,
    pub lyrics_loading: bool,
    pub lyrics_song_id: Option<String>,
    pub anim_time: f64,
    pub last_tick_instant: std::time::Instant,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_view: ActiveView::LibrarySongs,
            focused_panel: FocusedPanel::MainContent,
            modal: ModalState::None,
            sidebar_index: 1, // Start at Library Songs
            selected_index: 0,
            search_query: String::new(),
            search_results: SearchResults::default(),
            songs: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
            active_playlist: None,
            playlist_tracks: Vec::new(),
            recent_tracks: Vec::new(),
            queue: Vec::new(),
            text_input_buffer: String::new(),
            add_to_playlist_index: 0,
            playback: PlaybackStatus::default(),
            storefront: "us".to_string(),
            is_authenticated: false,
            status_message: None,
            is_loading: false,
            should_quit: false,
            volume: 80,
            pending_login: false,
            theme: crate::ui::theme::ThemePreset::AppleDark,
            show_visualizer: false,
            filter_query: String::new(),
            is_filtering: false,
            show_lyrics: false,
            lyrics: None,
            lyrics_loading: false,
            lyrics_song_id: None,
            anim_time: 0.0,
            last_tick_instant: std::time::Instant::now(),
        }
    }

    pub fn tick_animation(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_tick_instant).as_secs_f64();
        self.last_tick_instant = now;
        if self.playback.state == crate::playback::types::PlaybackState::Playing {
            self.anim_time += dt;
        }
    }

    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
    }

    pub fn cycle_theme(&mut self) -> crate::ui::theme::ThemePreset {
        self.theme = self.theme.cycle();
        self.theme
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.is_filtering = false;
        self.selected_index = 0;
    }

    pub fn filtered_songs(&self) -> Vec<Song> {
        let songs = match self.active_view {
            ActiveView::LibrarySongs => &self.songs,
            ActiveView::PlaylistDetail => &self.playlist_tracks,
            ActiveView::RecentlyPlayed => &self.recent_tracks,
            ActiveView::Search => &self.search_results.songs,
            ActiveView::Queue => &self.queue,
            _ => &self.songs,
        };

        if self.filter_query.is_empty() {
            return songs.clone();
        }

        let q = self.filter_query.to_lowercase();
        songs
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&q)
                    || s.artist_name.to_lowercase().contains(&q)
                    || s.album_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
            })
            .cloned()
            .collect()
    }

    pub fn current_list_len(&self) -> usize {
        if !self.filter_query.is_empty() {
            match self.active_view {
                ActiveView::LibrarySongs
                | ActiveView::PlaylistDetail
                | ActiveView::RecentlyPlayed
                | ActiveView::Search
                | ActiveView::Queue => return self.filtered_songs().len(),
                ActiveView::Playlists => {
                    let q = self.filter_query.to_lowercase();
                    return self
                        .playlists
                        .iter()
                        .filter(|p| p.name.to_lowercase().contains(&q))
                        .count();
                }
                ActiveView::LibraryAlbums => {
                    let q = self.filter_query.to_lowercase();
                    return self
                        .albums
                        .iter()
                        .filter(|a| {
                            a.name.to_lowercase().contains(&q)
                                || a.artist_name.to_lowercase().contains(&q)
                        })
                        .count();
                }
                ActiveView::LibraryArtists => {
                    let q = self.filter_query.to_lowercase();
                    return self
                        .artists
                        .iter()
                        .filter(|a| a.name.to_lowercase().contains(&q))
                        .count();
                }
            }
        }

        match self.active_view {
            ActiveView::Search => self.search_results.songs.len(),
            ActiveView::LibrarySongs => self.songs.len(),
            ActiveView::LibraryAlbums => self.albums.len(),
            ActiveView::LibraryArtists => self.artists.len(),
            ActiveView::Playlists => self.playlists.len(),
            ActiveView::PlaylistDetail => self.playlist_tracks.len(),
            ActiveView::RecentlyPlayed => self.recent_tracks.len(),
            ActiveView::Queue => self.queue.len(),
        }
    }

    pub fn move_selection_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 && self.selected_index + 1 < len {
            self.selected_index += 1;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_sidebar_down(&mut self) {
        let len = ActiveView::all_sidebar_views().len();
        if self.sidebar_index + 1 < len {
            self.sidebar_index += 1;
            self.active_view = ActiveView::all_sidebar_views()[self.sidebar_index];
            self.selected_index = 0;
        }
    }

    pub fn move_sidebar_up(&mut self) {
        if self.sidebar_index > 0 {
            self.sidebar_index -= 1;
            self.active_view = ActiveView::all_sidebar_views()[self.sidebar_index];
            self.selected_index = 0;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Sidebar => FocusedPanel::MainContent,
            FocusedPanel::MainContent => FocusedPanel::Sidebar,
        };
    }

    pub fn open_search(&mut self) {
        self.text_input_buffer.clear();
        self.modal = ModalState::Search;
    }

    pub fn open_create_playlist(&mut self) {
        self.text_input_buffer.clear();
        self.modal = ModalState::CreatePlaylist;
    }

    pub fn open_add_to_playlist(&mut self, song: Song) {
        self.add_to_playlist_index = 0;
        self.modal = ModalState::AddToPlaylist { song };
    }

    pub fn toggle_help(&mut self) {
        self.modal = match self.modal {
            ModalState::Help => ModalState::None,
            _ => ModalState::Help,
        };
    }

    pub fn open_auth_prompt(&mut self) {
        self.modal = ModalState::AuthPrompt;
    }

    pub fn close_modal(&mut self) {
        self.modal = ModalState::None;
        self.text_input_buffer.clear();
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn get_selected_song(&self) -> Option<Song> {
        let idx = self.selected_index;
        if !self.filter_query.is_empty() {
            return self.filtered_songs().get(idx).cloned();
        }
        match self.active_view {
            ActiveView::Search => self.search_results.songs.get(idx).cloned(),
            ActiveView::LibrarySongs => self.songs.get(idx).cloned(),
            ActiveView::PlaylistDetail => self.playlist_tracks.get(idx).cloned(),
            ActiveView::RecentlyPlayed => self.recent_tracks.get(idx).cloned(),
            ActiveView::Queue => self.queue.get(idx).cloned(),
            _ => None,
        }
    }
}
