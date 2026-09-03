use crate::app::state::{ActiveView, AppState, FocusedPanel};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::Sidebar;
    let views = ActiveView::all_sidebar_views();
    let theme = state.theme.theme();

    let items: Vec<ListItem> = views
        .iter()
        .enumerate()
        .map(|(idx, view)| {
            let is_selected = idx == state.sidebar_index;
            let symbol = if is_selected { " > " } else { "   " };
            let style = if is_selected {
                theme.selected_row_style()
            } else {
                ratatui::style::Style::default().fg(theme.text_primary)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    symbol,
                    ratatui::style::Style::default()
                        .fg(theme.accent)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::styled(view.display_name(), style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(" Library ", theme.title_style()))
        .borders(Borders::ALL)
        .border_style(theme.border_style(focused));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
