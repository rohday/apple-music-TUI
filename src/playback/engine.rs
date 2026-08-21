use crate::api::models::Song;
use crate::config::find_browser_binary;
use crate::playback::types::{PlaybackCommand, PlaybackState, PlaybackStatus, RepeatMode};
use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub struct PlaybackEngine {
    cmd_sender: Sender<PlaybackCommand>,
    status_receiver: Arc<Mutex<Receiver<PlaybackStatus>>>,
    current_status: Arc<Mutex<PlaybackStatus>>,
    is_mock: bool,
}

impl PlaybackEngine {
    pub async fn new(browser_bin: Option<PathBuf>, mock_mode: bool) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlaybackCommand>(64);
        let (status_tx, status_rx) = mpsc::channel::<PlaybackStatus>(64);
        let current_status = Arc::new(Mutex::new(PlaybackStatus::default()));

        if mock_mode {
            info!("Initializing mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(
                cmd_rx,
                status_tx,
                cur_status_clone,
            ));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                is_mock: true,
            });
        }

        let bin = browser_bin.or_else(find_browser_binary);
        if bin.is_none() {
            warn!("No browser binary found, falling back to mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(
                cmd_rx,
                status_tx,
                cur_status_clone,
            ));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                is_mock: true,
            });
        }

        let browser_path = bin.unwrap();
        info!("Launching headless browser from: {:?}", browser_path);

        let cur_status_clone = current_status.clone();
        tokio::spawn(async move {
            if let Err(e) =
                run_browser_playback_loop(browser_path, cmd_rx, status_tx, cur_status_clone).await
            {
                error!("Browser playback loop exited with error: {:?}", e);
            }
        });

        Ok(Self {
            cmd_sender: cmd_tx,
            status_receiver: Arc::new(Mutex::new(status_rx)),
            current_status,
            is_mock: false,
        })
    }

    pub async fn send_command(&self, cmd: PlaybackCommand) -> Result<()> {
        self.cmd_sender
            .send(cmd)
            .await
            .context("Failed to send playback command")
    }

    pub fn get_status_receiver(&self) -> Arc<Mutex<Receiver<PlaybackStatus>>> {
        self.status_receiver.clone()
    }

    pub async fn get_current_status(&self) -> PlaybackStatus {
        self.current_status.lock().await.clone()
    }

    pub fn is_mock(&self) -> bool {
        self.is_mock
    }
}

