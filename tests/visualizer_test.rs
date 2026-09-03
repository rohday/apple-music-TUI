use apple_tui::ui::visualizer::{braille_cell_from_levels, render_braille_ribbon};

#[test]
fn test_braille_cell_generation() {
    let baseline = braille_cell_from_levels(0, 0);
    // Baseline is resting dots 7 and 8: ⣀
    assert_eq!(baseline, '⣀');

    let full = braille_cell_from_levels(4, 4);
    // Full cell: ⣿ (0x28FF)
    assert_eq!(full, '⣿');

    let half = braille_cell_from_levels(2, 2);
    // Half dots 7, 3, 8, 6: ⣤
    assert_eq!(half, '⣤');
}

#[test]
fn test_render_braille_ribbon_dimensions_and_states() {
    let width = 60;
    let (top_line, bottom_line) = render_braille_ribbon(width, 10.5, true);

    assert_eq!(top_line.chars().count(), width);
    assert_eq!(bottom_line.chars().count(), width);

    // All characters should be valid Braille Unicode (U+2800 to U+28FF)
    for c in top_line.chars().chain(bottom_line.chars()) {
        let u = c as u32;
        assert!((0x2800..=0x28FF).contains(&u), "Char {} must be Braille", c);
    }

    // When stopped/paused, resting flat baseline is returned
    let (top_stopped, bottom_stopped) = render_braille_ribbon(width, 0.0, false);
    assert_eq!(top_stopped.chars().count(), width);
    assert_eq!(bottom_stopped.chars().count(), width);
    // Flat resting wave has all identical baseline characters
    assert!(bottom_stopped.chars().all(|c| c == '⣀'));
}
