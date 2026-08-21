use anyhow::Result;
use apple_tui::api::client::AppleMusicClient;
use apple_tui::app::state::AppState;
use apple_tui::auth::login::launch_interactive_login;
use apple_tui::config::{AuthConfig, Config};
use apple_tui::events::handle_key_event;
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

#[derive(Parser, Debug)]
#[command(
    name = "apple-tui",
    version = "0.1.0",
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

    let auth = AuthConfig::load();
    let is_auth = auth.is_authenticated() || config.mock_mode;

    let client = if config.mock_mode {
        AppleMusicClient::new_mock()
    } else {
        AppleMusicClient::new(auth.developer_token.clone(), auth.music_user_token.clone())
            .unwrap_or_else(|_| AppleMusicClient::new_mock())
    };

    let playback = PlaybackEngine::new(config.browser_path.clone(), config.mock_mode).await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let _guard = TerminalGuard;

    // Set panic hook for safety
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        eprintln!("Panic occurred: {:?}", info);
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

    // Preload initial library songs & playlists
    if let Ok(songs) = client.get_library_songs(50, 0).await {
        state.songs = songs;
    }
    if let Ok(playlists) = client.get_library_playlists().await {
        state.playlists = playlists;
    }

    let mut event_stream = EventStream::new();
    let status_rx_arc = playback.get_status_receiver();
    let mut ticker = tokio::time::interval(Duration::from_millis(config.tick_rate_ms));

    while !state.should_quit {
        terminal.draw(|f| {
            draw(f, &state);
        })?;

        tokio::select! {
            _ = ticker.tick() => {
                state.playback = playback.get_current_status().await;
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    handle_key_event(key, &mut state, &client, &playback).await?;
                }
            }
            status_opt = async {
                let mut rx = status_rx_arc.lock().await;
                rx.recv().await
            } => {
                if let Some(status) = status_opt {
                    state.playback = status;
                }
            }
        }
    }

    // Stop playback and cleanup
    let _ = playback.send_command(PlaybackCommand::Stop).await;
    drop(_guard);

    Ok(())
}
