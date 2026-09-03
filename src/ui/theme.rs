use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreset {
    #[default]
    AppleDark,
    CatppuccinMocha,
    TokyoNight,
    GruvboxDark,
    Nord,
}

impl ThemePreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AppleDark => "Apple Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::Nord => "Nord",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::AppleDark => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::TokyoNight,
            Self::TokyoNight => Self::GruvboxDark,
            Self::GruvboxDark => Self::Nord,
            Self::Nord => Self::AppleDark,
        }
    }

    pub fn theme(&self) -> Theme {
        match self {
            Self::AppleDark => Theme {
                accent: Color::Rgb(250, 45, 72),      // Apple Red
                secondary: Color::Rgb(140, 140, 240), // Soft Purple
                text_primary: Color::Rgb(240, 240, 245),
                text_muted: Color::Rgb(130, 130, 140),
                border_focused: Color::Rgb(250, 45, 72),
                border_unfocused: Color::Rgb(60, 60, 70),
                highlight_bg: Color::Rgb(40, 40, 50),
            },
            Self::CatppuccinMocha => Theme {
                accent: Color::Rgb(203, 166, 247),    // Mauve
                secondary: Color::Rgb(116, 199, 236), // Sapphire
                text_primary: Color::Rgb(205, 214, 244),
                text_muted: Color::Rgb(147, 153, 178),
                border_focused: Color::Rgb(203, 166, 247),
                border_unfocused: Color::Rgb(69, 71, 90),
                highlight_bg: Color::Rgb(49, 50, 68),
            },
            Self::TokyoNight => Theme {
                accent: Color::Rgb(125, 207, 255),    // Electric Blue
                secondary: Color::Rgb(187, 154, 247), // Magenta Purple
                text_primary: Color::Rgb(192, 202, 245),
                text_muted: Color::Rgb(86, 95, 137),
                border_focused: Color::Rgb(125, 207, 255),
                border_unfocused: Color::Rgb(41, 46, 66),
                highlight_bg: Color::Rgb(47, 56, 93),
            },
            Self::GruvboxDark => Theme {
                accent: Color::Rgb(254, 128, 25),     // Bright Orange
                secondary: Color::Rgb(142, 192, 124), // Bright Aqua
                text_primary: Color::Rgb(235, 219, 178),
                text_muted: Color::Rgb(146, 131, 116),
                border_focused: Color::Rgb(254, 128, 25),
                border_unfocused: Color::Rgb(80, 73, 69),
                highlight_bg: Color::Rgb(60, 56, 54),
            },
            Self::Nord => Theme {
                accent: Color::Rgb(136, 192, 208),    // Frost Blue
                secondary: Color::Rgb(129, 161, 193), // Glacial Blue
                text_primary: Color::Rgb(236, 239, 244),
                text_muted: Color::Rgb(118, 135, 164),
                border_focused: Color::Rgb(136, 192, 208),
                border_unfocused: Color::Rgb(59, 66, 82),
                highlight_bg: Color::Rgb(67, 76, 94),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub accent: Color,
    pub secondary: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub highlight_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        ThemePreset::AppleDark.theme()
    }
}

impl Theme {
    pub const ACCENT: Color = Color::Rgb(250, 45, 72); // Apple Music Pink/Red
    pub const SECONDARY: Color = Color::Rgb(140, 140, 240); // Soft Purple
    pub const TEXT_PRIMARY: Color = Color::Rgb(240, 240, 245);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 130, 140);
    pub const BORDER_FOCUSED: Color = Color::Rgb(250, 45, 72);
    pub const BORDER_UNFOCUSED: Color = Color::Rgb(60, 60, 70);
    pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 50);

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border_unfocused)
        }
    }

    pub fn selected_row_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}
