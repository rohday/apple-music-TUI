use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Renders a 60 FPS animated traveling glow beam progress bar.
/// `width`: total character width of the progress bar
/// `progress_ratio`: 0.0 to 1.0
/// `anim_time`: continuous high-resolution time in seconds (60 FPS)
/// `is_playing`: whether track is currently playing
/// `accent_color`: active theme accent color
/// `glow_color`: highlight color for the traveling light pulse (usually bright white or text primary)
/// `bg_color`: color for the unplayed track
pub fn render_shimmer_progress_bar<'a>(
    width: usize,
    progress_ratio: f64,
    anim_time: f64,
    is_playing: bool,
    accent_color: Color,
    glow_color: Color,
    bg_color: Color,
) -> Line<'a> {
    if width == 0 {
        return Line::default();
    }

    let clamped_ratio = progress_ratio.clamp(0.0, 1.0);
    let filled_len = ((width as f64) * clamped_ratio).round() as usize;
    let filled_len = filled_len.min(width);

    let mut spans = Vec::with_capacity(width);

    if filled_len == 0 {
        // Entirely unplayed
        spans.push(Span::styled("─".repeat(width), Style::default().fg(bg_color)));
        return Line::from(spans);
    }

    // Traveling beam calculation:
    // Speed: complete transit every 2.0 seconds
    let beam_speed = (filled_len as f64) / 1.8;
    let beam_pos = if is_playing {
        (anim_time * beam_speed) % (filled_len.max(1) as f64)
    } else {
        (filled_len as f64) * 0.5 // Stationary mid-point when paused
    };

    let beam_radius = 2.5; // Width of glow pulse

    for i in 0..filled_len {
        let is_tip = i == filled_len.saturating_sub(1);
        let dist = ((i as f64) - beam_pos).abs();

        if is_tip {
            spans.push(Span::styled(
                "╸",
                Style::default()
                    .fg(accent_color)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if dist < 1.0 {
            // Beam core: peak glow
            spans.push(Span::styled(
                "━",
                Style::default().fg(glow_color).add_modifier(Modifier::BOLD),
            ));
        } else if dist < beam_radius {
            // Beam shoulder: intermediate bright accent
            spans.push(Span::styled(
                "━",
                Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
            ));
        } else {
            // Normal played bar
            spans.push(Span::styled(
                "━",
                Style::default().fg(accent_color),
            ));
        }
    }

    // Unplayed remainder
    let unfilled_len = width.saturating_sub(filled_len);
    if unfilled_len > 0 {
        spans.push(Span::styled("─".repeat(unfilled_len), Style::default().fg(bg_color)));
    }

    Line::from(spans)
}
