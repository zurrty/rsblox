use std::process::Command;

use async_recursion::async_recursion;
use dialoguer::theme::ColorfulTheme;
use dialoguer::MultiSelect;
use dialoguer::Select;
use itertools::Itertools;
use wincompatlib::prelude::Dxvk;
use wincompatlib::prelude::InstallParams;

use crate::prefix::WinePrefix;

fn select(choices: &[&str], prompt: &str) -> crate::Result<usize> {
    Ok(Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(choices)
        .default(0)
        .interact()?)
}
// for if i want multiple choice things
#[allow(unused)]
fn multi(choices: &[(&str, bool)], prompt: &str) -> crate::Result<Vec<usize>> {
    let mut items = Vec::with_capacity(choices.len());
    let mut defaults = Vec::with_capacity(choices.len());
    choices.iter().for_each(|(item, value)| {
        items.push(item);
        defaults.push(value.clone());
    });
    Ok(MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .defaults(&defaults)
        .interact()?)
}
#[async_recursion(?Send)]
pub(crate) async fn gui(prefix: &WinePrefix) -> crate::Result<()> {
    let choices = &[
        "Open App",
        "Components",
        "(Re)Install Roblox",
        "Winetricks",
        "Exit",
    ];

    match select(choices, "What do you want to do?")? {
        0 => {
            if let Ok(player_path) = prefix.find_player() {
                prefix.run(player_path)?;
            } else {
                println!("Roblox player is either not installed or could not be located.")
            }
        }
        1 => manage_components(prefix).await?,
        2 => prefix.install_roblox().await?,
        3 => {
            println!("Starting Winetricks...");
            let process = Command::new("winetricks")
                .env("WINE_PREFIX", prefix.path().to_str().unwrap())
                .spawn();
            match process {
                Ok(mut process) => {
                    process.wait()?;
                }
                Err(_) => {
                    println!("Winetricks is not installed.");
                    return Ok(())
                }
            }
            gui(&prefix).await?;
        }
        _ => return Ok(()),
    }
    Ok(())
}

async fn manage_components(prefix: &WinePrefix) -> Result<(), Box<dyn std::error::Error>> {
    let choices = &["DXVK", "FPS Unlocker"];
    let dxvk_path = crate::data_path().join("components").join("dxvk");
    match select(choices, "Manage Components")? {
        /* DXVK */
        0 => match select(&["Toggle", "Download"], "Manage Components > DXVK")? {
            0 => {
                if !dxvk_path.exists() {
                    std::fs::create_dir_all(&dxvk_path)?;
                }
                if Dxvk::get_version(&prefix.path()).unwrap_or(None).is_some() {
                    prefix.wine.uninstall_dxvk(InstallParams::default())?;
                    println!("DXVK disabled.");
                } else {
                    let versions = crate::prefix::dxvk::dxvk_installed_versions()?;
                    if versions.is_empty() {
                        println!("No DXVK versions installed, choose one to download:");
                        download_dxvk(prefix, false).await?;
                    }
                    let versions = crate::prefix::dxvk::dxvk_installed_versions()?;
                    let index = select(
                        &versions.iter().map(|s| s.as_str()).collect_vec(),
                        "Select DXVK Version",
                    )?;
                    let version = versions.get(index).cloned().unwrap_or(String::new());
                    println!("Installing DXVK version {version}, this may take a while!");
                    prefix
                        .wine
                        .install_dxvk(dxvk_path.join(version), InstallParams::default())?;
                    println!("Done!");
                }
            }
            1 => {
                download_dxvk(prefix, true)
                    .await
                    .expect("Failed to install DXVK.");
            }
            _ => {}
        },
        1 => {
            todo!("FPS Unlocker")
        }
        _ => {}
    }
    Ok(())
}

async fn download_dxvk(
    prefix: &WinePrefix,
    ask_to_install: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let versions = crate::prefix::dxvk::dxvk_versions().await?;
    let index = select(
        &versions.iter().map(|v| v.tag_name.as_str()).collect_vec(),
        "Select DXVK Version",
    )?;
    if let Some(version) = versions.get(index).cloned() {
        let path = crate::prefix::dxvk::download_dxvk(version).await?;
        if ask_to_install
            && dialoguer::Confirm::new()
                .with_prompt("Would you like to enable DXVK now?")
                .interact()?
                == false
        {
            return Ok(());
        }
        prefix.wine.install_dxvk(path, InstallParams::default())?;
    }
    Ok(())
}
