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

use std::sync::atomic::{AtomicU32, Ordering};

pub struct PlaybackEngine {
    cmd_sender: Sender<PlaybackCommand>,
    status_receiver: Arc<Mutex<Receiver<PlaybackStatus>>>,
    current_status: Arc<Mutex<PlaybackStatus>>,
    browser_pid: Arc<AtomicU32>,
    is_mock: bool,
}

impl Drop for PlaybackEngine {
    fn drop(&mut self) {
        let pid = self.browser_pid.load(Ordering::SeqCst);
        if pid > 0 {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
                libc::kill(-(pid as i32), libc::SIGTERM);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        if let Ok(dir) = crate::config::Config::get_profile_dir() {
            crate::config::Config::clean_stale_browser_locks(&dir);
        }
    }
}

impl PlaybackEngine {
    #[allow(clippy::unused_async)]
    pub async fn new(browser_bin: Option<PathBuf>, mock_mode: bool) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlaybackCommand>(64);
        let (status_tx, status_rx) = mpsc::channel::<PlaybackStatus>(64);
        let current_status = Arc::new(Mutex::new(PlaybackStatus::default()));
        let browser_pid = Arc::new(AtomicU32::new(0));

        if mock_mode {
            info!("Initializing mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(cmd_rx, status_tx, cur_status_clone));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                browser_pid,
                is_mock: true,
            });
        }

        let bin = browser_bin.or_else(find_browser_binary);
        if bin.is_none() {
            warn!("No browser binary found, falling back to mock playback engine");
            let cur_status_clone = current_status.clone();
            tokio::spawn(run_mock_playback_loop(cmd_rx, status_tx, cur_status_clone));
            return Ok(Self {
                cmd_sender: cmd_tx,
                status_receiver: Arc::new(Mutex::new(status_rx)),
                current_status,
                browser_pid,
                is_mock: true,
            });
        }

        let browser_path = bin.unwrap();
        info!("Launching headless browser from: {:?}", browser_path);

        let cur_status_clone = current_status.clone();
        let browser_pid_clone = browser_pid.clone();
        tokio::spawn(async move {
            if let Err(e) = run_browser_playback_loop(
                browser_path,
                cmd_rx,
                status_tx,
                cur_status_clone,
                browser_pid_clone,
            )
            .await
            {
                error!("Browser playback loop exited with error: {:?}", e);
            }
        });

