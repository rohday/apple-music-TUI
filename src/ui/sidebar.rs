use crate::app::state::{AppState, FocusedPanel};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

/// Sidebar entries: `None` marks a section header, `Some(i)` the index into
/// `ActiveView::all_sidebar_views()`.
fn sidebar_entries() -> Vec<(Option<usize>, &'static str)> {
    vec![
        (Some(0), "Search"),
        (None, "Library"),
        (Some(1), "Songs"),
        (Some(2), "Albums"),
        (Some(3), "Artists"),
        (Some(4), "Playlists"),
        (Some(5), "Recently Played"),
        (None, "Playback"),
        (Some(6), "Queue"),
    ]
}

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::Sidebar;
    let theme = state.theme.theme();

    let items: Vec<ListItem> = sidebar_entries()
        .into_iter()
        .map(|(view_idx, label)| match view_idx {
            None => ListItem::new(Line::from(Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(theme.text_muted)
                    .add_modifier(Modifier::BOLD),
            ))),
            Some(idx) => {
                let is_selected = idx == state.sidebar_index;
                let symbol = if is_selected { " > " } else { "   " };
                let style = if is_selected {
                    theme.selected_row_style()
                } else {
                    Style::default().fg(theme.text_primary)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        symbol,
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(label, style),
                ]))
            }
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(" Library ", theme.title_style()))
        .borders(Borders::ALL)
        .border_style(theme.border_style(focused));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
