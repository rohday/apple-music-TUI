use crate::app::state::AppState;
use crate::playback::types::PlaybackState;
use crate::ui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub fn render_player_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_UNFOCUSED));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Song Info & Time
            Constraint::Length(1), // Progress Bar
            Constraint::Length(1), // Controls & Volume
        ])
        .split(inner);

    // 1. Song Info
    let (track_title, artist_album) = if let Some(song) = &state.playback.current_song {
        (
            song.name.clone(),
            format!(
                "{} • {}",
                song.artist_name,
                song.album_name.as_deref().unwrap_or("Single")
            ),
        )
    } else {
        (
            "No track playing".to_string(),
            "Select a song and press Enter to play".to_string(),
        )
    };

    let status_icon = match state.playback.state {
        PlaybackState::Playing => " [PLAY] ",
        PlaybackState::Paused => " [PAUSE] ",
        PlaybackState::Loading => " [LOAD] ",
        PlaybackState::Stopped => " [STOP] ",
    };

    let time_info = state.playback.formatted_position();

    let info_line = Line::from(vec![
        Span::styled(
            status_icon,
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", track_title),
            Style::default()
                .fg(Theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("- {} ", artist_album),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]);

    let time_line = Line::from(vec![Span::styled(
        time_info,
        Style::default().fg(Theme::TEXT_PRIMARY),
    )]);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(chunks[0]);

    f.render_widget(Paragraph::new(info_line), row1[0]);
    f.render_widget(
        Paragraph::new(time_line).alignment(Alignment::Right),
        row1[1],
    );

    // 2. Progress Gauge
    let ratio = state.playback.progress_ratio();
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Theme::ACCENT)
                .bg(Color::Rgb(40, 40, 50)),
        )
        .ratio(ratio)
        .label("");
    f.render_widget(gauge, chunks[1]);

    // 3. Controls and Volume
    let shuffle_style = if state.playback.shuffle {
        Style::default()
            .fg(Theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let repeat_style = if state.playback.repeat != crate::playback::types::RepeatMode::Off {
        Style::default()
            .fg(Theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_MUTED)
    };

    let controls_line = Line::from(vec![
        Span::raw("[p] |<<  "),
        Span::raw("[Space] >/||  "),
        Span::raw("[n] >>|   "),
        Span::styled(
            format!(
                "[s] Shuffle: {}   ",
                if state.playback.shuffle { "On" } else { "Off" }
            ),
            shuffle_style,
        ),
        Span::styled(
            format!(
                "[r] Repeat: {}   ",
                state.playback.repeat.display_label()
            ),
            repeat_style,
        ),
        Span::raw(format!("[+/-] Vol: {}%", state.playback.volume)),
    ]);

    f.render_widget(Paragraph::new(controls_line), chunks[2]);
}
