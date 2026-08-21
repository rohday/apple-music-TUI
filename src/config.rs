use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "appleTUI";
const APPLICATION: &str = "appleTUI";

pub const DEFAULT_FALLBACK_DEVELOPER_TOKEN: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldNVDNGM1dTMkQifQ.eyJpc3MiOiJNQTY2M1hTNEszIiwiaWF0IjoxNTc4OTQ2ODQyLCJleHAiOjE3NzA3NzA4NDJ9.b4sD8Q3N2P4dM9u2U9V4I9Vw3L7c3b2r1o5p9l9q2u4m3n5o1p6q8r9s0t1u2v3";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume: u8,
    pub storefront: String,
    pub browser_path: Option<PathBuf>,
    pub mock_mode: bool,
    pub tick_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 80,
            storefront: "us".to_string(),
            browser_path: None,
            mock_mode: false,
            tick_rate_ms: 250,
        }
    }
}

impl Config {
    pub fn get_config_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            let dir = proj_dirs.config_dir().to_path_buf();
            fs::create_dir_all(&dir)?;
            Ok(dir)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let dir = PathBuf::from(home).join(".config").join("appletui");
            fs::create_dir_all(&dir)?;
            Ok(dir)
        }
    }

    pub fn get_profile_dir() -> Result<PathBuf> {
        let dir = Self::get_config_dir()?.join("browser_profile");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn default_config_path() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("config.json"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::default_config_path() {
            if path.exists() {
                if let Ok(cfg) = Self::load_from(&path) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;
        let config: Config = serde_json::from_str(&content)
            .with_context(|| "Failed to parse config JSON")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_config_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub developer_token: Option<String>,
    pub music_user_token: Option<String>,
}

impl AuthConfig {
    pub fn default_auth_path() -> Result<PathBuf> {
        Ok(Config::get_config_dir()?.join("auth.json"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::default_auth_path() {
            if path.exists() {
                if let Ok(auth) = Self::load_from(&path) {
                    return auth;
                }
            }
        }
        Self::default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read auth file at {:?}", path))?;
        let auth: AuthConfig = serde_json::from_str(&content)
            .with_context(|| "Failed to parse auth JSON")?;
        Ok(auth)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_auth_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.music_user_token
            .as_ref()
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false)
    }
}

pub fn find_browser_binary() -> Option<PathBuf> {
    if let Ok(custom_path) = std::env::var("CHROME_BIN").or_else(|_| std::env::var("BRAVE_BIN")) {
        let path = PathBuf::from(custom_path);
        if path.is_file() {
            return Some(path);
        }
    }

    let candidates = [
        "/usr/bin/brave-browser",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
        "/usr/local/bin/brave-browser",
        "/usr/local/bin/google-chrome",
        "/usr/local/bin/chromium",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}
