use crate::app::state::AppState;
use crate::playback::types::PlaybackState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_player_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();
    let border_color = if state.show_visualizer {
        theme.accent
    } else {
        theme.border_unfocused
    };

    let title_badge = if state.show_visualizer {
        " Visualizer [v] "
    } else {
        ""
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    if !title_badge.is_empty() {
        block = block.title(Span::styled(title_badge, theme.title_style()));
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    // Cover art column: shown whenever art is loaded (or loading) for the
    // current track. Decoding/resizing happens per frame from the cached
    // RgbImage, which is cheap for 300x300 covers.
    let art_width = state
        .playback
        .current_song
        .as_ref()
        .and_then(|song| state.artwork.get(&song.id))
        .and_then(|img| {
            crate::ui::art::to_half_block_cells_from_image(img, 10, inner.height as usize)
        })
        .map(|cells| cells.first().map(|r| r.len()).unwrap_or(0))
        .unwrap_or(0);

    let rest_area = if art_width > 0 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(art_width as u16 + 1),
                Constraint::Min(10),
            ])
            .split(inner);
        if let Some(song) = &state.playback.current_song {
            if let Some(img) = state.artwork.get(&song.id) {
                if let Some(cells) =
                    crate::ui::art::to_half_block_cells_from_image(img, 10, inner.height as usize)
                {
                    let lines = crate::ui::art::art_lines(&cells);
                    f.render_widget(Paragraph::new(lines), chunks[0]);
                }
            }
        }
        chunks[1]
    } else {
        inner
    };

    let is_expanded = state.show_visualizer && rest_area.height >= 5;

    let (track_chunk, ribbon_chunk, gauge_chunk, controls_chunk) = if is_expanded {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Track Info & Time
                Constraint::Length(2), // 2-line Braille Ribbon Wave
                Constraint::Length(1), // Progress Bar
                Constraint::Length(1), // Controls
            ])
            .split(rest_area);
        (chunks[0], Some(chunks[1]), chunks[2], chunks[3])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Track Info & Time
                Constraint::Length(1), // Progress Bar
                Constraint::Length(1), // Controls
            ])
            .split(rest_area);
        (chunks[0], None, chunks[1], chunks[2])
    };

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
    let is_playing = state.playback.state == PlaybackState::Playing;

    let info_spans = vec![
        Span::styled(
            status_icon,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", track_title),
            Style::default()
                .fg(theme.text_primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("- {} ", artist_album),
            Style::default().fg(theme.text_muted),
        ),
    ];

    let info_line = Line::from(info_spans);
    let time_line = Line::from(vec![Span::styled(
        time_info,
        Style::default().fg(theme.text_primary),
    )]);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(track_chunk);

    f.render_widget(Paragraph::new(info_line), row1[0]);
    f.render_widget(
        Paragraph::new(time_line).alignment(Alignment::Right),
        row1[1],
    );

    // 2. Expanded Braille Fluid Ribbon (Single Theme Color)
    if let Some(r_area) = ribbon_chunk {
        let (top_ribbon, bottom_ribbon) = crate::ui::visualizer::render_braille_ribbon(
            r_area.width as usize,
            state.anim_time,
            is_playing,
        );
        let ribbon_lines = vec![
            Line::from(Span::styled(top_ribbon, Style::default().fg(theme.accent))),
            Line::from(Span::styled(
                bottom_ribbon,
                Style::default().fg(theme.accent),
            )),
        ];
        f.render_widget(Paragraph::new(ribbon_lines), r_area);
    }

    // 3. Shimmer Progress Bar Visualizer (60 FPS Traveling Glow Beam with Smooth Fade)
    let ratio = state.playback.progress_ratio();
    let glow_color = if is_playing {
        ratatui::style::Color::White
    } else {
        theme.accent
    };
    let progress_line = crate::ui::shimmer::render_shimmer_progress_bar(
        gauge_chunk.width as usize,
        ratio,
        state.anim_time,
        is_playing,
        theme.accent,
        glow_color,
        theme.highlight_bg,
    );
    f.render_widget(Paragraph::new(progress_line), gauge_chunk);

    // 4. Clean Minimal Controls (no verbose text labels)
    let shuffle_style = if state.playback.shuffle {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let repeat_style = if state.playback.repeat != crate::playback::types::RepeatMode::Off {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let repeat_text = match state.playback.repeat {
        crate::playback::types::RepeatMode::One => "[r]¹",
        _ => "[r]",
    };

    let viz_style = if state.show_visualizer {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let controls_line = Line::from(vec![
        Span::raw("[p]⏮  [Space]▶⏸  [n]⏭    "),
        Span::styled("[s]  ", shuffle_style),
        Span::styled(format!("{}  ", repeat_text), repeat_style),
        Span::styled("[v]    ", viz_style),
        Span::styled(
            format!("[+/-] {}%", state.playback.volume),
            Style::default().fg(theme.text_muted),
        ),
    ]);

    f.render_widget(Paragraph::new(controls_line), controls_chunk);
}