async fn run_mock_playback_loop(
    mut cmd_rx: Receiver<PlaybackCommand>,
    status_tx: Sender<PlaybackStatus>,
    status_store: Arc<Mutex<PlaybackStatus>>,
) {
    let mut status = PlaybackStatus {
        volume: 80,
        ..Default::default()
    };
    let mut queue: Vec<Song> = Vec::new();
    let mut queue_idx = 0;
    let mut ticker = tokio::time::interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if status.state == PlaybackState::Playing {
                    status.current_time_secs += 0.25;
                    if status.duration_secs > 0.0 && status.current_time_secs >= status.duration_secs {
                        // Track finished, advance queue
                        if queue_idx + 1 < queue.len() {
                            queue_idx += 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                        } else if status.repeat == RepeatMode::All && !queue.is_empty() {
                            queue_idx = 0;
                            let song = &queue[0];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                        } else {
                            status.state = PlaybackState::Stopped;
                            status.current_time_secs = 0.0;
                        }
                    }
                    let _ = status_tx.send(status.clone()).await;
                    *status_store.lock().await = status.clone();
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    PlaybackCommand::PlaySong(song) => {
                        queue = vec![song.clone()];
                        queue_idx = 0;
                        status.current_song = Some(song.clone());
                        status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                        status.current_time_secs = 0.0;
                        status.state = PlaybackState::Playing;
                    }
                    PlaybackCommand::SetQueueAndPlay(new_queue, start_idx) => {
                        if !new_queue.is_empty() && start_idx < new_queue.len() {
                            queue = new_queue;
                            queue_idx = start_idx;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::TogglePlayPause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                        } else if status.current_song.is_some() {
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Pause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                        }
                    }
                    PlaybackCommand::Resume => {
                        if status.current_song.is_some() {
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Next => {
                        if queue_idx + 1 < queue.len() {
                            queue_idx += 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Previous => {
                        if status.current_time_secs > 3.0 || queue_idx == 0 {
                            status.current_time_secs = 0.0;
                        } else if queue_idx > 0 {
                            queue_idx -= 1;
                            let song = &queue[queue_idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;
                        }
                    }
                    PlaybackCommand::Seek(pos) => {
                        status.current_time_secs = pos.clamp(0.0, status.duration_secs);
                    }
                    PlaybackCommand::SeekRelative(delta) => {
                        status.current_time_secs = (status.current_time_secs + delta).clamp(0.0, status.duration_secs);
                    }
                    PlaybackCommand::SetVolume(vol) => {
                        status.volume = vol.min(100);
                    }
                    PlaybackCommand::ToggleShuffle => {
                        status.shuffle = !status.shuffle;
                    }
                    PlaybackCommand::CycleRepeat => {
                        status.repeat = status.repeat.cycle();
                    }
                    PlaybackCommand::Stop => {
                        status.state = PlaybackState::Stopped;
                        status.current_time_secs = 0.0;
                        let _ = status_tx.send(status.clone()).await;
                        *status_store.lock().await = status.clone();
                        break;
                    }
                }
                let _ = status_tx.send(status.clone()).await;
                *status_store.lock().await = status.clone();
            }
        }
    }
}

async fn run_browser_playback_loop(
    browser_path: PathBuf,
    mut cmd_rx: Receiver<PlaybackCommand>,
    status_tx: Sender<PlaybackStatus>,
    status_store: Arc<Mutex<PlaybackStatus>>,
) -> Result<()> {
    let config = BrowserConfig::builder()
        .arg("--autoplay-policy=no-user-gesture-required")
        .arg("--enable-widevine-cdm")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-setuid-sandbox")
        .chrome_executable(browser_path)
        .build()
        .map_err(|e| anyhow::anyhow!("BrowserConfig error: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                error!("Browser handler error: {:?}", e);
                break;
            }
        }
    });

    let page = browser.new_page("https://music.apple.com").await?;
    info!("Navigated to Apple Music web player");

    let mut status = PlaybackStatus {
        volume: 80,
        ..Default::default()
    };

    let mut poll_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                let js_eval = r#"
                    (() => {
                        try {
                            const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                            if (!mk) return { ok: false };
                            return {
                                ok: true,
                                isPlaying: mk.isPlaying,
                                currentTime: mk.currentPlaybackTime || 0,
                                duration: mk.currentPlaybackDuration || 0,
                                volume: Math.round((mk.volume || 0.8) * 100)
                            };
                        } catch (e) {
                            return { ok: false, error: e.toString() };
                        }
                    })()
                "#;
                if let Ok(eval_result) = page.evaluate(js_eval).await {
                    if let Some(val) = eval_result.into_value::<serde_json::Value>().ok() {
                        if val.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            if let Some(playing) = val.get("isPlaying").and_then(|v| v.as_bool()) {
                                status.state = if playing {
                                    PlaybackState::Playing
                                } else if status.current_song.is_some() {
                                    PlaybackState::Paused
                                } else {
                                    PlaybackState::Stopped
                                };
                            }
                            if let Some(cur) = val.get("currentTime").and_then(|v| v.as_f64()) {
                                status.current_time_secs = cur;
                            }
                            if let Some(dur) = val.get("duration").and_then(|v| v.as_f64()) {
                                if dur > 0.0 {
                                    status.duration_secs = dur;
                                }
                            }
                            let _ = status_tx.send(status.clone()).await;
                            *status_store.lock().await = status.clone();
                        }
                    }
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    PlaybackCommand::PlaySong(song) => {
                        status.current_song = Some(song.clone());
                        status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                        status.current_time_secs = 0.0;
                        status.state = PlaybackState::Playing;

                        let js = format!(
                            "window.MusicKit && window.MusicKit.getInstance().setQueue({{ song: '{}' }}).then(() => window.MusicKit.getInstance().play());",
                            song.id
                        );
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SetQueueAndPlay(songs, idx) => {
                        if idx < songs.len() {
                            let song = &songs[idx];
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;

                            let song_ids: Vec<String> = songs.iter().map(|s| format!("'{}'", s.id)).collect();
                            let js = format!(
                                "window.MusicKit && window.MusicKit.getInstance().setQueue({{ songs: [{}] }}, {}).then(() => window.MusicKit.getInstance().play());",
                                song_ids.join(","), idx
                            );
                            let _ = page.evaluate(js).await;
                        }
                    }
                    PlaybackCommand::TogglePlayPause => {
                        if status.state == PlaybackState::Playing {
                            status.state = PlaybackState::Paused;
                            let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().pause();").await;
                        } else {
                            status.state = PlaybackState::Playing;
                            let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().play();").await;
                        }
                    }
                    PlaybackCommand::Pause => {
                        status.state = PlaybackState::Paused;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().pause();").await;
                    }
                    PlaybackCommand::Resume => {
                        status.state = PlaybackState::Playing;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().play();").await;
                    }
                    PlaybackCommand::Next => {
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().skipToNextItem();").await;
                    }
                    PlaybackCommand::Previous => {
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().skipToPreviousItem();").await;
                    }
                    PlaybackCommand::Seek(pos) => {
                        let js = format!("window.MusicKit && window.MusicKit.getInstance().seekToTime({});", pos);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SeekRelative(delta) => {
                        let new_pos = (status.current_time_secs + delta).max(0.0);
                        let js = format!("window.MusicKit && window.MusicKit.getInstance().seekToTime({});", new_pos);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::SetVolume(vol) => {
                        status.volume = vol.min(100);
                        let vol_f = (vol as f64) / 100.0;
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().volume = {};", vol_f);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::ToggleShuffle => {
                        status.shuffle = !status.shuffle;
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().shuffleMode = {};", if status.shuffle { 1 } else { 0 });
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::CycleRepeat => {
                        status.repeat = status.repeat.cycle();
                        let mode_int = match status.repeat {
                            RepeatMode::Off => 0,
                            RepeatMode::All => 1,
                            RepeatMode::One => 2,
                        };
                        let js = format!("if (window.MusicKit) window.MusicKit.getInstance().repeatMode = {};", mode_int);
                        let _ = page.evaluate(js).await;
                    }
                    PlaybackCommand::Stop => {
                        status.state = PlaybackState::Stopped;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().stop();").await;
                        break;
                    }
                }
                let _ = status_tx.send(status.clone()).await;
                *status_store.lock().await = status.clone();
            }
        }
    }

    let _ = browser.close().await;
    handler_handle.abort();
    Ok(())
}
