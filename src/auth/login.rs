use crate::config::{find_browser_binary, AuthConfig, Config};
use anyhow::{bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::time::Duration;
use tracing::{debug, info};

pub async fn launch_interactive_login() -> Result<AuthConfig> {
    let browser_bin = find_browser_binary()
        .context("No supported Chromium/Brave browser found for login flow")?;

    info!("Launching browser for Apple Music login: {:?}", browser_bin);

    let profile_dir = Config::get_profile_dir()?;

    // Clean up stale lock if any
    let lock_file = profile_dir.join("SingletonLock");
    if lock_file.exists() {
        let _ = std::fs::remove_file(&lock_file);
    }

    // Launch non-headless (visible) browser window for user to sign in
    let config = BrowserConfig::builder()
        .with_head()
        .user_data_dir(&profile_dir)
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-sync")
        .arg("--enable-widevine-cdm")
        .arg("--autoplay-policy=no-user-gesture-required")
        .chrome_executable(browser_bin)
        .build()
        .map_err(|e| anyhow::anyhow!("Browser config error: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                debug!("Login browser event debug: {:?}", e);
            }
        }
    });

    let page = browser.new_page("https://music.apple.com/login").await?;
    println!("Opened Apple Music login window.");
    println!("Please sign in with your Apple ID in the browser window.");
    println!("appleTUI is listening for your session token (waiting up to 3 minutes)...");

    let mut user_token: Option<String> = None;
    let poll_limit = 180; // 3 minutes timeout
    for i in 0..poll_limit {
        tokio::time::sleep(Duration::from_secs(1)).await;

        if let Ok(cookies) = page.get_cookies().await {
            for cookie in cookies {
                if cookie.name == "media-user-token" && !cookie.value.trim().is_empty() {
                    user_token = Some(cookie.value);
                    break;
                }
            }
        }

        if user_token.is_some() {
            println!("Successfully captured Apple Music user token!");
            break;
        }

        if (i + 1) % 15 == 0 {
            println!("Still waiting for login... ({}s elapsed)", i + 1);
        }
    }

    let _ = browser.close().await;
    handler_handle.abort();

    if let Some(token) = user_token {
        let auth = AuthConfig {
            developer_token: None, // Will use default fallback developer token
            music_user_token: Some(token),
        };
        auth.save()?;
        Ok(auth)
    } else {
        bail!("Login timed out or user token was not captured");
    }
}
