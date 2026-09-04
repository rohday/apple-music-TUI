use crate::api::models::{Album, Artist, Playlist, SearchResults, Song};
use crate::playback::types::PlaybackStatus;
use std::collections::HashMap;

/// Playback action deferred by an effect, executed by the main loop which owns
/// the playback engine handle.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingPlayback {
    /// Start playing the queue at the given index.
    QueueStart(usize),
}

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

    /// Sidebar layout: sections with headers; headers are skipped during
    /// navigation. Index positions are stable and drive `sidebar_index`.
    pub fn sidebar_items() -> Vec<SidebarItem> {
        vec![
            SidebarItem::View(ActiveView::Search),
            SidebarItem::Header("Library"),
            SidebarItem::View(ActiveView::LibrarySongs),
            SidebarItem::View(ActiveView::LibraryAlbums),
            SidebarItem::View(ActiveView::LibraryArtists),
            SidebarItem::View(ActiveView::Playlists),
            SidebarItem::View(ActiveView::RecentlyPlayed),
            SidebarItem::Header("Playback"),
            SidebarItem::View(ActiveView::Queue),
            SidebarItem::LyricsToggle,
        ]
    }
}

/// One entry in the sidebar: a selectable view, the lyrics panel toggle, or
/// a non-selectable section header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarItem {
    View(ActiveView),
    LyricsToggle,
    Header(&'static str),
}

impl ActiveView {
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

    // Async job pipeline & cache
    pub pending_jobs: Vec<crate::app::job::Job>,
    pub cache: crate::app::cache::DataCache,
    pub pending_playback: Option<PendingPlayback>,

    // Library songs pagination
    pub songs_offset: usize,
    pub songs_has_more: bool,
    pub songs_loading_more: bool,

    // In-memory artwork cache (song id -> decoded cover image)
    pub artwork: HashMap<String, image::RgbImage>,
    pub artwork_loading: std::collections::HashSet<String>,
    pub show_now_playing: bool,
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
            // Start on Library Songs
            sidebar_index: ActiveView::sidebar_items()
                .iter()
                .position(|item| matches!(item, SidebarItem::View(ActiveView::LibrarySongs)))
                .unwrap_or(0),
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
            pending_jobs: Vec::new(),
            cache: crate::app::cache::DataCache::default(),
            pending_playback: None,
            songs_offset: 0,
            songs_has_more: false,
            songs_loading_more: false,
            artwork: HashMap::new(),
            artwork_loading: std::collections::HashSet::new(),
            show_now_playing: false,
        }
    }

    /// Enqueues a background job and marks the UI as loading.
    pub fn enqueue_job(&mut self, job: crate::app::job::Job) {
        self.pending_jobs.push(job);
        self.is_loading = true;
    }

    /// Removes the queue entry at `idx` (Queue view `d` key).
    pub fn remove_from_queue(&mut self, idx: usize) -> Option<Song> {
        if idx < self.queue.len() {
            let song = self.queue.remove(idx);
            if self.selected_index >= self.queue.len() && !self.queue.is_empty() {
                self.selected_index = self.queue.len() - 1;
            }
            Some(song)
        } else {
            None
        }
    }

    /// Moves the queue entry at `idx` up (`up = true`) or down one slot.
    /// Returns true when the move happened.
    pub fn move_queue_item(&mut self, idx: usize, up: bool) -> bool {
        let (from, to) = if up {
            match idx.checked_sub(1) {
                Some(prev) => (idx, prev),
                None => return false,
            }
        } else {
            (idx, idx + 1)
        };
        if to >= self.queue.len() || from == to {
            return false;
        }
        self.queue.swap(from, to);
        if up {
            self.selected_index = self.selected_index.saturating_sub(1);
        } else if self.selected_index + 1 < self.queue.len() {
            self.selected_index += 1;
        }
        true
    }

    /// True when the selection is close enough to the end of the library
    /// songs list that the next page should be fetched.
    pub fn should_fetch_next_songs_page(&self) -> bool {
        self.active_view == ActiveView::LibrarySongs
            && self.filter_query.is_empty()
            && self.songs_has_more
            && !self.songs_loading_more
            && self.selected_index + 20 >= self.songs.len()
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

    /// Moves the sidebar selection down, skipping section headers. Lands on
    /// a view (switching to it) or the lyrics toggle (view unchanged).
    pub fn move_sidebar_down(&mut self) {
        let items = ActiveView::sidebar_items();
        let mut i = self.sidebar_index;
        while i + 1 < items.len() {
            i += 1;
            if matches!(items[i], SidebarItem::Header(_)) {
                continue;
            }
            self.sidebar_index = i;
            if let SidebarItem::View(view) = items[i] {
                self.active_view = view;
                self.selected_index = 0;
            }
            return;
        }
    }

    /// Moves the sidebar selection up, skipping section headers.
    pub fn move_sidebar_up(&mut self) {
        let items = ActiveView::sidebar_items();
        let mut i = self.sidebar_index;
        while i > 0 {
            i -= 1;
            if matches!(items[i], SidebarItem::Header(_)) {
                continue;
            }
            self.sidebar_index = i;
            if let SidebarItem::View(view) = items[i] {
                self.active_view = view;
                self.selected_index = 0;
            }
            return;
        }
    }

    /// Aligns `sidebar_index` with `active_view` after programmatic view
    /// changes (search results, station start, etc.).
    pub fn sync_sidebar_to_view(&mut self) {
        if let Some(idx) = ActiveView::sidebar_items()
            .iter()
            .position(|item| matches!(item, SidebarItem::View(v) if *v == self.active_view))
        {
            self.sidebar_index = idx;
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
