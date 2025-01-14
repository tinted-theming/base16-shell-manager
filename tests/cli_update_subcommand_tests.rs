mod utils;

use std::process::Command;

use crate::utils::{setup, REPO_NAME};
use anyhow::Result;
use utils::write_to_file;

#[test]
fn test_cli_update_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_non_unique_config_item_name",
        "update",
    )?;
    let expected_output = format!("tinted-shell not installed (run `{} install`)", REPO_NAME);

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_setup", "update")?;
    let expected_output = "tinted-shell up to date";

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_setup_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_setup_quiet_flag",
        "update --quiet",
    )?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_new_remote", "update")?;
    let expected_output = "tinted-jqp up to date";

    let config_content = r##"[[items]]
path = "https://github.com/bezhermoso/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
"##;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;

    // Replace the remote with a new one
    let config_content = r##"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
"##;
    write_to_file(&config_path, config_content)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    let mut data_path = data_path.clone();
    data_path.push("repos");
    data_path.push("tinted-jqp");

    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&data_path)
        .output()?;

    let remote_stdout = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    let remote_url = "https://github.com/tinted-theming/tinted-jqp";
    let has_match = remote_stdout.lines().any(|line| line == remote_url);

    assert!(
        has_match,
        "Expected origin remote to point to URL {}",
        remote_url
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_new_revision", "update")?;
    let expected_output = "tinted-jqp up to date";

    let config_content = r##"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
"##;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // --- utils::run_install_command(&config_path, &data_path)?;

    // Replace the remote with a new one
    utils::run_install_command(&config_path, &data_path)?;

    let config_content = r##"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
revision = "tinty-test-tag-01"
"##;
    write_to_file(&config_path, config_content)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    let mut data_path = data_path.clone();
    data_path.push("repos");
    data_path.push("tinted-jqp");

    println!(
        "repo_path: {}, exists?: {}",
        data_path.display(),
        data_path.exists()
    );

    let output = Command::new("git")
        .args(vec!["rev-parse", "--verify", "HEAD"])
        .current_dir(&data_path)
        .output()?;

    let rev_parse_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain expected output"
    );
    let expected_revision = "b6c6a7803c2669022167c9cfc5efb3dc3928507d";
    let has_match = rev_parse_out.lines().any(|line| line == expected_revision);

    assert!(has_match, "Expected revision {}", expected_revision);

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote_but_invalid_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_new_remote_but_invalid_revision",
        "update",
    )?;
    let expected_output = "tinted-jqp up to date";
    let config_content = r##"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
"##;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;

    // Replace the remote with a new one
    let config_content = r##"[[items]]
path = "https://github.com/bezhermoso/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
revision = "invalid-revision"
"##;
    write_to_file(&config_path, config_content)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    let mut data_path = data_path.clone();
    data_path.push("repos");
    data_path.push("tinted-jqp");

    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&data_path)
        .output()?;

    let remote_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        !stdout.contains(expected_output),
        "stdout contains unexpected output"
    );
    assert!(
        stderr.contains("cannot resolve invalid-revision"),
        "stderr does not contain the expected output"
    );
    let expected_remote_url = "https://github.com/tinted-theming/tinted-jqp";
    let has_match = remote_out.lines().any(|line| line == expected_remote_url);

    assert!(has_match, "Expected remote URL {}", expected_remote_url);

    Ok(())
}
