pub mod main_view;
pub mod modals;
pub mod player_bar;
pub mod sidebar;
pub mod theme;

use crate::app::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(f: &mut Frame, state: &AppState) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header Status Bar
            Constraint::Min(10),   // Content Area (Sidebar + Main)
            Constraint::Length(5), // Bottom Player Bar
        ])
        .split(size);

    // 1. Header Bar
    render_header(f, main_chunks[0], state);

    // 2. Main Content Split
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22), // Sidebar
            Constraint::Percentage(78), // Main Table
        ])
        .split(main_chunks[1]);

    sidebar::render_sidebar(f, content_chunks[0], state);
    main_view::render_main_view(f, content_chunks[1], state);

    // 3. Bottom Player Bar
    player_bar::render_player_bar(f, main_chunks[2], state);

    // 4. Modals / Popups
    modals::render_modals(f, size, state);
}

fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.theme.theme();
    let auth_badge = if state.is_authenticated {
        Span::styled(
            format!(" [{}]", state.storefront.to_uppercase()),
            Style::default().fg(theme.text_muted),
        )
    } else {
        Span::styled(
            " [offline]",
            Style::default().fg(Color::Yellow),
        )
    };

    let status_text = state
        .status_message
        .as_deref()
        .unwrap_or("Press '?' Help | '/' Search | 't' Theme | 'q' Quit");

    let left_header = Line::from(vec![
        Span::styled(
            " AppleTUI",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        auth_badge,
    ]);

    let right_header = Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(theme.text_muted),
    )]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    f.render_widget(Paragraph::new(left_header), chunks[0]);
    f.render_widget(
        Paragraph::new(right_header).alignment(ratatui::layout::Alignment::Right),
        chunks[1],
    );
}
