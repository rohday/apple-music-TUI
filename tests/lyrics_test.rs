use apple_tui::api::lyrics::LyricsData;

#[test]
fn test_parse_lrc_synced_lyrics() {
    let lrc = r#"
[00:05.10] Intro music playing
[00:15.50] First line of lyrics
[00:22.00] Second line of lyrics
[01:05.75] Chorus begins here
"#;

    let data = LyricsData::parse_lrc(lrc);
    assert!(data.synced);
    assert_eq!(data.lines.len(), 4);

    assert_eq!(data.lines[0].time_secs, 5.10);
    assert_eq!(data.lines[0].text, "Intro music playing");

    assert_eq!(data.lines[1].time_secs, 15.50);
    assert_eq!(data.lines[1].text, "First line of lyrics");

    assert_eq!(data.lines[2].time_secs, 22.00);
    assert_eq!(data.lines[3].time_secs, 65.75);

    // Test active line detection
    assert_eq!(data.current_line_idx(0.0), None);
    assert_eq!(data.current_line_idx(5.10), Some(0));
    assert_eq!(data.current_line_idx(10.0), Some(0));
    assert_eq!(data.current_line_idx(15.50), Some(1));
    assert_eq!(data.current_line_idx(20.0), Some(1));
    assert_eq!(data.current_line_idx(25.0), Some(2));
    assert_eq!(data.current_line_idx(70.0), Some(3));
}

#[test]
fn test_parse_plain_lyrics() {
    let plain = "Line 1\nLine 2\nLine 3";
    let data = LyricsData::from_plain(plain);
    assert!(!data.synced);
    assert_eq!(data.lines.len(), 3);
    assert_eq!(data.lines[0].text, "Line 1");
    assert_eq!(data.lines[1].text, "Line 2");
    assert_eq!(data.lines[2].text, "Line 3");
}