        Ok(Self {
            cmd_sender: cmd_tx,
            status_receiver: Arc::new(Mutex::new(status_rx)),
            current_status,
            browser_pid,
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

    pub fn get_cmd_sender(&self) -> Sender<PlaybackCommand> {
        self.cmd_sender.clone()
    }

    pub fn get_status_store(&self) -> Arc<Mutex<PlaybackStatus>> {
        self.current_status.clone()
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
                        // Track finished, advance queue or repeat
                        if status.repeat == RepeatMode::One {
                            status.current_time_secs = 0.0;
                        } else if queue_idx + 1 < queue.len() {
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
            cmd_opt = cmd_rx.recv() => {
                let cmd = match cmd_opt {
                    Some(c) => c,
                    None => break,
                };
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
                        status.current_song = None;
                        let _ = status_tx.send(status.clone()).await;
                        *status_store.lock().await = status.clone();
                    }
                    PlaybackCommand::Enqueue(songs) => {
                        queue.extend(songs);
                    }
                    PlaybackCommand::RemoveFromQueue(index) => {
                        if index < queue.len() {
                            queue.remove(index);
                            if index < queue_idx {
                                queue_idx -= 1;
                            } else if index == queue_idx && queue_idx >= queue.len() {
                                queue_idx = queue.len().saturating_sub(1);
                            }
                        }
                    }
                    PlaybackCommand::MoveQueueItem(from, to) => {
                        if from < queue.len() && to < queue.len() && from != to {
                            let song = queue.remove(from);
                            queue.insert(to, song);
                            if from == queue_idx {
                                queue_idx = to;
                            } else if from < queue_idx && to >= queue_idx {
                                queue_idx -= 1;
                            } else if from > queue_idx && to <= queue_idx {
                                queue_idx += 1;
                            }
                        }
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
    browser_pid: Arc<AtomicU32>,
) -> Result<()> {
    let profile_dir = crate::config::Config::get_profile_dir()?;
    crate::config::Config::clean_stale_browser_locks(&profile_dir);

    let config = BrowserConfig::builder()
        .with_head() // Prevents chromiumoxide from injecting --mute-audio and --ozone-platform=headless
        .user_data_dir(&profile_dir)
        .arg("--headless=new")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-sync")
        .arg("--autoplay-policy=no-user-gesture-required")
        .arg("--enable-widevine-cdm")
        .arg("--disable-background-timer-throttling")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-renderer-backgrounding")
        .chrome_executable(browser_path)
        .build()
        .map_err(|e| anyhow::anyhow!("BrowserConfig error: {}", e))?;

    let launch_res = Browser::launch(config).await;
    let (mut browser, mut handler) = match launch_res {
        Ok((b, h)) => (b, h),
        Err(e) => {
            warn!(
                "Failed to launch headless browser: {:#}. Falling back to mock playback engine.",
                e
            );
            run_mock_playback_loop(cmd_rx, status_tx, status_store).await;
            return Ok(());
        }
    };

    if let Some(child) = browser.get_mut_child() {
        if let Some(pid) = child.inner.id() {
            browser_pid.store(pid, Ordering::SeqCst);
            info!("Browser process launched with PID: {}", pid);
        }
    }
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                tracing::debug!("Playback browser event debug: {:?}", e);
            }
        }
    });

    let page = match browser.new_page("https://music.apple.com").await {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "Failed to navigate to Apple Music: {:#}. Falling back to mock playback engine.",
                e
            );
            browser_pid.store(0, Ordering::SeqCst);
            let _ = browser.close().await;
            handler_handle.abort();
            run_mock_playback_loop(cmd_rx, status_tx, status_store).await;
            return Ok(());
        }
    };
    info!("Navigated to Apple Music web player");

    // Inject active user authentication token cookies into browser session
    let auth = crate::config::AuthConfig::load();
    if let Some(user_token) = &auth.music_user_token {
        use chromiumoxide::cdp::browser_protocol::network::CookieParam;
        if let Ok(c1) = CookieParam::builder()
            .name("media-user-token")
            .value(user_token)
            .domain(".music.apple.com")
            .path("/")
            .secure(true)
            .build()
        {
            let _ = page.set_cookie(c1).await;
        }
        if let Ok(c2) = CookieParam::builder()
            .name("media-user-token")
            .value(user_token)
            .domain(".apple.com")
            .path("/")
            .secure(true)
            .build()
        {
            let _ = page.set_cookie(c2).await;
        }
    }

    let mut status = PlaybackStatus {
        volume: 80,
        ..Default::default()
    };

    let mut current_queue: Vec<Song> = Vec::new();
    let mut current_queue_idx: usize = 0;

    // Wait for MusicKit to be ready in browser
    for _ in 0..15 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let check_js = "!!(window.MusicKit && window.MusicKit.getInstance())";
        if let Ok(eval) = page.evaluate(check_js).await {
            if eval.into_value::<bool>().unwrap_or(false) {
                info!("MusicKit instance is ready in headless browser");
                break;
            }
        }
    }

    let mut poll_interval = tokio::time::interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                let js_eval = r#"
                    (() => {
                        try {
                            const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                            if (!mk) return { ok: false };
                            const item = mk.nowPlayingItem;
                            return {
                                ok: true,
                                isPlaying: !!mk.isPlaying,
                                playbackState: mk.playbackState,
                                currentTime: mk.currentPlaybackTime || 0,
                                duration: mk.currentPlaybackDuration || 0,
                                volume: Math.round((mk.volume || 0.8) * 100),
                                nowPlayingId: item ? (item.id || item.attributes?.playParams?.id || item.attributes?.playParams?.catalogId) : null,
                                nowPlayingTitle: item ? item.title : null,
                                nowPlayingArtist: item ? item.artistName : null,
                                nowPlayingAlbum: item ? item.albumName : null
                            };
                        } catch (e) {
                            return { ok: false, error: e.toString() };
                        }
                    })()
                "#;
                if let Ok(eval_result) = page.evaluate(js_eval).await {
                    if let Ok(val) = eval_result.into_value::<serde_json::Value>() {
                        if val.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let playing = val.get("isPlaying").and_then(|v| v.as_bool()).unwrap_or(false);
                            status.state = if playing {
                                PlaybackState::Playing
                            } else if status.current_song.is_some() {
                                PlaybackState::Paused
                            } else {
                                PlaybackState::Stopped
                            };

                            if let Some(cur) = val.get("currentTime").and_then(|v| v.as_f64()) {
                                status.current_time_secs = cur;
                            }
                            if let Some(dur) = val.get("duration").and_then(|v| v.as_f64()) {
                                if dur > 0.0 {
                                    status.duration_secs = dur;
                                }
                            }
                            if let Some(vol) = val.get("volume").and_then(|v| v.as_u64()) {
                                status.volume = vol as u8;
                            }

                            // Match song in current_queue if available
                            if let Some(np_id) = val.get("nowPlayingId").and_then(|v| v.as_str()) {
                                if let Some((idx, matched_song)) = current_queue
                                    .iter()
                                    .enumerate()
                                    .find(|(_, s)| s.playback_id() == np_id || s.id == np_id)
                                {
                                    current_queue_idx = idx;
                                    status.current_song = Some(matched_song.clone());
                                }
                            }

                            let _ = status_tx.send(status.clone()).await;
                            *status_store.lock().await = status.clone();
                        }
                    }
                }
            }
            cmd_opt = cmd_rx.recv() => {
                let cmd = match cmd_opt {
                    Some(c) => c,
                    None => break,
                };
                match cmd {
                    PlaybackCommand::PlaySong(song) => {
                        let pid = song.playback_id().to_string();
                        current_queue = vec![song.clone()];
                        current_queue_idx = 0;

                        status.current_song = Some(song.clone());
                        status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                        status.current_time_secs = 0.0;
                        status.state = PlaybackState::Playing;

                        let play_js = format!(r#"
                            (async () => {{
                                try {{
                                    let mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                    for (let i = 0; i < 10 && !mk; i++) {{
                                        await new Promise(r => setTimeout(r, 500));
                                        mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                    }}
                                    if (!mk) return {{ ok: false, err: 'No MusicKit' }};
                                    await mk.setQueue({{ song: '{}' }});
                                    for (const a of document.querySelectorAll('audio')) {{
                                        a.muted = false;
                                        a.volume = {};
                                    }}
                                    await mk.play();
                                    return {{ ok: true }};
                                }} catch(e) {{
                                    return {{ ok: false, err: e.toString() }};
                                }}
                            }})()
                        "#, pid, (status.volume as f64) / 100.0);

                        let _ = page.evaluate(play_js).await;
                    }
                    PlaybackCommand::SetQueueAndPlay(songs, idx) => {
                        current_queue = songs.clone();
                        current_queue_idx = idx;

                        if idx < songs.len() {
                            let song = &songs[idx];
                            let target_id = song.playback_id().to_string();
                            status.current_song = Some(song.clone());
                            status.duration_secs = (song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;

                            let upcoming_ids: Vec<String> = songs
                                .iter()
                                .skip(idx + 1)
                                .take(15)
                                .map(|s| s.playback_id().to_string())
                                .collect();
                            let upcoming_json = serde_json::to_string(&upcoming_ids).unwrap_or_else(|_| "[]".to_string());

                            let play_js = format!(r#"
                                (async () => {{
                                    try {{
                                        let mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                        for (let i = 0; i < 10 && !mk; i++) {{
                                            await new Promise(r => setTimeout(r, 500));
                                            mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                        }}
                                        if (!mk) return {{ ok: false, err: 'No MusicKit' }};
                                        
                                        // Set target song and play immediately
                                        await mk.setQueue({{ song: '{}' }});
                                        for (const a of document.querySelectorAll('audio')) {{
                                            a.muted = false;
                                            a.volume = {};
                                        }}
                                        await mk.play();

                                        // Append upcoming tracks non-blockingly
                                        const upcoming = {};
                                        (async () => {{
                                            for (const nextId of upcoming) {{
                                                try {{
                                                    await mk.playLater({{ song: nextId }});
                                                }} catch (err) {{
                                                    console.warn("Could not enqueue", nextId, err);
                                                }}
                                            }}
                                        }})();

                                        return {{ ok: true }};
                                    }} catch(e) {{
                                        return {{ ok: false, err: e.toString() }};
                                    }}
                                }})()
                            "#, target_id, (status.volume as f64) / 100.0, upcoming_json);

                            let _ = page.evaluate(play_js).await;
                        }
                    }
                    PlaybackCommand::TogglePlayPause => {
                        let toggle_js = r#"
                            (async () => {
                                const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                if (mk) {
                                    if (mk.isPlaying) {
                                        mk.pause();
                                    } else {
                                        for (const a of document.querySelectorAll('audio')) {
                                            a.muted = false;
                                        }
                                        await mk.play();
                                    }
                                }
                            })()
                        "#;
                        let _ = page.evaluate(toggle_js).await;
                    }
                    PlaybackCommand::Pause => {
                        status.state = PlaybackState::Paused;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().pause();").await;
                    }
                    PlaybackCommand::Resume => {
                        status.state = PlaybackState::Playing;
                        let resume_js = r#"
                            (async () => {
                                const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                if (mk) {
                                    for (const a of document.querySelectorAll('audio')) {
                                        a.muted = false;
                                    }
                                    await mk.play();
                                }
                            })()
                        "#;
                        let _ = page.evaluate(resume_js).await;
                    }
                    PlaybackCommand::Next => {
                        if !current_queue.is_empty() && current_queue_idx + 1 < current_queue.len() {
                            current_queue_idx += 1;
                            let next_song = &current_queue[current_queue_idx];
                            let next_id = next_song.playback_id().to_string();
                            status.current_song = Some(next_song.clone());
                            status.duration_secs = (next_song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;

                            let next_js = format!(r#"
                                (async () => {{
                                    const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                    if (mk) {{
                                        try {{
                                            await mk.skipToNextItem();
                                        }} catch (err) {{
                                            await mk.setQueue({{ song: '{}' }});
                                        }}
                                        for (const a of document.querySelectorAll('audio')) {{
                                            a.muted = false;
                                        }}
                                        await mk.play();
                                    }}
                                }})()
                            "#, next_id);
                            let _ = page.evaluate(next_js).await;
                        }
                    }
                    PlaybackCommand::Previous => {
                        if !current_queue.is_empty() && current_queue_idx > 0 {
                            current_queue_idx -= 1;
                            let prev_song = &current_queue[current_queue_idx];
                            let prev_id = prev_song.playback_id().to_string();
                            status.current_song = Some(prev_song.clone());
                            status.duration_secs = (prev_song.duration_in_millis as f64) / 1000.0;
                            status.current_time_secs = 0.0;
                            status.state = PlaybackState::Playing;

                            let prev_js = format!(r#"
                                (async () => {{
                                    const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                    if (mk) {{
                                        try {{
                                            if (mk.currentPlaybackTime > 3.0) {{
                                                await mk.seekToTime(0);
                                            }} else {{
                                                await mk.skipToPreviousItem();
                                            }}
                                        }} catch (err) {{
                                            await mk.setQueue({{ song: '{}' }});
                                        }}
                                        for (const a of document.querySelectorAll('audio')) {{
                                            a.muted = false;
                                        }}
                                        await mk.play();
                                    }}
                                }})()
                            "#, prev_id);
                            let _ = page.evaluate(prev_js).await;
                        }
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
                        let js = format!(r#"
                            (() => {{
                                const mk = window.MusicKit ? window.MusicKit.getInstance() : null;
                                if (mk) mk.volume = {};
                                for (const a of document.querySelectorAll('audio')) {{
                                    a.volume = {};
                                }}
                            }})()
                        "#, vol_f, vol_f);
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
                        status.current_time_secs = 0.0;
                        status.current_song = None;
                        let _ = page.evaluate("window.MusicKit && window.MusicKit.getInstance().stop();").await;
                    }
                    PlaybackCommand::Enqueue(songs) => {
                        // Best-effort: append to the MusicKit queue via playLater.
                        for song in &songs {
                            let id = song.playback_id().to_string();
                            let js = format!(
                                "window.MusicKit && window.MusicKit.getInstance().playLater({{ song: '{}' }}).catch(() => {{}});",
                                id
                            );
                            let _ = page.evaluate(js).await;
                        }
                        current_queue.extend(songs);
                    }
                    PlaybackCommand::RemoveFromQueue(index) => {
                        // MusicKit's queue cannot be edited retroactively; keep
                        // our mirrored queue in sync so Next/Previous metadata
                        // stays consistent.
                        if index < current_queue.len() {
                            current_queue.remove(index);
                            if index < current_queue_idx {
                                current_queue_idx -= 1;
                            } else if index == current_queue_idx
                                && current_queue_idx >= current_queue.len()
                            {
                                current_queue_idx = current_queue.len().saturating_sub(1);
                            }
                        }
                    }
                    PlaybackCommand::MoveQueueItem(from, to) => {
                        if from < current_queue.len() && to < current_queue.len() && from != to {
                            let song = current_queue.remove(from);
                            current_queue.insert(to, song);
                            if from == current_queue_idx {
                                current_queue_idx = to;
                            } else if from < current_queue_idx && to >= current_queue_idx {
                                current_queue_idx -= 1;
                            } else if from > current_queue_idx && to <= current_queue_idx {
                                current_queue_idx += 1;
                            }
                        }
                    }
                }
                let _ = status_tx.send(status.clone()).await;
                *status_store.lock().await = status.clone();
            }
        }
    }

    browser_pid.store(0, Ordering::SeqCst);
    let _ = browser.close().await;
    handler_handle.abort();
    Ok(())
}
