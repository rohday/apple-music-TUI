use anyhow::Result;
use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::job::{apply_effect, execute_job};
use apple_tui::app::state::AppState;
use apple_tui::auth::login::{fetch_live_developer_token, launch_interactive_login};
use apple_tui::config::{AuthConfig, Config, DEFAULT_FALLBACK_DEVELOPER_TOKEN};
use apple_tui::events::{
    enqueue_artwork_for_current_song, enqueue_lyrics_for_current_song, handle_key_event,
};
use apple_tui::playback::engine::PlaybackEngine;
use apple_tui::playback::types::PlaybackCommand;
use apple_tui::ui::draw;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(
    name = "appletui",
    version = env!("CARGO_PKG_VERSION"),
    about = "Fast, lightweight Apple Music TUI for Linux"
)]
struct Cli {
    /// Run with mock offline data
    #[arg(short, long)]
    mock: bool,

    /// Specify custom Chrome or Chromium binary path
    #[arg(short, long)]
    browser_path: Option<PathBuf>,

    /// Set Apple Music Developer Token
    #[arg(long)]
    set_dev_token: Option<String>,

    /// Set Apple Music User Token (media-user-token)
    #[arg(long)]
    set_user_token: Option<String>,

    /// Launch interactive login in browser
    #[arg(long)]
    login: bool,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle token setting CLI flags
    if let Some(token) = cli.set_user_token {
        let mut auth = AuthConfig::load();
        auth.music_user_token = Some(token);
        auth.save()?;
        println!("Saved user token to auth.json");
        return Ok(());
    }

    if let Some(token) = cli.set_dev_token {
        let mut auth = AuthConfig::load();
        auth.developer_token = Some(token);
        auth.save()?;
        println!("Saved developer token to auth.json");
        return Ok(());
    }

    if cli.login {
        println!("Launching browser login window...");
        let _auth = launch_interactive_login().await?;
        println!("Successfully authenticated! Token stored in config.");
        return Ok(());
    }

    let mut config = Config::load();
    if cli.mock {
        config.mock_mode = true;
    }
    if let Some(p) = cli.browser_path {
        config.browser_path = Some(p);
    }

    let mut auth = AuthConfig::load();
    if auth.developer_token.is_none() && auth.is_authenticated() {
        let dev_token = fetch_live_developer_token()
            .await
            .unwrap_or_else(|_| DEFAULT_FALLBACK_DEVELOPER_TOKEN.to_string());
        auth.developer_token = Some(dev_token);
        let _ = auth.save();
    }

    let is_auth = auth.is_authenticated() || config.mock_mode;

    let mut client = if config.mock_mode {
        AppleMusicClient::new_mock()
    } else {
        AppleMusicClient::new(auth.developer_token.clone(), auth.music_user_token.clone())
            .unwrap_or_else(|_| AppleMusicClient::new_mock())
    };

    let playback = PlaybackEngine::new(config.browser_path.clone(), config.mock_mode).await?;

    // Spawn MPRIS D-Bus background service for desktop integration (media keys, playerctl)
    tokio::spawn(apple_tui::playback::mpris::start_mpris_background_service(
        playback.get_cmd_sender(),
        playback.get_status_store(),
    ));

