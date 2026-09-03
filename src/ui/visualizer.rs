/// Encodes dot levels (0..=4) into a single Braille character (U+2800..U+28FF).
/// Level 0 uses baseline dots (dot 7 and dot 8) for a peaceful resting baseline line: ⠤.
pub fn braille_cell_from_levels(left_level: usize, right_level: usize) -> char {
    let left_bits = match left_level {
        1 => 0x40,
        2 => 0x44,
        3 => 0x46,
        4.. => 0x47,
        _ => 0x40, // baseline dot 7
    };
    let right_bits = match right_level {
        1 => 0x80,
        2 => 0xA0,
        3 => 0xB0,
        4.. => 0xB8,
        _ => 0x80, // baseline dot 8
    };

    let code = 0x2800 | left_bits | right_bits;
    std::char::from_u32(code).unwrap_or('⠤')
}

/// Encodes dot levels (0..=4) for upper rows where 0 means empty space (U+2800: ⠀).
pub fn braille_cell_upper_from_levels(left_level: usize, right_level: usize) -> char {
    let left_bits = match left_level {
        1 => 0x40,
        2 => 0x44,
        3 => 0x46,
        4.. => 0x47,
        _ => 0x00,
    };
    let right_bits = match right_level {
        1 => 0x80,
        2 => 0xA0,
        3 => 0xB0,
        4.. => 0xB8,
        _ => 0x00,
    };

    let code = 0x2800 | left_bits | right_bits;
    std::char::from_u32(code).unwrap_or('⠀')
}

/// Generates a 2-line smooth Braille waveform ribbon across `width` terminal characters.
/// Each character column contains 2 wave sub-columns (left and right).
pub fn render_braille_ribbon(width: usize, time_secs: f64, is_playing: bool) -> (String, String) {
    if width == 0 {
        return (String::new(), String::new());
    }

    if !is_playing {
        let top = "⠀".repeat(width);
        let bottom = "⣀".repeat(width);
        return (top, bottom);
    }

    let mut top_line = String::with_capacity(width * 4);
    let mut bottom_line = String::with_capacity(width * 4);

    let total_subcols = width * 2;
    let t = time_secs * 3.5;

    for col in 0..width {
        let sub1 = col * 2;
        let sub2 = col * 2 + 1;

        let h1 = compute_wave_height(sub1, total_subcols, t);
        let h2 = compute_wave_height(sub2, total_subcols, t);

        let (top1, bot1) = split_height_to_rows(h1);
        let (top2, bot2) = split_height_to_rows(h2);

        top_line.push(braille_cell_upper_from_levels(top1, top2));
        bottom_line.push(braille_cell_from_levels(bot1, bot2));
    }

    (top_line, bottom_line)
}

fn split_height_to_rows(h: usize) -> (usize, usize) {
    if h <= 4 {
        (0, h)
    } else {
        (h - 4, 4)
    }
}

fn compute_wave_height(x: usize, total_x: usize, t: f64) -> usize {
    let norm_x = (x as f64) / (total_x.max(1) as f64);

    // Multi-harmonic traveling wave with amplitude modulation
    let w1 = (norm_x * 8.0 * std::f64::consts::PI - t).sin();
    let w2 = (norm_x * 14.0 * std::f64::consts::PI + t * 0.7).sin() * 0.5;
    let w3 = (norm_x * 4.0 * std::f64::consts::PI - t * 1.5).cos() * 0.4;
    let envelope = (norm_x * std::f64::consts::PI).sin();

    let combined = (w1 + w2 + w3) * envelope;
    let scaled = ((combined + 1.2) / 2.4 * 8.0).clamp(0.0, 8.0);
    scaled.round() as usize
}

/// Compact single-line braille wave indicator for compact player bar.
pub fn render_compact_braille_wave(cells: usize, time_secs: f64, is_playing: bool) -> String {
    if cells == 0 {
        return String::new();
    }
    if !is_playing {
        return "⣀".repeat(cells);
    }
    let mut out = String::with_capacity(cells * 4);
    let total_subcols = cells * 2;
    let t = time_secs * 4.0;
    for col in 0..cells {
        let sub1 = col * 2;
        let sub2 = col * 2 + 1;
        let h1 = compute_wave_height(sub1, total_subcols, t).min(4);
        let h2 = compute_wave_height(sub2, total_subcols, t).min(4);
        out.push(braille_cell_from_levels(h1, h2));
    }
    out
}
