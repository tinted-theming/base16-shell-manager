use crate::config::Config;
use crate::constants::{REPO_DIR, SCHEMES_REPO_NAME, SCHEMES_REPO_REVISION, SCHEMES_REPO_URL};
use crate::utils::git_clone;
use anyhow::{anyhow, Result};
use std::fs::{remove_file as remove_symlink, symlink_metadata};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use url::Url;

fn install_git_url(
    data_item_path: &Path,
    item_name: &str,
    item_git_url: &str,
    revision: Option<&str>,
    is_quiet: bool,
) -> Result<()> {
    if !data_item_path.is_dir() {
        git_clone(item_git_url, data_item_path, revision)?;

        if !is_quiet {
            println!("{} installed", item_name);
        }
    } else if !is_quiet {
        println!("{} already installed", item_name);
    }

    Ok(())
}

fn install_dir(
    data_item_path: &Path,
    item_name: &str,
    item_path: &Path,
    is_quiet: bool,
) -> Result<()> {
    if item_path.exists() && !item_path.is_dir() {
        return Err(anyhow!(
            "{} is not a symlink to a directory. Please remove it and try again",
            item_path.display()
        ));
    }

    if data_item_path.exists() {
        match symlink_metadata(data_item_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    if remove_symlink(data_item_path).is_err() {
                        return Err(anyhow!("Error trying to remove symlink at \"{}\". Remove it manually and try again", data_item_path.display()));
                    }
                    symlink(item_path, data_item_path)?;

                    if !is_quiet {
                        println!("{} already installed", item_name);
                    }
                }
            }
            Err(_) => {
                return Err(anyhow!("\"{}\" is a not a symlink, but according to your config it should be. Please remove this directory and try again", data_item_path.display()));
            }
        }
    } else {
        symlink(item_path, data_item_path)?;

        if !is_quiet {
            println!("{} installed", item_name);
        }
    }

    Ok(())
}

/// Install cli tool
///
/// Clones the provided config repositories and ensures everything is ready for when the user runs
/// any other command
pub fn install(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let data_item_path = hooks_path.join(&item.name);
        let item_path = PathBuf::from(item.path.as_str());

        match Url::parse(item.path.as_str()) {
            Ok(_) => install_git_url(
                &data_item_path,
                item.name.as_str(),
                item.path.as_str(),
                item.revision.as_deref(),
                is_quiet,
            )?,
            Err(_) => install_dir(&data_item_path, item.name.as_str(), &item_path, is_quiet)?,
        }
    }

    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);

    install_git_url(
        &schemes_repo_path,
        SCHEMES_REPO_NAME,
        SCHEMES_REPO_URL,
        Some(SCHEMES_REPO_REVISION),
        is_quiet,
    )?;

    Ok(())
}
