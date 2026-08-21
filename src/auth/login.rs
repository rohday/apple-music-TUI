use crate::config::{find_browser_binary, AuthConfig, Config, DEFAULT_FALLBACK_DEVELOPER_TOKEN};
use anyhow::{bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, info};

pub async fn fetch_live_developer_token() -> Result<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let html = client
        .get("https://music.apple.com/us/browse")
        .send()
        .await?
        .text()
        .await?;

    // Search for /assets/index~*.js
    let mut js_url = None;
    for line in html.split('\"') {
        if line.starts_with("/assets/index~") && line.ends_with(".js") {
            js_url = Some(format!("https://music.apple.com{}", line));
            break;
        }
    }

    if let Some(url) = js_url {
        let js = client.get(&url).send().await?.text().await?;
        if let Some(pos) = js.find("$c=\"") {
            let start = pos + 4;
            if let Some(end) = js[start..].find('\"') {
                let token = &js[start..start + end];
                if token.starts_with("eyJ") {
                    info!(
                        "Successfully extracted live developer token from Apple Music web bundle"
                    );
                    return Ok(token.to_string());
                }
            }
        }
    }

    Ok(DEFAULT_FALLBACK_DEVELOPER_TOKEN.to_string())
}

pub async fn launch_interactive_login() -> Result<AuthConfig> {
    let browser_bin = find_browser_binary()
        .context("No supported Chromium/Brave browser found for login flow")?;

    info!("Launching browser for Apple Music login: {:?}", browser_bin);

    // Fetch live developer token concurrently
    let dev_token = fetch_live_developer_token()
        .await
        .unwrap_or_else(|_| DEFAULT_FALLBACK_DEVELOPER_TOKEN.to_string());

    let profile_dir = Config::get_profile_dir()?;
    Config::clean_stale_browser_locks(&profile_dir);

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
            developer_token: Some(dev_token),
            music_user_token: Some(token),
        };
        auth.save()?;
        Ok(auth)
    } else {
        bail!("Login timed out or user token was not captured");
    }
}
