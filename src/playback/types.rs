use crate::api::models::Song;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "One",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub state: PlaybackState,
    pub current_time_secs: f64,
    pub duration_secs: f64,
    pub current_song: Option<Song>,
    pub volume: u8,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl PlaybackStatus {
    pub fn progress_ratio(&self) -> f64 {
        if self.duration_secs <= 0.0 {
            0.0
        } else {
            (self.current_time_secs / self.duration_secs).clamp(0.0, 1.0)
        }
    }

    pub fn formatted_position(&self) -> String {
        let cur = self.current_time_secs as u64;
        let dur = self.duration_secs as u64;
        format!(
            "{}:{:02} / {}:{:02}",
            cur / 60,
            cur % 60,
            dur / 60,
            dur % 60
        )
    }
}

#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    PlaySong(Song),
    SetQueueAndPlay(Vec<Song>, usize),
    TogglePlayPause,
    Pause,
    Resume,
    Next,
    Previous,
    Seek(f64),
    SeekRelative(f64),
    SetVolume(u8),
    ToggleShuffle,
    CycleRepeat,
    Stop,
    /// Append songs to the end of the current queue.
    Enqueue(Vec<Song>),
    /// Remove the song at `index` from the queue.
    RemoveFromQueue(usize),
    /// Move the song at `from` so it ends up at `to`.
    MoveQueueItem(usize, usize),
}
