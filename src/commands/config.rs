use anyhow::Result;
use clap::{Args, Subcommand};

use crate::display::ui;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize configuration
    Init,
    /// List configuration
    List,
    /// Set configuration value
    Set { key: String, value: String },
    /// Get configuration value (or entire config if no key specified)
    Get { key: Option<String> },
}

use crate::context::AppContext;

pub async fn handle(ctx: &AppContext, args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Init => {
            crate::config::setup::interactive_init()?;
        }
        ConfigCommands::List => {
            let config = crate::config::manager::ProfileConfig::load_global().unwrap_or_default();
            let repo_root = crate::git::get_repo_root().ok();
            let local_config =
                crate::config::manager::ProfileConfig::load_local(repo_root.as_deref())?;

            // Resolve active values
            let active_profile = config.get_active_profile();

            let mut config_values = Vec::new();

            // Helper to add values if present
            let mut add_val = |key: &str, val: Option<String>| {
                if let Some(v) = val {
                    config_values.push((key.to_string(), v));
                }
            };

            // 1. User (Global only)
            add_val("user", active_profile.and_then(|p| p.user.clone()));

            // 2. Workspace (Local > Global)
            let workspace = local_config
                .as_ref()
                .and_then(|c| c.project.as_ref())
                .and_then(|p| p.workspace.clone())
                .or_else(|| active_profile.and_then(|p| p.workspace.clone()));
            add_val("workspace", workspace);

            // 3. Repository (Local only)
            let repo = local_config
                .as_ref()
                .and_then(|c| c.project.as_ref())
                .and_then(|p| p.repository.clone());
            add_val("repository", repo);

            // 4. Remote (Local only)
            let remote = local_config
                .as_ref()
                .and_then(|c| c.project.as_ref())
                .and_then(|p| p.remote.clone());
            add_val("remote", remote);

            if ctx.json {
                let mut map = serde_json::Map::new();
                for (k, v) in config_values {
                    map.insert(k, serde_json::Value::String(v));
                }
                ui::print_json(&map)?;
            } else {
                for (k, v) in config_values {
                    println!("{}={}", k, v);
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            // Context-aware setting
            // If key is "user", set global user.
            // If key is "workspace", "repository", "remote", set it for the ACTIVE profile.
            // Otherwise, set as provided (full key).

            let real_key = if key == "user" {
                key
            } else if ["workspace", "repository", "remote"].contains(&key.as_str()) {
                let config =
                    crate::config::manager::ProfileConfig::load_global().unwrap_or_default();
                // If no active profile (user) is set, default to "default"
                let profile_name = config.user.as_deref().unwrap_or("default");
                format!("profile.{}.{}", profile_name, key)
            } else {
                key
            };

            crate::config::manager::set_config_value(&real_key, &value)?;
            ui::success(&format!("Set {} = {}", real_key, value));
        }
        ConfigCommands::Get { key } => {
            let config = crate::config::manager::ProfileConfig::load()?;

            // If no key provided, show full config
            if key.is_none() || key.as_ref().is_none_or(|s| s.is_empty()) {
                println!("{:#?}", config);
                return Ok(());
            }
            let p = config.get_active_profile();

            match key {
                Some(key) => match key.as_str() {
                    "user" => println!("{}", config.user.as_deref().unwrap_or("Not set")),
                    "workspace" => {
                        println!(
                            "{}",
                            p.and_then(|prof| prof.workspace.as_deref())
                                .unwrap_or("Not set")
                        )
                    }
                    _ => {
                        ui::error(&format!("Unknown key: '{}'", key));
                        ui::info("Valid keys: user, workspace");
                    }
                },
                None => {
                    println!("Current Profile Settings:");
                    println!("  User: {}", config.user.as_deref().unwrap_or("Not set"));
                    if let Some(profile) = p {
                        println!(
                            "  Workspace: {}",
                            profile.workspace.as_deref().unwrap_or("Not set")
                        );
                    } else {
                        ui::warning("No active profile found.");
                    }
                }
            }
        }
    }
    Ok(())
}
