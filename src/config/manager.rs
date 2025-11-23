use anyhow::{Context, Result};
use config::{Config, FileFormat};
use dirs;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProfileConfig {
    pub default_profile: Option<String>,
    #[serde(rename = "profile")]
    pub profiles: Option<std::collections::HashMap<String, Profile>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub workspace: String,
    pub user: Option<String>,
    pub repository: Option<String>,
    pub api_url: Option<String>,
    pub output_format: Option<String>,
    pub remote: Option<String>,
}

impl ProfileConfig {
    pub fn load() -> Result<Self> {
        let config = build_config()?;
        let app_config: ProfileConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;
        Ok(app_config)
    }

    pub fn get_active_profile(&self) -> Option<&Profile> {
        let profile_name = self.default_profile.as_deref().unwrap_or("default");
        self.profiles.as_ref().and_then(|p| p.get(profile_name))
    }

    pub fn get_default_user(&self) -> Option<String> {
        self.get_active_profile().and_then(|p| p.user.clone())
    }

    pub fn create_client(
        &self,
        profile_override: Option<&str>,
    ) -> Result<crate::api::client::BitbucketClient> {
        let profile_name = profile_override
            .or(self.default_profile.as_deref())
            .unwrap_or("default");

        let profile = self.profiles.as_ref().and_then(|p| p.get(profile_name));

        if let Some(p) = profile {
            crate::utils::debug::log(&format!("Profile loaded. User: {:?}", p.user));
        } else {
            crate::utils::debug::log(&format!("Profile '{}' NOT found in config.", profile_name));
        }

        let base_url = profile
            .and_then(|p| p.api_url.clone())
            .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

        let mut auth = None;
        if let Some(username) = profile.and_then(|p| p.user.as_ref()) {
            match crate::utils::auth::get_credentials(username) {
                Ok(api_token) => {
                    crate::utils::debug::log(&format!("Credentials found for user '{}'", username));
                    auth = Some((username.clone(), api_token));
                }
                Err(e) => {
                    crate::utils::debug::log(&format!(
                        "Failed to load credentials for user '{}': {}",
                        username, e
                    ));
                }
            }
        } else {
            crate::utils::debug::log("No user configured in profile. Running unauthenticated.");
        }

        crate::api::client::BitbucketClient::new(base_url, auth)
    }
}

fn build_config() -> Result<Config> {
    let mut builder = Config::builder();

    // 1. Global config: ~/.config/bb-cli/config.toml
    if let Some(config_dir) = get_config_dir() {
        let global_config_path = config_dir
            .join(crate::constants::CONFIG_DIR_NAME)
            .join(crate::constants::CONFIG_FILE_NAME);
        if global_config_path.exists() {
            builder =
                builder.add_source(config::File::from(global_config_path).format(FileFormat::Toml));
        }
    }

    // 2. Local config: Walk up from current directory looking for .bb-cli file
    let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let mut config_found = false;

    loop {
        let local_config_path = current_dir.join(crate::constants::LOCAL_CONFIG_FILE_NAME);
        if local_config_path.exists() {
            builder =
                builder.add_source(config::File::from(local_config_path).format(FileFormat::Toml));
            config_found = true;
            break;
        }

        if !current_dir.pop() {
            break;
        }
    }

    // If we didn't find one by walking up, check if we are in a git repo and check the root
    if !config_found {
        // Placeholder for git repo check if needed
    }

    builder.build().context("Failed to build configuration")
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

pub fn set_config_value(key: &str, value: &str) -> Result<()> {
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

        // Handle nested keys (e.g. profile.default.user)
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_table = doc.as_table_mut();

        // TODO Evaluate if mixing loop and iterator can be improved
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                current_table.insert(part, toml_edit::Item::Value(toml_edit::Value::from(value)));
            } else {
                // Intermediate part, navigate or create table
                let entry = current_table
                    .entry(part)
                    .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));

                if let toml_edit::Item::Table(t) = entry {
                    current_table = t;
                } else {
                    // It might be an inline table or something else.
                    // For simplicity, if it's not a table, we can't easily descend.
                    // But `or_insert` with Table should work for new entries.
                    // If it exists and is not a table, we have a conflict.
                    return Err(anyhow::anyhow!("Config key conflict at '{}'", part));
                }
            }
        }

        std::fs::write(&config_path, doc.to_string())?;
        // println!("Updated configuration at: {:?}", config_path);
    }
    Ok(())
}

pub fn init_local_config(workspace: &str, repo: &str, remote: &str) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let config_path = current_dir.join(crate::constants::LOCAL_CONFIG_FILE_NAME);

    if config_path.exists() {
        return Err(anyhow::anyhow!(
            "Local configuration file already exists at {:?}",
            config_path
        ));
    }

    let mut doc = toml_edit::DocumentMut::new();

    // Create [profile.default]
    let mut profile_default = toml_edit::Table::new();
    profile_default.insert(
        "workspace",
        toml_edit::Item::Value(toml_edit::Value::from(workspace)),
    );
    profile_default.insert(
        "repository",
        toml_edit::Item::Value(toml_edit::Value::from(repo)),
    );
    profile_default.insert(
        "remote",
        toml_edit::Item::Value(toml_edit::Value::from(remote)),
    );

    let mut profile = toml_edit::Table::new();
    profile.insert("default", toml_edit::Item::Table(profile_default));

    doc.insert("profile", toml_edit::Item::Table(profile));

    std::fs::write(&config_path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_active_profile_default() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                workspace: "ws".to_string(),
                user: Some("default_user".to_string()),
                repository: None,
                api_url: None,
                output_format: None,
                remote: None,
            },
        );

        let config = ProfileConfig {
            default_profile: None,
            profiles: Some(profiles),
        };

        let profile = config.get_active_profile();
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().workspace, "ws");
        assert_eq!(profile.unwrap().user.as_deref(), Some("default_user"));
    }

    #[test]
    fn test_get_active_profile_named() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "custom".to_string(),
            Profile {
                workspace: "custom_ws".to_string(),
                user: Some("custom_user".to_string()),
                repository: None,
                api_url: None,
                output_format: None,
                remote: None,
            },
        );

        let config = ProfileConfig {
            default_profile: Some("custom".to_string()),
            profiles: Some(profiles),
        };

        let profile = config.get_active_profile();
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().workspace, "custom_ws");
    }

    #[test]
    fn test_get_default_user() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                workspace: "ws".to_string(),
                user: Some("test_user".to_string()),
                repository: None,
                api_url: None,
                output_format: None,
                remote: None,
            },
        );

        let config = ProfileConfig {
            default_profile: None,
            profiles: Some(profiles),
        };

        let user = config.get_default_user();
        assert_eq!(user, Some("test_user".to_string()));
    }

    #[test]
    fn test_get_default_user_none() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                workspace: "ws".to_string(),
                user: None,
                repository: None,
                api_url: None,
                output_format: None,
                remote: None,
            },
        );

        let config = ProfileConfig {
            default_profile: None,
            profiles: Some(profiles),
        };

        let user = config.get_default_user();
        assert_eq!(user, None);
    }
}
