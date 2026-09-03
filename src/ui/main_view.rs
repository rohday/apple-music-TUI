use crate::app::state::{ActiveView, AppState, FocusedPanel};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn calculate_viewport_range(
    selected_index: usize,
    total_items: usize,
    viewport_height: usize,
) -> (usize, usize) {
    if total_items == 0 || viewport_height == 0 {
        return (0, 0);
    }
    let half_window = viewport_height / 2;
    let start = selected_index.saturating_sub(half_window);
    let start = if start + viewport_height > total_items {
        total_items.saturating_sub(viewport_height)
    } else {
        start
    };
    let end = (start + viewport_height).min(total_items);
    (start, end)
}

pub fn render_main_view(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::MainContent;
    let title = state.active_view.display_name();
    let theme = state.theme.theme();

    let (table_area, filter_area) = if state.is_filtering {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(3)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let viewport_height = (table_area.height.saturating_sub(4) as usize).max(1);

    if let Some(f_area) = filter_area {
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(Span::styled(" Filter View ", theme.title_style()));
        let filter_line = Line::from(vec![
            Span::styled(" Query: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{}_", state.filter_query),
                Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [Enter: Apply | Esc: Clear]", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(filter_line).block(filter_block), f_area);
    }

    match state.active_view {
        ActiveView::LibrarySongs
        | ActiveView::PlaylistDetail
        | ActiveView::RecentlyPlayed
        | ActiveView::Search
        | ActiveView::Queue => {
            let songs = state.filtered_songs();

            let (start_idx, end_idx) =
                calculate_viewport_range(state.selected_index, songs.len(), viewport_height);
            let visible_songs = if songs.is_empty() {
                &[]
            } else {
                &songs[start_idx..end_idx]
            };

            let filter_badge = if !state.filter_query.is_empty() {
                format!(" [Filter: \"{}\"]", state.filter_query)
            } else {
                String::new()
            };

            let title_text = if songs.len() > viewport_height {
                format!(
                    " {}{} [{}-{}/{}] ",
                    title,
                    filter_badge,
                    start_idx + 1,
                    end_idx,
                    songs.len()
                )
            } else {
                format!(" {}{} ", title, filter_badge)
            };

            let block = Block::default()
                .title(Span::styled(title_text, theme.title_style()))
                .borders(Borders::ALL)
                .border_style(theme.border_style(focused));

            if state.active_view == ActiveView::Search && songs.is_empty() {
                let empty_lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "🔍 Apple Music Catalog Search",
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press '/' or Enter to search millions of songs, albums, and artists.",
                        Style::default().fg(theme.text_primary),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Example queries: \"The Weeknd\", \"Daft Punk\", \"Starboy\"",
                        Style::default().fg(theme.text_muted),
                    )),
                ];
                let paragraph = Paragraph::new(empty_lines)
                    .block(block)
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(paragraph, table_area);
                return;
            }

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Title").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Artist").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Album").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Duration ").style(Style::default().fg(theme.text_muted)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = visible_songs
                .iter()
                .enumerate()
                .map(|(offset, song)| {
                    let true_idx = start_idx + offset;
                    let is_selected = true_idx == state.selected_index;
                    let is_playing = state
                        .playback
                        .current_song
                        .as_ref()
                        .map(|s| s.id == song.id)
                        .unwrap_or(false);

                    let num_prefix = if is_playing {
                        ">".to_string()
                    } else {
                        format!("{:>2}", true_idx + 1)
                    };

                    let row_style = if is_selected {
                        theme.selected_row_style()
                    } else if is_playing {
                        Style::default().fg(theme.accent)
                    } else {
                        Style::default().fg(theme.text_primary)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {} ", num_prefix)),
                        Cell::from(song.name.clone()),
                        Cell::from(song.artist_name.clone()),
                        Cell::from(song.album_name.clone().unwrap_or_default()),
                        Cell::from(format!(" {} ", song.formatted_duration())),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Length(10),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, table_area);
        }
        ActiveView::Playlists => {
            let (start_idx, end_idx) = calculate_viewport_range(
                state.selected_index,
                state.playlists.len(),
                viewport_height,
            );
            let visible_playlists = if state.playlists.is_empty() {
                &[]
            } else {
                &state.playlists[start_idx..end_idx]
            };

            let title_text = if state.playlists.len() > viewport_height {
                format!(
                    " {} [{}-{}/{}] ",
                    title,
                    start_idx + 1,
                    end_idx,
                    state.playlists.len()
                )
            } else {
                format!(" {} ", title)
            };

            let block = Block::default()
                .title(Span::styled(title_text, theme.title_style()))
                .borders(Borders::ALL)
                .border_style(theme.border_style(focused));

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Playlist Name").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Tracks").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Description").style(Style::default().fg(theme.text_muted)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = visible_playlists
                .iter()
                .enumerate()
                .map(|(offset, pl)| {
                    let true_idx = start_idx + offset;
                    let is_selected = true_idx == state.selected_index;
                    let row_style = if is_selected {
                        theme.selected_row_style()
                    } else {
                        Style::default().fg(theme.text_primary)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", true_idx + 1)),
                        Cell::from(pl.name.clone()),
                        Cell::from(
                            pl.track_count
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        Cell::from(pl.description.clone().unwrap_or_default()),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Percentage(50),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, table_area);
        }
        ActiveView::LibraryAlbums => {
            let (start_idx, end_idx) =
                calculate_viewport_range(state.selected_index, state.albums.len(), viewport_height);
            let visible_albums = if state.albums.is_empty() {
                &[]
            } else {
                &state.albums[start_idx..end_idx]
            };

            let title_text = if state.albums.len() > viewport_height {
                format!(
                    " {} [{}-{}/{}] ",
                    title,
                    start_idx + 1,
                    end_idx,
                    state.albums.len()
                )
            } else {
                format!(" {} ", title)
            };

            let block = Block::default()
                .title(Span::styled(title_text, theme.title_style()))
                .borders(Borders::ALL)
                .border_style(theme.border_style(focused));

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Album Name").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Artist").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Tracks").style(Style::default().fg(theme.text_muted)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = visible_albums
                .iter()
                .enumerate()
                .map(|(offset, alb)| {
                    let true_idx = start_idx + offset;
                    let is_selected = true_idx == state.selected_index;
                    let row_style = if is_selected {
                        theme.selected_row_style()
                    } else {
                        Style::default().fg(theme.text_primary)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", true_idx + 1)),
                        Cell::from(alb.name.clone()),
                        Cell::from(alb.artist_name.clone()),
                        Cell::from(
                            alb.track_count
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(45),
                Constraint::Percentage(35),
                Constraint::Length(10),
            ];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, table_area);
        }
        ActiveView::LibraryArtists => {
            let (start_idx, end_idx) = calculate_viewport_range(
                state.selected_index,
                state.artists.len(),
                viewport_height,
            );
            let visible_artists = if state.artists.is_empty() {
                &[]
            } else {
                &state.artists[start_idx..end_idx]
            };

            let title_text = if state.artists.len() > viewport_height {
                format!(
                    " {} [{}-{}/{}] ",
                    title,
                    start_idx + 1,
                    end_idx,
                    state.artists.len()
                )
            } else {
                format!(" {} ", title)
            };

            let block = Block::default()
                .title(Span::styled(title_text, theme.title_style()))
                .borders(Borders::ALL)
                .border_style(theme.border_style(focused));

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(theme.text_muted)),
                Cell::from(" Artist Name").style(Style::default().fg(theme.text_muted)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = visible_artists
                .iter()
                .enumerate()
                .map(|(offset, art)| {
                    let true_idx = start_idx + offset;
                    let is_selected = true_idx == state.selected_index;
                    let row_style = if is_selected {
                        theme.selected_row_style()
                    } else {
                        Style::default().fg(theme.text_primary)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", true_idx + 1)),
                        Cell::from(art.name.clone()),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [Constraint::Length(5), Constraint::Percentage(90)];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, table_area);
        }
    }
}
