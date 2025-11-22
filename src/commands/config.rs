use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::{self, Write};

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

pub async fn handle(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Init => {
            ui::info("Initializing config...");
            // Interactive setup
            let mut input = String::new();

            print!("Initialize configuration in current directory? (y/n) [n]: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            let local_init = input.trim().to_lowercase() == "y";

            input.clear();
            print!("Workspace (e.g., myworkspace): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            let workspace = input.trim().to_string();

            input.clear();
            print!("Default repository (optional): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            let repo = input.trim().to_string();

            input.clear();
            print!("Default remote [origin]: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            let input_remote = input.trim().to_string();
            let remote = if input_remote.is_empty() {
                "origin".to_string()
            } else {
                input_remote
            };

            if local_init {
                crate::config::manager::init_local_config(&workspace, &repo, &remote)?;
                ui::success("Local configuration initialized");
            } else {
                crate::config::manager::set_config_value("profile.default.workspace", &workspace)?;
                if !repo.is_empty() {
                    crate::config::manager::set_config_value("profile.default.repository", &repo)?;
                }
                crate::config::manager::set_config_value("profile.default.remote", &remote)?;

                input.clear();
                print!("Default user email (optional): ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut input)?;
                let user = input.trim().to_string();

                if !user.is_empty() {
                    crate::config::manager::set_config_value("profile.default.user", &user)?;
                }

                ui::success("Configuration initialized");
            }
        }
        ConfigCommands::List => {
            let config = crate::config::manager::AppConfig::load()?;
            println!("{:#?}", config);
        }
        ConfigCommands::Set { key, value } => {
            crate::config::manager::set_config_value(&key, &value)?;
            ui::success(&format!("Set {} = {}", key, value));
        }
        ConfigCommands::Get { key } => {
            let config = crate::config::manager::AppConfig::load()?;

            // If no key provided, show full config
            if key.is_none() || key.as_ref().map_or(true, |s| s.is_empty()) {
                println!("{:#?}", config);
                return Ok(());
            }

            let key = key.unwrap();

            // Match on the key to access the appropriate field
            let value = match key.as_str() {
                "default_profile" => config.default_profile.as_ref().map(|s| s.as_str()),
                "workspace" => config.get_active_profile().map(|p| p.workspace.as_str()),
                "user" => config
                    .get_active_profile()
                    .and_then(|p| p.user.as_ref().map(|s| s.as_str())),
                "repository" => config
                    .get_active_profile()
                    .and_then(|p| p.repository.as_ref().map(|s| s.as_str())),
                "api_url" => config
                    .get_active_profile()
                    .and_then(|p| p.api_url.as_ref().map(|s| s.as_str())),
                "output_format" => config
                    .get_active_profile()
                    .and_then(|p| p.output_format.as_ref().map(|s| s.as_str())),
                "remote" => config
                    .get_active_profile()
                    .and_then(|p| p.remote.as_ref().map(|s| s.as_str())),

                _ => {
                    ui::error(&format!("Unknown key: '{}'", key));
                    ui::info(
                        "Valid keys: default_profile, workspace, user, repository, api_url, output_format, remote",
                    );
                    return Ok(());
                }
            };

            match value {
                Some(v) => println!("{}", v),
                None => ui::warning("Key not found or not set"),
            }
        }
    }
    Ok(())
}
