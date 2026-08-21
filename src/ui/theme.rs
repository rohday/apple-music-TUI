use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const ACCENT: Color = Color::Rgb(250, 45, 72); // Apple Music Pink/Red
    pub const SECONDARY: Color = Color::Rgb(140, 140, 240); // Soft Purple
    pub const TEXT_PRIMARY: Color = Color::Rgb(240, 240, 245);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 130, 140);
    pub const BORDER_FOCUSED: Color = Color::Rgb(250, 45, 72);
    pub const BORDER_UNFOCUSED: Color = Color::Rgb(60, 60, 70);
    pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 50);

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(focused: bool) -> Style {
        if focused {
            Style::default().fg(Self::BORDER_FOCUSED)
        } else {
            Style::default().fg(Self::BORDER_UNFOCUSED)
        }
    }

    pub fn selected_row_style() -> Style {
        Style::default()
            .bg(Self::HIGHLIGHT_BG)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}
