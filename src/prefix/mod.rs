pub mod components;

use std::fs;
use std::io::{self, Result};
use std::path::{Path, PathBuf};
use std::process::Child;

use itertools::Itertools;
use wincompatlib::prelude::*;

pub fn installs_path() -> PathBuf {
    super::data_path().join("installs")
}

#[derive(Debug)]
pub struct WinePrefix {
    pub wine: Wine,
}

impl Default for WinePrefix {
    fn default() -> Self {
        Self::new(installs_path().join("Default")).unwrap()
    }
}

impl WinePrefix {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !fs::try_exists(path)? {
            fs::create_dir_all(path).ok();
        }
        let mut wine = Wine::default();
        wine.prefix = Some(path.to_owned());
        Ok(Self { wine })
    }
    pub fn path(&self) -> &PathBuf {
        self.wine.prefix.as_ref().unwrap()
    }
    pub fn versions_path(&self) -> PathBuf {
        self.path()
            .join("drive_c/Program Files (x86)/Roblox/Versions")
    }
    pub fn versions(&self) -> Result<Vec<PathBuf>> {
        let versions = std::fs::read_dir(
            self.path()
                .join("drive_c/Program Files (x86)/Roblox/Versions"),
        )?
        .filter_map(|v| v.ok())
        .filter(|v| v.file_name().to_str().unwrap().starts_with("version-"))
        .sorted_by(|a, b| {
            let time_a = a.metadata().map(|meta| meta.created().unwrap()).unwrap();
            let time_b = b.metadata().map(|meta| meta.created().unwrap()).unwrap();
            time_a.cmp(&time_b)
        })
        .map(|version| version.path().to_path_buf())
        .rev()
        .collect_vec();
        Ok(versions)
    }
    pub fn execute(&self, path: impl Into<PathBuf>, args: &[&str]) -> io::Result<Child> {
        let path: PathBuf = path.into();
        let mut arguments = vec![path.to_str().unwrap()];
        arguments.append(&mut Vec::from(args));
        self.wine.run_args(args)
    }
    pub fn run_args(&self, args: &[&str]) -> io::Result<Child> {
        self.wine.run_args(args)
    }
    pub fn run(&self, path: impl Into<PathBuf>) -> io::Result<Child> {
        self.wine.run(path.into())
    }
    pub async fn install_roblox(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        const URL: &str = "http://roblox.com/download/client";
        const EXE: &str = "/tmp/RobloxPlayerLauncher.exe";
        if !fs::try_exists(EXE)? {
            let response = reqwest::get(URL).await?;

            if !response.status().is_success() {
                return Err(String::from("Failed to download Roblox installer!").into());
            }

            let mut file = std::fs::File::create(EXE)?;

            let content = response.bytes().await?;

            file.write_all(&content)?;
        }
        println!("If the installer fails with \"An error occurred in the secure channel support\", you need to install lib32-gnutls from your distributions package manager.");
        self.wine.run(EXE)?;
        Ok(())
    }
    pub fn find_player(&self) -> io::Result<PathBuf> {
        for version in self.versions()? {
            println!("{:?}", version);
            let exe_path = version.join("RobloxPlayerBeta.exe");
            if fs::try_exists(&exe_path)? {
                return Ok(exe_path);
            }
        }
        Err(io::ErrorKind::NotFound.into())
    }
    pub fn find_launcher(&self) -> io::Result<PathBuf> {
        for version in self.versions()? {
            println!("{:?}", version);
            let exe_path = version.join("RobloxPlayerLauncher.exe");
            if fs::try_exists(&exe_path)? {
                return Ok(exe_path);
            }
        }
        Err(io::ErrorKind::NotFound.into())
    }
    pub fn find_studio(&self) -> io::Result<PathBuf> {
        for version in self.versions()? {
            let exe_path = version.join("RobloxStudioBeta.exe");
            if fs::try_exists(&exe_path)? {
                return Ok(exe_path);
            }
        }
        Err(io::ErrorKind::NotFound.into())
    }
}
