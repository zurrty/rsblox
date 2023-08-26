use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use itertools::Itertools;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DXVKRelease {
    pub tag_name: String,
    pub assets: Vec<DXVKReleaseAsset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DXVKReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn dxvk_installed_versions() -> io::Result<Vec<String>> {
    let dxvk_path = crate::data_path().join("components").join("dxvk");
    let dir = fs::read_dir(&dxvk_path)?;
    let versions = dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_str().unwrap_or_default().to_string())
        .collect_vec();
    Ok(versions)
}

pub async fn dxvk_versions() -> Result<Vec<DXVKRelease>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("rsblox/0.1")
        .build()?;

    let response = client
        .get("https://api.github.com/repos/doitsujin/dxvk/releases")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch DXVK release versions from Github API. HTTP Code {} ",
            response.status()
        )
        .into());
    }

    Ok(response.json().await?)
}

pub async fn download_dxvk(version: DXVKRelease) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("rsblox/0.1")
        .build()?;
    let download_url = &version.assets.first().unwrap().browser_download_url;
    let response = client.get(download_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download DXVK. HTTP Code {} ", response.status()).into());
    }
    let path = Path::new("/tmp/").join(download_url.split("/").last().unwrap());
    let mut file = std::fs::File::create(&path)?;
    let content = response.bytes().await?;

    file.write_all(&content)?;
    let out_path = crate::data_path().join("components").join("dxvk");
    if !out_path.exists() {
        std::fs::create_dir_all(&out_path)?;
    }
    Command::new("tar")
        .args(&[
            "-xf",
            path.to_str().unwrap(),
            "-C",
            &out_path.to_str().unwrap(),
        ])
        .spawn()?;
    println!("Downloaded DXVK {}", version.tag_name);
    Ok(out_path)
}
