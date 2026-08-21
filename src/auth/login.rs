use crate::config::{find_browser_binary, AuthConfig};
use anyhow::{bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::time::Duration;
use tracing::{info, warn};

pub async fn launch_interactive_login() -> Result<AuthConfig> {
    let browser_bin = find_browser_binary()
        .context("No supported Chromium/Brave browser found for login flow")?;

    info!("Launching browser for Apple Music login: {:?}", browser_bin);

    // Launch non-headless browser window for user to sign in
    let config = BrowserConfig::builder()
        .chrome_executable(browser_bin)
        .build()
        .map_err(|e| anyhow::anyhow!("Browser config error: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;
    let handler_handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                warn!("Login browser event error: {:?}", e);
                break;
            }
        }
    });

    let page = browser.new_page("https://music.apple.com/login").await?;
    info!("Opened login page. Waiting for user authorization...");

    let mut user_token: Option<String> = None;
    let poll_limit = 120; // 2 minutes timeout
    for _ in 0..poll_limit {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let cookies = page.get_cookies().await?;
        for cookie in cookies {
            if cookie.name == "media-user-token" {
                user_token = Some(cookie.value);
                break;
            }
        }

        if user_token.is_some() {
            info!("Captured media-user-token!");
            break;
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
