use std::io::{self, Write};

use anyhow::Result;

use crate::cli::lang::CliLang;
use crate::updater::github::{
    CURRENT_VERSION_TAG, latest_release_download, latest_release_notification, normalize_tag,
    platform_update_asset_name, spawn_helper_script,
};
use crate::utils::path_utils;

pub fn run_version_cli() -> Result<i32> {
    let lang = CliLang::load();
    let current = normalize_tag(CURRENT_VERSION_TAG);
    println!("{}", lang.fmt("version.current", &[("{version}", current.as_str())]));

    match latest_release_notification()? {
        Some(latest) => {
            println!(
                "{}",
                lang.fmt("version.latest", &[("{version}", latest.latest_version.as_str())])
            );
            if crate::updater::github::is_version_newer(&latest.latest_version, &current) {
                println!("{}", lang.t("version.update_available"));
            } else {
                println!("{}", lang.t("version.up_to_date"));
            }
        }
        None => {
            println!("{}", lang.t("version.check_failed"));
        }
    }

    Ok(0)
}

pub fn run_updata_cli(_version_override: Option<String>, _release_url_override: Option<String>) -> Result<i32> {
    let lang = CliLang::load();
    println!("{}", lang.t("updata.checking"));

    match latest_release_download()? {
        Some(download) => {
            if !crate::updater::github::is_version_newer(&download.latest_version, CURRENT_VERSION_TAG) {
                println!("{}", lang.t("updata.no_update"));
                return Ok(0);
            }

            let install_dir = path_utils::runtime_dir()?;
            let install_dir = install_dir.to_string_lossy().to_string();
            let helper_args = [
                install_dir.as_str(),
                download.asset_url.as_str(),
                download.asset_name.as_str(),
                download.latest_version.as_str(),
            ];

            println!(
                "{}",
                lang.fmt("updata.update_found", &[("{version}", download.latest_version.as_str())])
            );
            if !spawn_helper_script("updata", &helper_args, None)? {
                println!("{}", lang.t("updata.helper_missing"));
                return Ok(1);
            }
            println!("{}", lang.t("updata.launching"));
            Ok(0)
        }
        None => {
            println!(
                "{}",
                lang.fmt("updata.asset_missing", &[("{asset}", platform_update_asset_name())])
            );
            Ok(1)
        }
    }
}

pub fn run_remove_cli() -> Result<i32> {
    let lang = CliLang::load();
    if !confirm(&lang.t("remove.confirm_first"))? {
        println!("{}", lang.t("remove.cancelled"));
        return Ok(0);
    }

    println!("{}", lang.t("remove.confirm_mode"));
    print!("> ");
    io::stdout().flush()?;
    let mut mode = String::new();
    io::stdin().read_line(&mut mode)?;
    let mode = mode.trim();
    let (delete_data_flag, mode_text) = match mode {
        "1" => ("0", lang.t("remove.mode.keep")),
        "2" => ("1", lang.t("remove.mode.full")),
        _ => {
            println!("{}", lang.t("remove.cancelled"));
            return Ok(0);
        }
    };

    if !confirm(&lang.fmt("remove.confirm_second", &[("{mode}", mode_text.as_str())]))? {
        println!("{}", lang.t("remove.cancelled"));
        return Ok(0);
    }

    let install_dir = path_utils::runtime_dir()?;
    let install_dir = install_dir.to_string_lossy().to_string();
    if !spawn_helper_script("remove", &[install_dir.as_str(), delete_data_flag], None)? {
        println!("{}", lang.t("remove.helper_missing"));
        return Ok(1);
    }

    println!("{}", lang.t("remove.launching"));
    Ok(0)
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y"))
}
