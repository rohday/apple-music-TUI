use apple_tui::ui::shimmer::render_shimmer_progress_bar;
use ratatui::style::Color;

#[test]
fn test_render_shimmer_progress_bar_lengths_and_states() {
    let width = 50;

    // Test playing state at 50%
    let line_playing = render_shimmer_progress_bar(
        width,
        0.5,
        1.5,
        true,
        Color::Red,
        Color::White,
        Color::DarkGray,
    );
    let rendered_text: String = line_playing
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert_eq!(rendered_text.chars().count(), width);

    // Test empty/zero progress
    let line_zero = render_shimmer_progress_bar(
        width,
        0.0,
        0.0,
        false,
        Color::Red,
        Color::White,
        Color::DarkGray,
    );
    let zero_text: String = line_zero.spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(zero_text.chars().count(), width);
    assert!(zero_text.chars().all(|c| c == ' '));

    // Test 100% full progress
    let line_full = render_shimmer_progress_bar(
        width,
        1.0,
        2.5,
        true,
        Color::Red,
        Color::White,
        Color::DarkGray,
    );
    let full_text: String = line_full.spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(full_text.chars().count(), width);
    assert!(full_text.chars().all(|c| c == '█'));
}
