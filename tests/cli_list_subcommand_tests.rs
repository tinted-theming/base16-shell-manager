mod utils;

use crate::utils::REPO_NAME;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tinted_builder::{SchemeSystem, SchemeVariant};
use utils::setup;

const SCHEME_COUNT: usize = 287;

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_list_subcommand_without_setup", "list")?;
    let expected_output = format!(
        "Schemes are missing, run install and then try again: `{} install`",
        REPO_NAME
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_without_setup_with_custom_schemes_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let test_name = "test_cli_list_subcommand_without_setup_with_custom_schemes_flag";
    let (_, _, command_vec, cleanup) = setup(test_name, "list --custom-schemes")?;
    let expected_output = format!(
        "You don't have any local custom schemes at: data_path_{}/custom-schemes",
        test_name
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_setup", "list")?;
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_with_custom() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_custom", "list")?;
    let scheme_system = "base16";
    let scheme_name_one = "tinted-theming";
    let scheme_name_two = "tinty";
    let expected_output = format!(
        "{}-{}\n{}-{}",
        scheme_system, scheme_name_one, scheme_system, scheme_name_two
    );
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::write(
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_one)),
        "",
    )?;
    fs::write(
        custom_scheme_path.join(format!("{}/{}.yml", scheme_system, scheme_name_two)),
        "",
    )?;

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup()?;
    Ok(())
}

// These structs may appear as duplicate of the serializable structs in operations/list.rs, but
// this is intentional. These are asserting the expected structure of the JSON output, and
// sharing the exact same struct would hide a certain class of issues (i.e. comparing a broken
// implementation w/ an equally-broken assertion)
#[derive(Clone, Deserialize, PartialEq)]
struct TestSchemeEntry {
    pub id: String,
    pub name: String,
    pub author: String,
    pub system: SchemeSystem,
    pub variant: SchemeVariant,
    pub slug: String,
    pub palette: HashMap<String, TestColorOut>,
    pub lightness: Option<TestLightness>,
}

impl std::ops::Deref for TestSchemeEntry {
    type Target = SchemeSystem;

    fn deref(&self) -> &Self::Target {
        &self.system
    }
}

