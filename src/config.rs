use crate::constants::REPO_NAME;
use anyhow::{anyhow, Context, Result};
use home::home_dir;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;
use tinted_builder::SchemeSystem;
use url::Url;

pub const DEFAULT_CONFIG_SHELL: &str = "sh -c '{}'";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const ORG_NAME: &str = "tinted-theming";
pub const BASE16_SHELL_REPO_URL: &str = "https://github.com/tinted-theming/tinted-shell";
pub const BASE16_SHELL_REPO_NAME: &str = "tinted-shell";
pub const BASE16_SHELL_THEMES_DIR: &str = "scripts";
pub const BASE16_SHELL_HOOK: &str = ". %f";

/// Structure for configuration apply items
#[derive(Deserialize, Debug)]
pub struct ConfigItem {
    pub name: String,
    pub path: String,
    pub hook: Option<String>,
    #[serde(rename = "themes-dir")]
    pub themes_dir: String,
    #[serde(rename = "supported-systems")]
    pub supported_systems: Option<Vec<SchemeSystem>>,
    #[serde(rename = "theme-file-extension")]
    pub theme_file_extension: Option<String>,
    pub revision: Option<String>,
}

impl fmt::Display for ConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hook = self.hook.clone().unwrap_or_default();
        let revision = self.revision.clone().unwrap_or_default();
        let default_supported_systems = vec![SchemeSystem::default()];
        let system_text = self
            .supported_systems
            .clone()
            .unwrap_or(default_supported_systems)
            .into_iter()
            .map(|system| format!("\"{}\"", system))
            .collect::<Vec<String>>()
            .join(", ");

        // You can format the output however you like
        writeln!(f)?;
        writeln!(f, "[[items]]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        writeln!(f, "path = \"{}\"", self.path)?;
        if !hook.is_empty() {
            writeln!(f, "hook = \"{}\"", hook)?;
        }
        if !revision.is_empty() {
            writeln!(f, "revision = \"{}\"", revision)?;
        }
        writeln!(f, "supported-systems = [{}]", system_text)?;
        write!(f, "themes-dir = \"{}\"", self.themes_dir)
    }
}

/// Structure for configuration
#[derive(Deserialize, Debug)]
pub struct Config {
    pub shell: Option<String>,
    #[serde(rename = "default-scheme")]
    pub default_scheme: Option<String>,
    pub items: Option<Vec<ConfigItem>>,
    pub hooks: Option<Vec<String>>,
}

fn ensure_item_name_is_unique(items: &[ConfigItem]) -> Result<()> {
    let mut names = HashSet::new();

    for item in items.iter() {
        if !names.insert(&item.name) {
            return Err(anyhow!("config.toml item.name should be unique values, but \"{}\" is used for more than 1 item.name. Please change this to a unique value.", item.name));
        }
    }

    Ok(())
}

impl Config {
    pub fn read(path: &Path) -> Result<Config> {
        if path.exists() && !path.is_file() {
            return Err(anyhow!(
                "The provided config path is a directory and not a file: {}",
                path.display()
            ));
        }

        let contents = fs::read_to_string(path).unwrap_or(String::from(""));
        let mut config: Config = toml::from_str(contents.as_str()).with_context(|| {
            format!(
                "Couldn't parse {} configuration file ({:?}). Check if it's syntactically correct",
                REPO_NAME, path
            )
        })?;

        // Create default `item`
        let shell = config
            .shell
            .clone()
            .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.into());
        let base16_shell_config_item = ConfigItem {
            path: BASE16_SHELL_REPO_URL.to_string(),
            name: BASE16_SHELL_REPO_NAME.to_string(),
            themes_dir: BASE16_SHELL_THEMES_DIR.to_string(),
            hook: Some(BASE16_SHELL_HOOK.to_string()),
            supported_systems: Some(vec![SchemeSystem::Base16]), // DEFAULT_SCHEME_SYSTEM
            theme_file_extension: None,
            revision: None,
        };

        // Add default `item` if no items exist
        match config.items.as_ref() {
            Some(items) => {
                ensure_item_name_is_unique(items)?;
            }
            None => {
                config.items = Some(vec![base16_shell_config_item]);
            }
        }

        // Set default `system` property for missing systems
        if let Some(ref mut items) = config.items {
            for item in items.iter_mut() {
                if item.supported_systems.is_none() {
                    item.supported_systems = Some(vec![SchemeSystem::default()]);
                }

                // Replace `~/` with absolute home path
                let trimmed_path = item.path.trim();
                if trimmed_path.starts_with("~/") {
                    match home_dir() {
                        Some(home_dir) => {
                            item.path = trimmed_path.replacen(
                                "~/",
                                format!("{}/", home_dir.display()).as_str(),
                                1,
                            );
                        }
                        None => {
                            return Err(anyhow!("Unable to determine a home directory for \"{}\", please use an absolute path instead", item.path));
                        }
                    }
                }

                // Return Err if path is not a valid url or an existing directory path
                if Url::parse(item.path.as_str()).is_err()
                    && !Path::new(item.path.as_str()).is_dir()
                {
                    return Err(anyhow!("One of your config.toml items has an invalid `path` value. \"{}\" is not a valid url and is not a path to an existing local directory", item.path));
                }
            }
        }

        if !shell.contains("{}") {
            let msg = "The configured shell does not contain the required command placeholder '{}'. Check the default file or github for config examples.";
            return Err(anyhow!(msg));
        }

        config.shell = Some(shell);

        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "shell = \"{}\"",
            self.shell.as_ref().unwrap_or(&"None".to_string())
        )?;
        if self.default_scheme.is_some() {
            writeln!(
                f,
                "default-scheme = \"{}\"",
                self.default_scheme.as_ref().unwrap_or(&"".to_string())
            )?;
        }

        match &self.items {
            Some(items) => {
                for item in items {
                    writeln!(f, "{}", item)?;
                }
            }
            None => {}
        }

        Ok(())
    }
}
