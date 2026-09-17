use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn};

const API_URL: &str =
    "https://api.github.com/repos/obsidianmd/obsidian-releases/releases/latest";
const ALL_RELEASES_URL: &str =
    "https://api.github.com/repos/obsidianmd/obsidian-releases/releases?per_page=30";
#[derive(Deserialize, Clone)]
pub struct Asset {
    pub name:                 String,
    pub browser_download_url: String,
}

#[derive(Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub assets:   Vec<Asset>,
}

/// Fetches the latest Obsidian release from the GitHub API with exponential backoff.
/// Retries up to 3 times on transient network errors (1s → 2s → 4s).
pub fn fetch_latest() -> Result<Release, Box<dyn std::error::Error>> {
    let max_retries = 3;
    let mut last_err: Option<Box<dyn std::error::Error>> = None;

    for attempt in 0..max_retries {
        if attempt > 0 {
            let delay = Duration::from_secs(1 << attempt); // 2s, 4s
            warn!(attempt, delay_secs = delay.as_secs(), "Retrying GitHub API request");
            std::thread::sleep(delay);
        }

        match try_fetch() {
            Ok(release) => {
                info!(tag = %release.tag_name, "Fetched latest release");
                return Ok(release);
            }
            Err(e) => {
                let msg = e.to_string();
                // Don't retry on rate limit or auth errors
                if msg.contains("rate limit") || msg.contains("403") || msg.contains("401") {
                    return Err(e);
                }
                warn!(attempt, error = %msg, "GitHub API request failed");
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "Unknown error".into()))
}

fn try_fetch() -> Result<Release, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::Client::builder()
        .user_agent("obsidian-portable-launcher")
        .timeout(Duration::from_secs(15))
        .build()?
        .get(API_URL)
        .send()?;

    // GitHub's unauthenticated API is rate-limited to 60 req/hour per IP.
    // Surface that as a clear message instead of a confusing JSON parse error.
    if resp.status() == reqwest::StatusCode::FORBIDDEN
        || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err("GitHub API rate limit exceeded — try again later".into());
    }

    let resp = resp.error_for_status()?;
    let release: Release = resp.json()?;
    Ok(release)
}

/// Fetches up to 30 recent releases (newest first) for fallback search.
pub fn fetch_releases() -> Result<Vec<Release>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::Client::builder()
        .user_agent("obsidian-portable-launcher")
        .timeout(Duration::from_secs(15))
        .build()?
        .get(ALL_RELEASES_URL)
        .send()?;

    if resp.status() == reqwest::StatusCode::FORBIDDEN
        || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        return Err("GitHub API rate limit exceeded — try again later".into());
    }

    let resp = resp.error_for_status()?;
    let releases: Vec<Release> = resp.json()?;
    info!(count = releases.len(), "Fetched release list");
    Ok(releases)
}