impl std::fmt::Debug for TestSchemeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSchemeEntry")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("author", &self.author)
            .field("system", &self.system)
            .field("variant", &self.variant)
            .field("slug", &self.slug)
            .field("palette", &self.palette)
            .field("lightness", &self.lightness)
            .finish()
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct TestColorOut {
    pub hex_str: String,
    pub hex: (String, String, String),
    pub rgb: (u8, u8, u8),
    pub dec: (f32, f32, f32),
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct TestLightness {
    pub foreground: f32,
    pub background: f32,
}

// This is the basis of the other as_json tests. We'll assert that TestSchemeEntry are
// deserializing fields as expected.
#[test]
fn test_cli_list_subcommand_deserialize_fixture_scheme_entry() -> Result<()> {
    // -------
    // Arrange
    // -------
    let github_json = fs::read_to_string(Path::new("fixtures/github.json"))?;
    // ---
    // Act
    // ---
    let github: TestSchemeEntry = serde_json::from_str(&github_json).unwrap();

    assert!(
        github.name == "Github",
        "Expected name to be Github, got {}",
        github.name
    );
    assert!(
        github.system == SchemeSystem::Base16,
        "Expected system to be base16, got {}",
        github.system
    );
    assert!(
        github.variant == SchemeVariant::Light,
        "Expected variant to be base16, got {}",
        github.variant
    );
    assert!(
        github.author == "Defman21",
        "Expected wauthorname to be 'Defman21', got {}",
        github.author
    );
    assert!(
        github.slug == "github",
        "Expected slug to be github, got {}",
        github.slug
    );
    assert!(
        github.id == "base16-github",
        "Expected id to be base16-github, got {}",
        github.id
    );
    assert!(
        github.palette.len() == 16,
        "Expected 16 colors in palette, got {}",
        github.palette.len()
    );

    let expected_colors = vec![
        (
            "base00",
            "#ffffff",
            ("ff".to_string(), "ff".to_string(), "ff".to_string()),
            (1.0, 1.0, 1.0),
            (255, 255, 255),
        ),
        (
            "base01",
            "#f5f5f5",
            ("f5".to_string(), "f5".to_string(), "f5".to_string()),
            (0.9607843, 0.9607843, 0.9607843),
            (245, 245, 245),
        ),
        (
            "base02",
            "#c8c8fa",
            ("c8".to_string(), "c8".to_string(), "fa".to_string()),
            (0.78431374, 0.78431374, 0.98039216),
            (200, 200, 250),
        ),
        (
            "base03",
            "#969896",
            ("96".to_string(), "98".to_string(), "96".to_string()),
            (0.5882353, 0.59607846, 0.5882353),
            (150, 152, 150),
        ),
        (
            "base04",
            "#e8e8e8",
            ("e8".to_string(), "e8".to_string(), "e8".to_string()),
            (0.9098039, 0.9098039, 0.9098039),
            (232, 232, 232),
        ),
        (
            "base05",
            "#333333",
            ("33".to_string(), "33".to_string(), "33".to_string()),
            (0.2, 0.2, 0.2),
            (51, 51, 51),
        ),
        (
            "base06",
            "#ffffff",
            ("ff".to_string(), "ff".to_string(), "ff".to_string()),
            (1.0, 1.0, 1.0),
            (255, 255, 255),
        ),
        (
            "base07",
            "#ffffff",
            ("ff".to_string(), "ff".to_string(), "ff".to_string()),
            (1.0, 1.0, 1.0),
            (255, 255, 255),
        ),
        (
            "base08",
            "#ed6a43",
            ("ed".to_string(), "6a".to_string(), "43".to_string()),
            (0.92941177, 0.41568628, 0.2627451),
            (237, 106, 67),
        ),
        (
            "base09",
            "#0086b3",
            ("00".to_string(), "86".to_string(), "b3".to_string()),
            (0.0, 0.5254902, 0.7019608),
            (0, 134, 179),
        ),
        (
            "base0A",
            "#795da3",
            ("79".to_string(), "5d".to_string(), "a3".to_string()),
            (0.4745098, 0.3647059, 0.6392157),
            (121, 93, 163),
        ),
        (
            "base0B",
            "#183691",
            ("18".to_string(), "36".to_string(), "91".to_string()),
            (0.09411765, 0.21176471, 0.5686275),
            (24, 54, 145),
        ),
        (
            "base0C",
            "#183691",
            ("18".to_string(), "36".to_string(), "91".to_string()),
            (0.09411765, 0.21176471, 0.5686275),
            (24, 54, 145),
        ),
        (
            "base0D",
            "#795da3",
            ("79".to_string(), "5d".to_string(), "a3".to_string()),
            (0.4745098, 0.3647059, 0.6392157),
            (121, 93, 163),
        ),
        (
            "base0E",
            "#a71d5d",
            ("a7".to_string(), "1d".to_string(), "5d".to_string()),
            (0.654902, 0.11372549, 0.3647059),
            (167, 29, 93),
        ),
        (
            "base0F",
            "#333333",
            ("33".to_string(), "33".to_string(), "33".to_string()),
            (0.2, 0.2, 0.2),
            (51, 51, 51),
        ),
    ];

    for (color, expected_hex_str, expected_hex, expected_dec, expected_rgb) in expected_colors {
        let palette_color = github
            .palette
            .get(color)
            .context(format!("color {} not found", color))?;
        assert!(
            palette_color.hex_str == expected_hex_str,
            "Exoected {}.hex_str to equal {}, got {}",
            color,
            expected_hex_str,
            palette_color.hex_str
        );
        assert!(
            palette_color.hex == expected_hex,
            "Exoected {}.hex to equal {}, got {}",
            color,
            expected_hex_str,
            palette_color.hex_str
        );
        assert!(
            palette_color.dec == expected_dec,
            "Exoected {}.dec to equal ({}, {}, {}), got ({}, {}, {})",
            color,
            expected_dec.0,
            expected_dec.1,
            expected_dec.2,
            palette_color.dec.0,
            palette_color.dec.1,
            palette_color.dec.2,
        );
        assert!(
            palette_color.rgb == expected_rgb,
            "Exoected {}.rgb to equal ({}, {}, {}), got ({}, {}, {})",
            color,
            expected_rgb.0,
            expected_rgb.1,
            expected_rgb.2,
            palette_color.rgb.0,
            palette_color.rgb.1,
            palette_color.rgb.2,
        );
    }

    let (foreground, background) = github
        .lightness
        .map(|l| (l.foreground, l.background))
        .unwrap();

    assert!(
        background == 100.0,
        "Expected lightness.background to be 100, got {}",
        background
    );
    assert!(
        foreground == 21.246727,
        "Expected lightness.foreground to be 21.246727, got {}",
        foreground
    );

    Ok(())
}

#[test]
fn test_cli_list_subcommand_as_json_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_as_json_with_setup", "list --json")?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();

    assert!(
        results.len() == SCHEME_COUNT,
        "expected JSON to contain {} entries, found {}",
        SCHEME_COUNT,
        results.len()
    );

    let entry_map: HashMap<String, TestSchemeEntry> =
        results.into_iter().map(|e| (e.id.clone(), e)).collect();

    let github = entry_map.get("base16-github").unwrap().clone();
    let gruvbox = entry_map
        .get("base16-gruvbox-material-dark-hard")
        .unwrap()
        .clone();

    let github_json = fs::read_to_string(Path::new("fixtures/github.json"))?;
    let gruvbox_json = fs::read_to_string(Path::new("fixtures/gruvbox-material-dark-hard.json"))?;

    let expected_github: TestSchemeEntry = serde_json::from_str(&github_json).unwrap();
    let expected_gruvbox: TestSchemeEntry = serde_json::from_str(&gruvbox_json).unwrap();

    assert!(
        expected_github == github,
        "{:?} does not match {:?}",
        expected_github,
        github
    );
    assert!(
        expected_gruvbox == gruvbox,
        "{:?} does not match {:?}",
        expected_gruvbox,
        gruvbox
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_as_json_with_custom() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_list_subcommand_as_json_with_custom",
        "list --json",
    )?;
    let scheme_system = "base16";
    let scheme_name_one = "tinted-theming";
    let scheme_name_two = "tinty";
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::write(
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_one)),
        "",
    )?;

    fs::copy(
        Path::new("fixtures/tinty-city-dark.yaml"),
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_two)),
    )
    .context("failed to copy scheme from fixtures")?;

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    let expected_json = fs::read_to_string(Path::new("fixtures/tinty-city-dark.json"))?;
    let expected_entry: TestSchemeEntry = serde_json::from_str(&expected_json).unwrap();

    // ------
    // Assert
    // ------
    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();
    // There are two YAMLs in the directory of custom schemes but only one of them
    // contains a YAML body
    assert!(
        results.len() == 1,
        "expected JSON to contain 1 entry, found {}",
        results.len()
    );
    assert!(
        expected_entry == results[0],
        "{:?} does not match {:?}",
        expected_entry,
        results[0],
    );

    cleanup()?;
    Ok(())
}
