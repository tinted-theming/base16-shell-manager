use crate::config::{Config, ConfigItem, DEFAULT_CONFIG_SHELL};
use crate::constants::{REPO_NAME, SCHEME_EXTENSION};
use anyhow::{anyhow, Context, Error, Result};
use home::home_dir;
use rand::Rng;
use regex::bytes::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use tinted_builder::SchemeSystem;

/// Ensures that a directory exists, creating it if it does not.
pub fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {}", path.display()))?;
    }

    Ok(())
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    let mut file = File::create(path)
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.display()))?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_shell_command_from_string(config_path: &Path, command: &str) -> Result<Vec<String>> {
    let config = Config::read(config_path)?;
    let shell = config
        .shell
        .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.to_string());
    let full_command = shell.replace("{}", command);

    shell_words::split(&full_command).map_err(anyhow::Error::new)
}

pub fn git_clone(repo_url: &str, target_dir: &Path, revision: Option<&str>) -> Result<()> {
    if target_dir.exists() {
        return Err(anyhow!(
            "Error cloning {}. Target directory '{}' already exists",
            repo_url,
            target_dir.display()
        ));
    }

    let command = format!("git clone \"{}\" \"{}\"", repo_url, target_dir.display());
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {}", repo_url))?;

    let revision_str = revision.unwrap_or("main");
    let result = git_to_revision(target_dir, "origin", revision_str);
    if let Err(e) = result {
        // Cleanup! If we cannot checkout the revision, remove the directory.
        fs::remove_dir_all(target_dir)
            .with_context(|| format!("Failed to remove directory {}", target_dir.display()))?;
        return Err(e);
    }
    Ok(())
}

pub fn git_update(repo_path: &Path, repo_url: &str, revision: Option<&str>) -> Result<()> {
    if !repo_path.is_dir() {
        return Err(anyhow!(
            "Error with updating. {} is not a directory",
            repo_path.display()
        ));
    }

    // To make this operation atomic, we'll satisfy the remote & revision in this sequence:
    // 1.) add the remote URL as a new temporary remote.
    // 2.) check if the revision exists in the temporary remote.
    // 3.) checkout the revision from temporary remote
    // 4.) On success:
    //      4.1) replace the origin remote URL
    //      4.2) remove the temporary remote
    // 5.) On error, remove temporary remote
    //
    // Note that this sequence works even if the directory is already on that remote & revision.
    //
    let tmp_remote_name = random_remote_name();

    // Create a temporary remote
    safe_command(
        format!("git remote add \"{}\" \"{}\"", tmp_remote_name, repo_url),
        repo_path,
    )?
    .current_dir(repo_path)
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Error with adding {} as a remote named {} in {}",
            repo_url,
            tmp_remote_name,
            repo_path.display()
        )
    })?;

    let revision_str = revision.unwrap_or("main");
    let res = git_to_revision(repo_path, &tmp_remote_name, revision_str);

    if let Err(e) = res {
        // Failed to switch to the desired revision. Cleanup!
        safe_command(format!("git remote rm \"{}\"", &tmp_remote_name), repo_path)?
            .stdout(Stdio::null())
            .status()
            .with_context(|| {
                format!(
                    "Failed to remove temporary remote {} in {}",
                    tmp_remote_name,
                    repo_path.display()
                )
            })?;
        return Err(e);
    }

    safe_command(format!("git remote set-url origin \"{}\"", repo_url), repo_path)?
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to set origin remote to {} in {}",
            repo_url,
            repo_path.display()
        )
    })?;
    safe_command(format!("git remote rm \"{}\"", tmp_remote_name), repo_path)?
        .stdout(Stdio::null())
        .status()
        .with_context(|| {
            format!(
                "Failed to remove temporary remote {} in {}",
                tmp_remote_name,
                repo_path.display()
            )
        })?;
    return Ok(());
}

fn random_remote_name() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen();
    format!("tinty-remote-{}", random_number)
}

