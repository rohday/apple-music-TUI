use crate::app::state::{ActiveView, AppState, FocusedPanel};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render_main_view(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::MainContent;
    let title = state.active_view.display_name();

    let block = Block::default()
        .title(Span::styled(format!(" {} ", title), Theme::title_style()))
        .borders(Borders::ALL)
        .border_style(Theme::border_style(focused));

    match state.active_view {
        ActiveView::LibrarySongs
        | ActiveView::PlaylistDetail
        | ActiveView::RecentlyPlayed
        | ActiveView::Search
        | ActiveView::Queue => {
            let songs = match state.active_view {
                ActiveView::LibrarySongs => &state.songs,
                ActiveView::PlaylistDetail => &state.playlist_tracks,
                ActiveView::RecentlyPlayed => &state.recent_tracks,
                ActiveView::Search => &state.search_results.songs,
                ActiveView::Queue => &state.queue,
                _ => &state.songs,
            };

            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Title").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Album").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Duration ").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = songs
                .iter()
                .enumerate()
                .map(|(idx, song)| {
                    let is_selected = idx == state.selected_index;
                    let is_playing = state
                        .playback
                        .current_song
                        .as_ref()
                        .map(|s| s.id == song.id)
                        .unwrap_or(false);

                    let num_prefix = if is_playing {
                        ">".to_string()
                    } else {
                        format!("{:>2}", idx + 1)
                    };

                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else if is_playing {
                        Style::default().fg(Theme::ACCENT)
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
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
            f.render_widget(table, area);
        }
        ActiveView::Playlists => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Playlist Name").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Tracks").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Description").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .playlists
                .iter()
                .enumerate()
                .map(|(idx, pl)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
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
            f.render_widget(table, area);
        }
        ActiveView::LibraryAlbums => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Album Name").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Tracks").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .albums
                .iter()
                .enumerate()
                .map(|(idx, alb)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
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
            f.render_widget(table, area);
        }
        ActiveView::LibraryArtists => {
            let header = Row::new(vec![
                Cell::from(" # ").style(Style::default().fg(Theme::TEXT_MUTED)),
                Cell::from(" Artist Name").style(Style::default().fg(Theme::TEXT_MUTED)),
            ])
            .bottom_margin(1);

            let rows: Vec<Row> = state
                .artists
                .iter()
                .enumerate()
                .map(|(idx, art)| {
                    let is_selected = idx == state.selected_index;
                    let row_style = if is_selected {
                        Theme::selected_row_style()
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };

                    Row::new(vec![
                        Cell::from(format!(" {:>2} ", idx + 1)),
                        Cell::from(art.name.clone()),
                    ])
                    .style(row_style)
                })
                .collect();

            let widths = [Constraint::Length(5), Constraint::Percentage(90)];

            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
    }
}
