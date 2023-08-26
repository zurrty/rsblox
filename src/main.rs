#![feature(fs_try_exists)]

pub mod gui;
pub mod prefix;
use std::path::PathBuf;

use clap::Parser;

pub fn data_path() -> PathBuf {
    let mut path = PathBuf::from(
        std::env::var("HOME").expect("couldn't find home directory, are you running as root?"),
    );
    path.extend([".local", "share", "rsblox"]);
    path
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    IO(#[from] std::io::Error),
    Reqwest(#[from] reqwest::Error),
    Unknown(#[from] Box<dyn std::error::Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    install: Option<String>,
    input: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let prefix = match args.install {
        Some(path) => {
            prefix::WinePrefix::new(prefix::installs_path().join(path)).unwrap_or_default()
        }
        None => prefix::WinePrefix::default(),
    };
    if let Some(input) = args.input {
        if input.starts_with("roblox-player:") {
            prefix
                .run_args(&[prefix.find_launcher().unwrap().to_str().unwrap(), &input])
                .unwrap();
        } else if input.starts_with("roblox-studio:") {
            prefix
                .run_args(&[prefix.find_studio().unwrap().to_str().unwrap(), &input])
                .unwrap();
        } else if input == "install" {
            prefix.install_roblox().await.unwrap();
        } else if input == "app" {
            prefix
                .run(prefix.find_player().unwrap().to_str().unwrap())
                .unwrap();
        }
    } else {
        gui::gui(&prefix).await.unwrap();
    }
}
