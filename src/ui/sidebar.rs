use crate::app::state::{ActiveView, AppState, FocusedPanel, SidebarItem};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focused_panel == FocusedPanel::Sidebar;
    let theme = state.theme.theme();
    let items_meta = ActiveView::sidebar_items();

    let items: Vec<ListItem> = items_meta
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == state.sidebar_index;
            match item {
                SidebarItem::Header(label) => ListItem::new(Line::from(Span::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(theme.text_muted)
                        .add_modifier(Modifier::BOLD),
                ))),
                SidebarItem::View(view) => {
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
                        Span::styled(view.display_name(), style),
                    ]))
                }
                SidebarItem::LyricsToggle => {
                    let symbol = if is_selected { " > " } else { "   " };
                    let style = if is_selected {
                        theme.selected_row_style()
                    } else {
                        Style::default().fg(theme.text_primary)
                    };
                    let toggle = if state.show_lyrics {
                        Span::styled(" [on]", Style::default().fg(theme.accent))
                    } else {
                        Span::styled(" [off]", Style::default().fg(theme.text_muted))
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            symbol,
                            Style::default()
                                .fg(theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Lyrics", style),
                        toggle,
                    ]))
                }
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
