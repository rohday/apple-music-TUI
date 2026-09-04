use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

fn extract_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::White => (255, 255, 255),
        Color::Red => (255, 50, 50),
        Color::Blue => (50, 100, 255),
        Color::Green => (50, 255, 50),
        _ => (200, 200, 200),
    }
}

fn blend_rgb(c1: (u8, u8, u8), c2: (u8, u8, u8), factor: f64) -> Color {
    let f = factor.clamp(0.0, 1.0);
    let r = (c1.0 as f64 + (c2.0 as f64 - c1.0 as f64) * f).round() as u8;
    let g = (c1.1 as f64 + (c2.1 as f64 - c1.1 as f64) * f).round() as u8;
    let b = (c1.2 as f64 + (c2.2 as f64 - c1.2 as f64) * f).round() as u8;
    Color::Rgb(r, g, b)
}

/// Renders a thick 60 FPS animated traveling glow beam progress bar with smooth fade.
pub fn render_shimmer_progress_bar<'a>(
    width: usize,
    progress_ratio: f64,
    anim_time: f64,
    is_playing: bool,
    accent_color: Color,
    glow_color: Color,
    unplayed_bg: Color,
) -> Line<'a> {
    if width == 0 {
        return Line::default();
    }

    let clamped_ratio = progress_ratio.clamp(0.0, 1.0);
    let filled_len = ((width as f64) * clamped_ratio).round() as usize;
    let filled_len = filled_len.min(width);

    let mut spans = Vec::with_capacity(width);

    let accent_rgb = extract_rgb(accent_color);
    let glow_rgb = extract_rgb(glow_color);

    if filled_len == 0 {
        // Entirely unplayed thick track
        spans.push(Span::styled(
            " ".repeat(width),
            Style::default().bg(unplayed_bg),
        ));
        return Line::from(spans);
    }

    let beam_radius = 6.0; // Wide, soft, faded radius
    let pad = beam_radius + 2.0; // Padding to ensure beam fully enters and exits off-screen
    let calm_gap = 5.0; // Brief peaceful pause between sweeps
    let total_travel = (filled_len as f64) + 2.0 * pad + calm_gap;
    let sweep_period = 5.5; // Seconds per complete sweep cycle
    let beam_speed = total_travel / sweep_period;

    let beam_pos = if is_playing {
        ((anim_time * beam_speed) % total_travel) - pad
    } else {
        (filled_len as f64) * 0.5
    };

    for i in 0..filled_len {
        let dist = ((i as f64) - beam_pos).abs();
        let color = if is_playing && dist < beam_radius {
            // Smooth, soft cosine fade: calm and never overly bright or opaque
            let factor = ((std::f64::consts::PI * dist / beam_radius).cos() * 0.5 + 0.5) * 0.40;
            blend_rgb(accent_rgb, glow_rgb, factor)
        } else {
            accent_color
        };

        spans.push(Span::styled("█", Style::default().fg(color)));
    }

    // Unplayed thick track
    let unfilled_len = width.saturating_sub(filled_len);
    if unfilled_len > 0 {
        spans.push(Span::styled(
            " ".repeat(unfilled_len),
            Style::default().bg(unplayed_bg),
        ));
    }

    Line::from(spans)
}
