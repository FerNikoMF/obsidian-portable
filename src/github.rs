use serde::Deserialize;
use std::time::Duration;

const API_URL: &str =
    "https://api.github.com/repos/obsidianmd/obsidian-releases/releases/latest";

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

/// Fetches the latest Obsidian release from the GitHub API.
pub fn fetch_latest() -> Result<Release, Box<dyn std::error::Error>> {
    let release: Release = reqwest::blocking::Client::builder()
        .user_agent("obsidian-portable-launcher")
        .timeout(Duration::from_secs(15))
        .build()?
        .get(API_URL)
        .send()?
        .json()?;
    Ok(release)
}
