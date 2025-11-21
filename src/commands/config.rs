use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::{self, Write};

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
            println!("Initializing config...");
            // Interactive setup
            let mut input = String::new();

            print!("Default Workspace: ");
            io::stdout().flush()?;
            input.clear();
            io::stdin().read_line(&mut input)?;
            let workspace = input.trim();
            if !workspace.is_empty() {
                crate::config::manager::set_config_value("profile.default.workspace", workspace)?;
            }

            print!("Default User: ");
            io::stdout().flush()?;
            input.clear();
            io::stdin().read_line(&mut input)?;
            let user = input.trim();
            if !user.is_empty() {
                crate::config::manager::set_config_value("profile.default.user", user)?;
            }

            println!("Configuration initialized.");
        }
        ConfigCommands::List => {
            let config = crate::config::manager::AppConfig::load()?;
            println!("{:#?}", config);
        }
        ConfigCommands::Set { key, value } => {
            crate::config::manager::set_config_value(&key, &value)?;
            println!("Set {} = {}", key, value);
        }
        ConfigCommands::Get { key } => {
            let config = crate::config::manager::AppConfig::load()?;

            // If no key provided, show entire config
            if key.is_none() {
                println!("{:#?}", config);
                return Ok(());
            }

            let key = key.unwrap();

            // Match on simple, intuitive keys. Most keys default to the active profile.
            let value = match key.as_str() {
                // Top-level config
                "default_profile" => config.default_profile.as_ref().map(|s| s.as_str()),

                // Active profile fields (simple keys)
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

                _ => {
                    println!("Unknown key: '{}'", key);
                    println!(
                        "Valid keys: default_profile, workspace, user, repository, api_url, output_format"
                    );
                    return Ok(());
                }
            };

            match value {
                Some(v) => println!("{}", v),
                None => println!("Key not found or not set."),
            }
        }
    }
    Ok(())
}
