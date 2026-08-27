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
        }
    }

    pub fn current_list_len(&self) -> usize {
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

    pub fn close_modal(&mut self) {
        self.modal = ModalState::None;
        self.text_input_buffer.clear();
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn get_selected_song(&self) -> Option<Song> {
        let idx = self.selected_index;
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