    // Setup terminal
    let guard = TerminalGuard;
    enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen, crossterm::cursor::Hide)?;

    // Set panic hook for safety
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        eprintln!("Panic occurred: {info:?}");
    }));

    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    state.is_authenticated = is_auth;
    state.storefront = client
        .get_storefront()
        .await
        .unwrap_or_else(|_| "us".to_string());
    state.volume = config.volume;
    state.theme = config.theme;

    if is_auth {
        // Preload initial library songs & playlists
        match client.get_library_songs(100, 0).await {
            Ok(songs) => {
                state.songs = songs;
                state.songs_offset = state.songs.len();
                state.songs_has_more = state.songs.len() >= 100;
                state.cache.insert_songs(&state.songs);
            }
            Err(e) => {
                state.set_status(format!("Error loading library: {e}"));
            }
        }
        if let Ok(playlists) = client.get_library_playlists().await {
            state.playlists = playlists;
            state.cache.insert_playlists(&state.playlists);
        }
    } else {
        state.open_auth_prompt();
    }

    let (effect_tx, mut effect_rx) = mpsc::channel::<apple_tui::app::job::Effect>(64);
    let mut event_stream = EventStream::new();
    let status_rx_arc = playback.get_status_receiver();

    let mut status_poll = tokio::time::interval(Duration::from_millis(250));
    status_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    #[cfg(unix)]
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    #[cfg(unix)]
    let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;

    while !state.should_quit {
        // Drain queued jobs: execute each on a background task and deliver
        // results back as Effects. The UI thread never awaits the network.
        if !state.pending_jobs.is_empty() {
            let jobs = std::mem::take(&mut state.pending_jobs);
            for job in jobs {
                let client = client.clone();
                let tx = effect_tx.clone();
                tokio::spawn(async move {
                    let effect = execute_job(job, &client).await;
                    let _ = tx.send(effect).await;
                });
            }
        }

        // Serve any deferred playback actions triggered by effects.
        if let Some(action) = state.pending_playback.take() {
            match action {
                apple_tui::app::state::PendingPlayback::QueueStart(idx) => {
                    let list = state.queue.clone();
                    let _ = playback
                        .send_command(PlaybackCommand::SetQueueAndPlay(list, idx))
                        .await;
                }
            }
        }

        terminal.draw(|f| {
            draw(f, &state);
        })?;

        // 60 FPS while playing or animating; low-power idle otherwise.
        let tick_period =
            if state.playback.state == apple_tui::playback::types::PlaybackState::Playing {
                Duration::from_millis(16)
            } else {
                Duration::from_millis(250)
            };

        tokio::select! {
            _ = tokio::time::sleep(tick_period) => {
                state.tick_animation();
            }
            _ = status_poll.tick() => {
                state.playback = playback.get_current_status().await;
                enqueue_artwork_for_current_song(&mut state);
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    if let Err(e) = handle_key_event(key, &mut state, &client, &playback).await {
                        state.set_status(format!("Error: {e}"));
                    }
                    if state.show_lyrics {
                        enqueue_lyrics_for_current_song(&mut state);
                    }
                    if state.pending_login {
                        state.pending_login = false;
                        let _ = disable_raw_mode();
                        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
                        println!("\n[appleTUI] Launching Apple Music browser login...");
                        match launch_interactive_login().await {
                            Ok(new_auth) => {
                                println!("[appleTUI] Authentication successful! Updating session...");
                                state.is_authenticated = true;
                                client = AppleMusicClient::new(
                                    new_auth.developer_token,
                                    new_auth.music_user_token,
                                )
                                .unwrap_or_else(|_| AppleMusicClient::new_mock());
                                if let Ok(storefront) = client.get_storefront().await {
                                    state.storefront = storefront;
                                }
                                if let Ok(songs) = client.get_library_songs(100, 0).await {
                                    state.songs_offset = songs.len();
                                    state.songs_has_more = songs.len() >= 100;
                                    state.songs = songs;
                                    state.cache.insert_songs(&state.songs);
                                }
                                if let Ok(playlists) = client.get_library_playlists().await {
                                    state.playlists = playlists;
                                    state.cache.insert_playlists(&state.playlists);
                                }
                                state.set_status("Logged in successfully!");
                            }
                            Err(e) => {
                                state.set_status(format!("Login failed: {e}"));
                            }
                        }
                        let _ = enable_raw_mode();
                        let mut stdout_handle = stdout();
                        let _ = execute!(stdout_handle, EnterAlternateScreen, crossterm::cursor::Hide);
                        terminal.clear()?;
                    }
                }
            }
            status_opt = async {
                let mut rx = status_rx_arc.lock().await;
                rx.recv().await
            } => {
                if let Some(status) = status_opt {
                    state.playback = status;
                    if state.show_lyrics && state.lyrics_song_id != state.playback.current_song.as_ref().map(|s| s.id.clone()) {
                        enqueue_lyrics_for_current_song(&mut state);
                    }
                }
            }
            effect = effect_rx.recv() => {
                if let Some(effect) = effect {
                    apply_effect(&mut state, effect);
                }
            }
            _ = sigterm.recv() => {
                state.should_quit = true;
            }
            _ = sigint.recv() => {
                state.should_quit = true;
            }
            _ = sighup.recv() => {
                state.should_quit = true;
            }
        }
    }

    // Stop playback and cleanup
    let _ = playback.send_command(PlaybackCommand::Stop).await;
    drop(guard);

    Ok(())
}
