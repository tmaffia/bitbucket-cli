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

use crate::context::AppContext;

pub async fn handle(_ctx: &AppContext, args: ConfigArgs) -> Result<()> {
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
            let config = crate::config::manager::ProfileConfig::load()?;
            println!("{:#?}", config);
        }
        ConfigCommands::Set { key, value } => {
            crate::config::manager::set_config_value(&key, &value)?;
            ui::success(&format!("Set {} = {}", key, value));
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
                    "default_profile" => {
                        println!("{}", config.default_profile.as_deref().unwrap_or("Not set"))
                    }
                    "user" => println!(
                        "{}",
                        p.and_then(|prof| prof.user.as_deref()).unwrap_or("Not set")
                    ),
                    "workspace" => {
                        println!(
                            "{}",
                            p.and_then(|prof| prof.workspace.as_deref())
                                .unwrap_or("Not set")
                        )
                    }
                    "api_url" => {
                        println!(
                            "{}",
                            p.and_then(|prof| prof.api_url.as_deref())
                                .unwrap_or("Not set")
                        )
                    }
                    "output_format" => {
                        println!(
                            "{}",
                            p.and_then(|prof| prof.output_format.as_deref())
                                .unwrap_or("Not set")
                        )
                    }
                    _ => {
                        ui::error(&format!("Unknown key: '{}'", key));
                        ui::info(
                            "Valid keys: default_profile, workspace, user, api_url, output_format",
                        );
                    }
                },
                None => {
                    println!("Current Profile Settings:");
                    println!(
                        "  Default Profile: {}",
                        config.default_profile.as_deref().unwrap_or("Not set")
                    );
                    if let Some(profile) = p {
                        println!("  User: {}", profile.user.as_deref().unwrap_or("Not set"));
                        println!(
                            "  Workspace: {}",
                            profile.workspace.as_deref().unwrap_or("Not set")
                        );
                        println!(
                            "  API URL: {}",
                            profile.api_url.as_deref().unwrap_or("Not set")
                        );
                        println!(
                            "  Output Format: {}",
                            profile.output_format.as_deref().unwrap_or("Not set")
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
