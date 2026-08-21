use apple_tui::config::{find_browser_binary, AuthConfig, Config};
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.volume, 80);
    assert_eq!(cfg.storefront, "us");
    assert!(!cfg.mock_mode);
}

#[test]
fn test_config_save_and_load() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.json");
    let auth_path = tmp.path().join("auth.json");

    let cfg = Config {
        volume: 65,
        storefront: "jp".to_string(),
        ..Default::default()
    };
    cfg.save_to(&config_path).unwrap();

    let loaded = Config::load_from(&config_path).unwrap();
    assert_eq!(loaded.volume, 65);
    assert_eq!(loaded.storefront, "jp");

    let auth = AuthConfig {
        developer_token: Some("dev_token_123".to_string()),
        music_user_token: Some("user_token_abc".to_string()),
    };
    auth.save_to(&auth_path).unwrap();

    let loaded_auth = AuthConfig::load_from(&auth_path).unwrap();
    assert_eq!(loaded_auth.developer_token.as_deref(), Some("dev_token_123"));
    assert_eq!(loaded_auth.music_user_token.as_deref(), Some("user_token_abc"));
    assert!(loaded_auth.is_authenticated());
}

#[test]
fn test_find_browser_binary() {
    let browser = find_browser_binary();
    assert!(browser.is_some(), "Should find a chromium-compatible browser (e.g. brave-browser)");
}
