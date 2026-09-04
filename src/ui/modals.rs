use crate::app::state::{AppState, ModalState};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_modals(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();

    if state.show_now_playing {
        if let Some(song) = &state.playback.current_song {
            render_now_playing(f, area, state, song);
        }
    }

    match &state.modal {
        ModalState::None => {}
        ModalState::Search => {
            let popup = centered_rect(60, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Search Apple Music ", theme.title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let input_text = format!(" Query: {}_", state.text_input_buffer);
            let instructions = "\n Press [Enter] to Search, [Esc] to Cancel";

            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(
                    input_text,
                    Style::default().fg(theme.text_primary),
                )),
                Line::from(Span::styled(
                    instructions,
                    Style::default().fg(theme.text_muted),
                )),
            ])
            .block(block);

            f.render_widget(paragraph, popup);
        }
        ModalState::CreatePlaylist => {
            let popup = centered_rect(50, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Create New Playlist ", theme.title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let input_text = format!(" Name: {}_", state.text_input_buffer);
            let instructions = "\n Press [Enter] to Confirm, [Esc] to Cancel";

            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(
                    input_text,
                    Style::default().fg(theme.text_primary),
                )),
                Line::from(Span::styled(
                    instructions,
                    Style::default().fg(theme.text_muted),
                )),
            ])
            .block(block);

            f.render_widget(paragraph, popup);
        }
        ModalState::AddToPlaylist { song } => {
            let popup = centered_rect(50, 40, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(
                    format!(" Add '{}' to Playlist ", song.name),
                    theme.title_style(),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let viewport_height = (popup.height.saturating_sub(2) as usize).max(1);
            let (start_idx, end_idx) = crate::ui::main_view::calculate_viewport_range(
                state.add_to_playlist_index,
                state.playlists.len(),
                viewport_height,
            );
            let visible_playlists = if state.playlists.is_empty() {
                &[]
            } else {
                &state.playlists[start_idx..end_idx]
            };

            let items: Vec<ListItem> = visible_playlists
                .iter()
                .enumerate()
                .map(|(offset, pl)| {
                    let true_idx = start_idx + offset;
                    let is_sel = true_idx == state.add_to_playlist_index;
                    let style = if is_sel {
                        theme.selected_row_style()
                    } else {
                        Style::default().fg(theme.text_primary)
                    };
                    ListItem::new(format!("  {} {}", if is_sel { ">" } else { " " }, pl.name))
                        .style(style)
                })
                .collect();

            let list = List::new(items).block(block);
            f.render_widget(list, popup);
        }
        ModalState::Help => {
            let popup = centered_rect(75, 80, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(
                    " Keyboard Shortcuts & Help ",
                    theme.title_style(),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary));

            let help_text = vec![
                Line::from(Span::styled(
                    "Navigation:",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  ↑ / k, ↓ / j     : Move selection up / down"),
                Line::from("  ← / h, → / l     : Focus Sidebar / Main View"),
                Line::from("  Tab              : Toggle panel focus"),
                Line::from("  Enter            : Play song / Open playlist"),
                Line::from("  Esc              : Close popup / Return"),
                Line::from(""),
                Line::from(Span::styled(
                    "Playback:",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  Space            : Toggle Play / Pause"),
                Line::from("  n / p            : Next / Previous track"),
                Line::from("  [ / ]            : Seek -10s / +10s"),
                Line::from("  + / -            : Volume Up / Down"),
                Line::from("  s / r            : Toggle Shuffle / Cycle Repeat"),
                Line::from(""),
                Line::from(Span::styled(
                    "Features & Actions:",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  t                : Cycle themes (Apple Dark, Catppuccin, Tokyo Night, Gruvbox, Nord)"),
                Line::from("  v                : Toggle Visualizer"),
                Line::from("  f                : In-View live search filter"),
                Line::from("  R                : Start Radio Station for selected song"),
                Line::from("  y                : Toggle side-by-side synced lyrics panel"),
                Line::from("  o                : Now Playing popup (cover art + progress)"),
                Line::from("  /                : Open Catalog Search"),
                Line::from("  c                : Create new playlist (in Playlists view)"),
                Line::from("  a                : Add selected track to playlist"),
                Line::from("  A                : Add selected track to queue"),
                Line::from("  d / < / >        : Queue view - remove / move up / move down"),
                Line::from("  F5               : Refresh library data (bypasses cache)"),
                Line::from("  ?                : Toggle this help overlay"),
                Line::from("  q / Ctrl+C       : Quit application"),
            ];

            let paragraph = Paragraph::new(help_text)
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, popup);
        }
        ModalState::Notification(msg) => {
            let popup = centered_rect(50, 20, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(" Notification ", theme.title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let paragraph = Paragraph::new(format!("\n {}\n\n Press [Esc] to dismiss", msg))
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, popup);
        }
        ModalState::AuthPrompt => {
            let popup = centered_rect(60, 25, area);
            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Span::styled(
                    " Apple Music Login Required ",
                    theme.title_style(),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let text = vec![
                Line::from(Span::styled(
                    "You are not logged in to Apple Music.",
                    Style::default().fg(theme.text_primary),
                )),
                Line::from(""),
                Line::from("To access your library and stream full tracks:"),
                Line::from("  1. Press [L] to launch browser login window"),
                Line::from("  2. Or run: apple-tui --set-user-token <TOKEN>"),
                Line::from(""),
                Line::from(Span::styled(
                    "Press [Esc] to continue in preview / mock mode.",
                    Style::default().fg(theme.text_muted),
                )),
            ];

            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, popup);
        }
    }
}

/// Full "Now Playing" popup: large cover art, track info, progress. Toggled
/// with `o`, closed with `Esc` or `o`.
fn render_now_playing(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    song: &crate::api::models::Song,
) {
    let theme = state.theme.theme();
    let popup = centered_rect(60, 80, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Now Playing [o: Close] ",
            theme.title_style(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    if inner.height < 8 || inner.width < 12 {
        return;
    }

    let info_rows = 3; // title, artist/album, progress+time
    let art_budget_h = (inner.height as usize).saturating_sub(info_rows).max(1);
    let art_budget_w = inner.width as usize;

    let art_lines = state
        .artwork
        .get(&song.id)
        .and_then(|img| {
            crate::ui::art::to_half_block_cells_from_image(img, art_budget_w, art_budget_h)
        })
        .map(|cells| crate::ui::art::art_lines(&cells))
        .unwrap_or_else(|| {
            crate::ui::art::glyph_thumbnail_lines(
                art_budget_w.min(30),
                art_budget_h.min(6),
                theme.accent,
                theme.highlight_bg,
            )
        });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(art_lines.len() as u16),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let art_area = Rect {
        height: art_lines.len() as u16,
        ..chunks[0]
    };
    let paragraph = Paragraph::new(art_lines).alignment(Alignment::Center);
    f.render_widget(paragraph, art_area);

    let title = Line::from(Span::styled(
        song.name.clone(),
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
    let subtitle = Line::from(Span::styled(
        format!(
            "{} • {}",
            song.artist_name,
            song.album_name.as_deref().unwrap_or("Single")
        ),
        Style::default().fg(theme.text_muted),
    ));

    let progress = state.playback.progress_ratio();
    let bar_width = (inner.width as usize).saturating_sub(2).max(1);
    let filled = ((bar_width as f64) * progress).round() as usize;
    let bar = Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(theme.accent)),
        Span::styled(
            "─".repeat(bar_width.saturating_sub(filled)),
            Style::default().fg(theme.border_unfocused),
        ),
    ]);

    f.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(subtitle).alignment(Alignment::Center),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(format!(
            "{}  [{}]",
            bar,
            state.playback.formatted_position()
        ))
        .alignment(Alignment::Center),
        chunks[3],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
