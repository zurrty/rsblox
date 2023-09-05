pub mod gui;
pub mod prefix;
use clap::{arg, Command, Parser};
use std::path::PathBuf;

pub fn data_path() -> PathBuf {
    let mut path = PathBuf::from(
        std::env::var("HOME").expect("couldn't find home directory, are you running as root?"),
    );
    path.extend([".local", "share", "rsblox"]);
    path
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    WinetricksNotInstalled,
    PlayerNotInstalled,

    IO(#[from] std::io::Error),
    Reqwest(#[from] reqwest::Error),
    Other(#[from] Box<dyn std::error::Error>),
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
    install: Option<PathBuf>,

    input: Option<String>,
}

fn cli() -> Command {
    Command::new("rsblox")
        .arg_required_else_help(false)
        .arg(arg!(-i --install [INSTALL] "Path to wine prefix"))
        .subcommand(
            Command::new("player")
                .alias("app")
                .about("Launches Roblox experiences")
                .arg(arg!([INPUT])),
        )
        .subcommand(
            Command::new("studio")
                .about("Launches Roblox Studio")
                .arg(arg!([INPUT])),
        )
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli().get_matches();
    let is_interactive = dialoguer::console::user_attended();

    let prefix = match matches.get_one::<PathBuf>("INSTALL") {
        Some(path) => {
            if path.is_absolute() {
                prefix::WinePrefix::new(path).unwrap_or_default()
            } else {
                prefix::WinePrefix::new(prefix::installs_path().join(path)).unwrap_or_default()
            }
        }
        None => prefix::WinePrefix::default(),
    };

    match matches.subcommand() {
        Some(("player", submatches)) => {
            let player_path = prefix.find_player()?;
            let input = submatches
                .get_one::<String>("INPUT")
                .cloned()
                .unwrap_or_default();
            prefix.run_args(&[player_path.to_str().unwrap(), &input])?;
        }
        Some(("studio", submatches)) => {
            let studio_path = prefix.find_studio()?;
            let input = submatches
                .get_one::<String>("INPUT")
                .cloned()
                .unwrap_or_default();
            prefix.run_args(&[studio_path.to_str().unwrap(), &input])?;
        }
        _ => {
            if is_interactive {
                gui::gui(&prefix).await?;
            }
            else {
                let launcher_path = prefix.find_player()?;
                prefix.run(launcher_path)?;
            }
        }
    }
    Ok(())
}
