use clap::{Args, Subcommand};
use anyhow::Result;

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
}

pub async fn handle(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Init => {
            println!("Initializing config...");
        }
        ConfigCommands::List => {
            println!("Listing config...");
        }
    }
    Ok(())
}
