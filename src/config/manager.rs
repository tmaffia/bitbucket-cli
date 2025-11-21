use anyhow::{Context, Result};
use config::{Config, FileFormat};
use dirs;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub default_profile: Option<String>,
    pub profiles: Option<std::collections::HashMap<String, ProfileConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProfileConfig {
    pub workspace: String,
    pub user: Option<String>,
    pub repository: Option<String>,
    pub api_url: Option<String>,
    pub output_format: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let mut builder = Config::builder();

        // 1. Global config: ~/.config/bb-cli/config.toml
        if let Some(config_dir) = get_config_dir() {
            let global_config_path = config_dir
                .join(crate::constants::CONFIG_DIR_NAME)
                .join(crate::constants::CONFIG_FILE_NAME);
            if global_config_path.exists() {
                builder = builder
                    .add_source(config::File::from(global_config_path).format(FileFormat::Toml));
            }
        }

        // 2. Local config: Walk up from current directory looking for .bb-cli file
        let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;
        let mut config_found = false;

        loop {
            let local_config_path = current_dir.join(crate::constants::LOCAL_CONFIG_FILE_NAME);
            if local_config_path.exists() {
                builder = builder
                    .add_source(config::File::from(local_config_path).format(FileFormat::Toml));
                config_found = true;
                // We found a config, but we might want to continue walking up?
                // Usually local overrides global, and closer overrides further.
                // Config crate merges sources. If we add them in order, later ones override earlier ones.
                // So we should probably collect all .bb-cli files from root to current dir?
                // Or just take the nearest one?
                // Requirement says "Repo-local configuration... overrides global settings".
                // Usually this means the one in the repo root.
                // For simplicity and typical behavior, we'll just take the nearest one we find.
                break;
            }

            if !current_dir.pop() {
                break;
            }
        }

        // If we didn't find one by walking up, check if we are in a git repo and check the root
        if !config_found {
            // Assuming `crate::git` exists and provides `get_repo_info`
            // If `crate::git` is not defined, this will cause a compilation error.
            if let Ok(_repo_info) = crate::git::get_repo_info() {
                // This implies we are in a git repo, but get_repo_info doesn't give us the root path.
                // We might want a helper to get the repo root.
                // For now, the walk-up loop should cover it if .bb-cli is at the repo root.
            }
        }

        let config = builder.build().context("Failed to build configuration")?;

        let app_config: AppConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        Ok(app_config)
    }
}

pub fn get_config_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join(".config"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        dirs::config_dir()
    }
}

pub fn update_global_user(username: &str) -> Result<()> {
    if let Some(config_dir) = get_config_dir() {
        let config_dir = config_dir.join(crate::constants::CONFIG_DIR_NAME);
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join(crate::constants::CONFIG_FILE_NAME);

        let mut config_content = String::new();
        if config_path.exists() {
            config_content = std::fs::read_to_string(&config_path)?;
        }

        let mut doc = config_content
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_default();

        let profile = doc
            .entry("profile")
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
        if let toml_edit::Item::Table(profile_table) = profile {
            profile_table.insert(
                "user",
                toml_edit::Item::Value(toml_edit::Value::from(username)),
            );
        }

        std::fs::write(&config_path, doc.to_string())?;
        println!("Updated configuration at: {:?}", config_path);
    }
    Ok(())
}
