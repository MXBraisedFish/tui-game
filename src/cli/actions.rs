use anyhow::Result;
use std::process::Command;

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
    let install_dir = path_utils::runtime_dir()?;
    let install_dir = install_dir.to_string_lossy().to_string();
    let lang = CliLang::load();
    if !run_helper_script_blocking("remove", &[install_dir.as_str()])? {
        println!("{}", lang.t("remove.helper_missing"));
        return Ok(1);
    }
    Ok(0)
}

fn run_helper_script_blocking(helper_name: &str, args: &[&str]) -> Result<bool> {
    let script = path_utils::helper_script_file(helper_name)?;
    if !script.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    let status = {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(script.as_os_str());
        for arg in args {
            command.arg(arg);
        }
        command.status()?
    };

    #[cfg(not(target_os = "windows"))]
    let status = {
        let mut command = Command::new("sh");
        command.arg(script.as_os_str());
        for arg in args {
            command.arg(arg);
        }
        command.status()?
    };

    Ok(status.success())
}
