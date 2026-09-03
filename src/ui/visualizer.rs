use crate::app::state::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

const UNICODE_BARS: [char; 9] = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn compute_band_heights(time_secs: f64, num_bars: usize, is_playing: bool) -> Vec<f64> {
    if !is_playing {
        return vec![0.05; num_bars];
    }

    (0..num_bars)
        .map(|i| {
            let fi = i as f64;
            // Multi-harmonic oscillating wave
            let w1 = ((time_secs * 4.5 + fi * 0.9).sin() + 1.0) * 0.35;
            let w2 = ((time_secs * 7.2 - fi * 1.3).cos() + 1.0) * 0.25;
            let w3 = ((time_secs * 11.0 + fi * 2.1).sin() + 1.0) * 0.2;
            (w1 + w2 + w3 + 0.1).clamp(0.0, 1.0)
        })
        .collect()
}

pub fn compute_spectrum_bars(time_secs: f64, num_bars: usize, is_playing: bool) -> String {
    let heights = compute_band_heights(time_secs, num_bars, is_playing);
    heights
        .into_iter()
        .map(|h| {
            let idx = (h * 8.0).round() as usize;
            UNICODE_BARS[idx.min(8)]
        })
        .collect()
}

pub fn render_fullscreen_visualizer(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();
    let is_playing = state.playback.state == crate::playback::types::PlaybackState::Playing;
    let time = state.playback.current_time_secs;

    let block = Block::default()
        .title(Span::styled(
            " Spectrum Visualizer [v: Close] ",
            theme.title_style(),
        ))
        .borders(Borders::ALL)
        .border_style(theme.border_style(true));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 4 || inner.width < 10 {
        return;
    }

    let num_bars = (inner.width as usize).saturating_sub(4).min(64);
    let heights = compute_band_heights(time, num_bars, is_playing);

    let max_h = (inner.height as usize).saturating_sub(2);
    let mut lines: Vec<Line> = Vec::with_capacity(max_h + 1);

    for row in (0..max_h).rev() {
        let threshold = (row as f64) / (max_h as f64);
        let mut spans = Vec::new();
        spans.push(Span::raw("  "));

        for &h in &heights {
            if h >= threshold {
                let color = if (row as f64) > (max_h as f64) * 0.75 {
                    theme.accent
                } else if (row as f64) > (max_h as f64) * 0.4 {
                    theme.secondary
                } else {
                    theme.text_primary
                };
                spans.push(Span::styled("█", Style::default().fg(color)));
            } else if h >= threshold - (0.5 / max_h as f64) {
                spans.push(Span::styled("▄", Style::default().fg(theme.text_muted)));
            } else {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    let bottom_label = if let Some(song) = &state.playback.current_song {
        format!(
            "♫ {} - {} [{}]",
            song.name,
            song.artist_name,
            state.playback.formatted_position()
        )
    } else {
        "No song playing".to_string()
    };
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            bottom_label,
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
