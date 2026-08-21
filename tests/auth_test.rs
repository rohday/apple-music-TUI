use apple_tui::config::AuthConfig;

#[test]
fn test_auth_config_validity() {
    let mut auth = AuthConfig::default();
    assert!(!auth.is_authenticated());

    auth.music_user_token = Some("valid_token_xyz".to_string());
    assert!(auth.is_authenticated());

    auth.music_user_token = Some("   ".to_string());
    assert!(!auth.is_authenticated());
}
