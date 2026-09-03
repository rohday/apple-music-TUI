use apple_tui::ui::visualizer::{compute_spectrum_bars, compute_band_heights};

#[test]
fn test_visualizer_bars_playing_vs_stopped() {
    let stopped_bars = compute_spectrum_bars(10.0, 12, false);
    assert_eq!(stopped_bars.chars().count(), 12);
    // When stopped/paused, all bars should be low/flat
    for c in stopped_bars.chars() {
        assert!(c == ' ' || c == ' ');
    }

    let playing_bars_1 = compute_spectrum_bars(1.0, 16, true);
    let playing_bars_2 = compute_spectrum_bars(2.5, 16, true);
    assert_eq!(playing_bars_1.chars().count(), 16);
    assert_eq!(playing_bars_2.chars().count(), 16);

    // Spectrum varies with time
    assert_ne!(playing_bars_1, playing_bars_2);
}

#[test]
fn test_band_heights_range() {
    let heights = compute_band_heights(5.0, 24, true);
    assert_eq!(heights.len(), 24);
    for h in heights {
        assert!((0.0..=1.0).contains(&h), "Height {} must be in [0.0, 1.0]", h);
    }
}
