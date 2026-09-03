use apple_tui::ui::theme::{Theme, ThemePreset};

#[test]
fn test_theme_preset_cycle() {
    let t0 = ThemePreset::AppleDark;
    let t1 = t0.cycle();
    assert_eq!(t1, ThemePreset::CatppuccinMocha);
    let t2 = t1.cycle();
    assert_eq!(t2, ThemePreset::TokyoNight);
    let t3 = t2.cycle();
    assert_eq!(t3, ThemePreset::GruvboxDark);
    let t4 = t3.cycle();
    assert_eq!(t4, ThemePreset::Nord);
    let t5 = t4.cycle();
    assert_eq!(t5, ThemePreset::AppleDark);
}

#[test]
fn test_theme_colors_and_styles() {
    for preset in [
        ThemePreset::AppleDark,
        ThemePreset::CatppuccinMocha,
        ThemePreset::TokyoNight,
        ThemePreset::GruvboxDark,
        ThemePreset::Nord,
    ] {
        let theme: Theme = preset.theme();
        let title_style = theme.title_style();
        let border_focused = theme.border_style(true);
        let border_unfocused = theme.border_style(false);
        let selected_row = theme.selected_row_style();

        assert!(!preset.display_name().is_empty());
        assert_ne!(theme.accent, theme.border_unfocused);
        assert_eq!(border_focused.fg, Some(theme.border_focused));
        assert_eq!(border_unfocused.fg, Some(theme.border_unfocused));
        assert_eq!(title_style.fg, Some(theme.accent));
        assert_eq!(selected_row.bg, Some(theme.highlight_bg));
    }
}