// Resolvees the SHA1 of revision at remote_name.
// revision can be a tag, a branch, or a commit SHA1.
fn git_resolve_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<String> {
    // 1.) Check if its a tag.
    let expected_tag_ref = format!("refs/tags/{}", revision);
    let mut command = safe_command(
        format!(
            "git ls-remote --quiet --tags \"{}\" \"{}\"",
            remote_name, expected_tag_ref
        ),
        repo_path,
    )?;
    let mut child = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn"))?;

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let parts: Vec<&str> = line.split("\t").collect();
                if parts.len() != 2 {
                    return Err(anyhow!(
                        "malformed ls-remote result. Expected tab-delimited tuple, found {} parts",
                        parts.len()
                    ));
                }
                // To hedge against non-exact matches, we'll compare the ref field with
                // what we'd expect an exact match would look i.e. refs/tags/<TAG_NAME>
                if parts[1] == expected_tag_ref {
                    // Found the tag. Return the SHA1
                    return Ok(parts[0].to_string());
                }
            }
            Err(e) => return Err(anyhow!("failed to capture lines: {}", e)),
        }
    }
    child
        .wait()
        .with_context(|| format!("Failed to list remote tags from {}", remote_name))?;

    // 2.) Check if its a branch
    let expected_branch_ref = format!("refs/heads/{}", revision);
    let mut command = safe_command(
        format!(
            "git ls-remote --quiet --branches \"{}\" \"{}\"",
            remote_name, expected_branch_ref
        ),
        repo_path,
    )?;
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn"))?;

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let parts: Vec<&str> = line.split("\t").collect();
                if parts.len() != 2 {
                    return Err(anyhow!(
                        "malformed ls-remote result. Expected tab-delimited tuple, found {} parts",
                        parts.len()
                    ));
                }
                // To hedge against non-exact matches, we'll compare the ref field with
                // what we'd expect an exact match would look i.e. refs/heads/<BRANCH_NAME>
                if parts[1] == expected_branch_ref {
                    // Found the tag. Return the SHA1
                    return Ok(parts[0].to_string());
                }
            }
            Err(e) => return Err(anyhow!("failed to capture lines: {}", e)),
        }
    }

    child
        .wait()
        .with_context(|| format!("Failed to list branches tags from {}", remote_name))?;

    // We are here because revision isn't a tag or a branch.
    // First, we'll check if revision itself *could* be a SHA1.
    // If it doesn't look like one, we'll return early.
    let pattern = r"^[0-9a-f]{1,40}$";
    let re = Regex::new(pattern).expect("Invalid regex");
    if !re.is_match(revision.as_bytes()) {
        return Err(anyhow!("cannot resolve {} into a Git SHA1", revision));
    }

    safe_command(format!("git fetch --quiet \"{}\"", remote_name), repo_path)?
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("unable to fetch objects from remote {}", remote_name))?;

    // 3.) Check if any branch in remote contains the SHA1:
    // It seems that the only way to do this is to list the branches that contain the SHA1
    // and check if it belongs in the remote.
    let remote_branch_prefix = format!("refs/remotes/{}/", remote_name);
    let mut command = safe_command(
        format!(
            "git branch --format=\"%(refname)\" -a --contains \"{}\"",
            revision
        ),
        repo_path,
    )?;
    let mut child = command.stdout(Stdio::piped()).spawn().with_context(|| {
        format!(
            "Failed to find branches containing commit {} from {}",
            revision, remote_name
        )
    })?;
    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.clone().starts_with(&remote_branch_prefix) {
                    // Found a branch
                    return Ok(revision.to_string());
                }
            }
            Err(e) => return Err(anyhow!("failed to capture lines: {}", e)),
        }
    }

    child.wait().with_context(|| {
        format!(
            "Failed to list branches from {} containing SHA1 {}",
            remote_name, revision
        )
    })?;

    return Err(anyhow!(
        "cannot find revision {} in remote {}",
        revision,
        remote_name
    ));
}

fn safe_command(command: String, cwd: &Path) -> Result<Command, Error> {
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;
    let mut command = Command::new(&command_vec[0]);
    command.args(&command_vec[1..]).current_dir(cwd);
    Ok(command)
}

fn git_to_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<()> {
    // Download the object from the remote
    safe_command(
        format!("git fetch --quiet \"{}\" \"{}\"", remote_name, revision),
        repo_path,
    )?
    .status()
    .with_context(|| {
        format!(
            "Error with fetching revision {} in {}",
            revision,
            repo_path.display()
        )
    })?;

    // Normalize the revision into the SHA.
    let commit_sha = git_resolve_revision(repo_path, remote_name, revision)?;

    safe_command(
        format!(
            "git -c advice.detachedHead=false checkout --quiet \"{}\"",
            commit_sha
        ),
        repo_path,
    )?
    .stdout(Stdio::null())
    .current_dir(repo_path)
    .status()
    .with_context(|| {
        format!(
            "Failed to checkout SHA {} in {}",
            commit_sha,
            repo_path.display()
        )
    })?;

    Ok(())
}

pub fn git_is_working_dir_clean(target_dir: &Path) -> Result<bool> {

    // We use the Git plumbing diff-index command to tell us of files that has changed,
    // both staged and unstaged.
    let status = safe_command("git diff-index --quiet HEAD --".to_string(), target_dir)?
        .status()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;

    // With the --quiet flag, it will return a 0 exit-code if no files has changed.
    Ok(!status.success())
}

pub fn create_theme_filename_without_extension(item: &ConfigItem) -> Result<String> {
    Ok(format!(
        "{}-{}-file",
        item.name.clone(),
        item.themes_dir.clone().replace('/', "-"), // Flatten path/to/dir to path-to-dir
    ))
}

pub fn get_all_scheme_names(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<Vec<String>> {
    if !schemes_path.exists() {
        return Err(anyhow!(
            "Schemes do not exist, run install and try again: `{} install`",
            REPO_NAME
        ));
    }

    // For each supported scheme system, add schemes to vec
    let mut scheme_vec: Vec<String> = Vec::new();
    let scheme_systems = scheme_systems_option
        .map(|s| vec![s])
        .unwrap_or(SchemeSystem::variants().to_vec());
    for scheme_system in scheme_systems {
        let scheme_system_dir = schemes_path.join(scheme_system.as_str());
        if !scheme_system_dir.exists() {
            continue;
        }

        for file in fs::read_dir(&scheme_system_dir)? {
            let file_path = file.as_ref().unwrap().path();
            let extension = file_path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();

            if extension == SCHEME_EXTENSION {
                scheme_vec.push(format!(
                    "{}-{}",
                    scheme_system.as_str(),
                    file.unwrap()
                        .path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                ));
            }
        }
    }

    scheme_vec.sort();

    Ok(scheme_vec)
}
pub fn replace_tilde_slash_with_home(path_str: &str) -> Result<PathBuf> {
    let trimmed_path_str = path_str.trim();
    if trimmed_path_str.starts_with("~/") {
        match home_dir() {
            Some(home_dir) => Ok(PathBuf::from(trimmed_path_str.replacen(
                "~/",
                format!("{}/", home_dir.display()).as_str(),
                1,
            ))),
            None => Err(anyhow!("Unable to determine a home directory for \"{}\", please use an absolute path instead", trimmed_path_str))
        }
    } else {
        Ok(PathBuf::from(trimmed_path_str))
    }
}
