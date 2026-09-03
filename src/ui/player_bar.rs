use crate::app::state::AppState;
use crate::playback::types::PlaybackState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub fn render_player_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();
    let border_color = if state.show_visualizer {
        theme.accent
    } else {
        theme.border_unfocused
    };

    let title_badge = if state.show_visualizer {
        " Visualizer [v: Compact] "
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

    let is_expanded = state.show_visualizer && inner.height >= 5;

    let (track_chunk, ribbon_chunk, gauge_chunk, controls_chunk) = if is_expanded {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Track Info & Time
                Constraint::Length(2), // 2-line Braille Ribbon Wave
                Constraint::Length(1), // Progress Bar
                Constraint::Length(1), // Controls
            ])
            .split(inner);
        (chunks[0], Some(chunks[1]), chunks[2], chunks[3])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Track Info & Time
                Constraint::Length(1), // Progress Bar
                Constraint::Length(1), // Controls
            ])
            .split(inner);
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

    let mut info_spans = vec![Span::styled(
        status_icon,
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )];

    if !is_expanded {
        let inline_wave = crate::ui::visualizer::render_compact_braille_wave(
            8,
            state.anim_time,
            is_playing,
        );
        info_spans.push(Span::styled(
            format!("{} ", inline_wave),
            Style::default().fg(theme.secondary),
        ));
    }

    info_spans.push(Span::styled(
        format!("{} ", track_title),
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
    info_spans.push(Span::styled(
        format!("- {} ", artist_album),
        Style::default().fg(theme.text_muted),
    ));

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

    // 2. Expanded Braille Fluid Ribbon
    if let Some(r_area) = ribbon_chunk {
        let (top_ribbon, bottom_ribbon) = crate::ui::visualizer::render_braille_ribbon(
            r_area.width as usize,
            state.anim_time,
            is_playing,
        );
        let ribbon_lines = vec![
            Line::from(Span::styled(top_ribbon, Style::default().fg(theme.secondary))),
            Line::from(Span::styled(bottom_ribbon, Style::default().fg(theme.accent))),
        ];
        f.render_widget(Paragraph::new(ribbon_lines), r_area);
    }

    // 3. Progress Gauge
    let ratio = state.playback.progress_ratio();
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(theme.accent)
                .bg(theme.highlight_bg),
        )
        .ratio(ratio)
        .label("");
    f.render_widget(gauge, gauge_chunk);

    // 4. Controls and Volume
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

    let viz_label = if state.show_visualizer {
        "Expanded"
    } else {
        "Compact"
    };

    let controls_line = if area.width < 90 {
        Line::from(vec![
            Span::raw("[p]⏮ [Space]▶⏸ [n]⏭  "),
            Span::styled(
                format!(
                    "[s]Shuf:{} ",
                    if state.playback.shuffle { "On" } else { "Off" }
                ),
                shuffle_style,
            ),
            Span::styled(
                format!("[r]Rep:{} ", state.playback.repeat.display_label()),
                repeat_style,
            ),
            Span::styled(
                format!("[v]Wave:{} ", viz_label),
                Style::default().fg(theme.secondary),
            ),
            Span::raw(format!("[+/-]Vol:{}%", state.playback.volume)),
        ])
    } else {
        Line::from(vec![
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
                format!("[r] Repeat: {}   ", state.playback.repeat.display_label()),
                repeat_style,
            ),
            Span::styled(
                format!("[v] Wave Ribbon: {}   ", viz_label),
                Style::default().fg(theme.secondary),
            ),
            Span::raw(format!("[+/-] Vol: {}%", state.playback.volume)),
        ])
    };

    f.render_widget(Paragraph::new(controls_line), controls_chunk);
}
