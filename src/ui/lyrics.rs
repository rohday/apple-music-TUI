use crate::app::state::AppState;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_lyrics_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();
    let block = Block::default()
        .title(Span::styled(" Lyrics [y: Close] ", theme.title_style()))
        .borders(Borders::ALL)
        .border_style(theme.border_style(false));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 || inner.width < 5 {
        return;
    }

    if state.lyrics_loading {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Loading lyrics...",
                Style::default().fg(theme.text_muted),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(p, inner);
        return;
    }

    let lyrics_data = match &state.lyrics {
        Some(data) if !data.lines.is_empty() => data,
        _ => {
            let p = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No lyrics available for this track",
                    Style::default().fg(theme.text_muted),
                )),
            ])
            .alignment(Alignment::Center);
            f.render_widget(p, inner);
            return;
        }
    };

    let viewport_height = inner.height as usize;
    let current_time = state.playback.current_time_secs;
    let active_idx = lyrics_data.current_line_idx(current_time);

    // Center active line in viewport
    let half_h = viewport_height / 2;
    let center_idx = active_idx.unwrap_or(0);
    let start_idx = center_idx.saturating_sub(half_h);
    let end_idx = (start_idx + viewport_height).min(lyrics_data.lines.len());
    let visible_lines = &lyrics_data.lines[start_idx..end_idx];

    let lines: Vec<Line> = visible_lines
        .iter()
        .enumerate()
        .map(|(offset, line)| {
            let true_idx = start_idx + offset;
            let is_active = Some(true_idx) == active_idx;

            let style = if is_active {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if Some(true_idx) < active_idx {
                Style::default().fg(theme.text_muted)
            } else {
                Style::default().fg(theme.text_primary)
            };

            let prefix = if is_active { "▶ " } else { "  " };
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                Span::styled(&line.text, style),
            ])
        })
        .collect();

    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
}
