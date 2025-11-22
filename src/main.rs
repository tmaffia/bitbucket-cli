use clap::Parser;
use std::process;

mod api;
mod cli;
mod commands;
mod config;
mod constants;
mod context;
mod display;
mod git;
mod utils;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    utils::debug::set_enabled(cli.verbose);

    // Load configuration
    let config = match config::manager::ProfileConfig::load() {
        Ok(c) => c,
        Err(e) => {
            if !cli.quiet {
                display::ui::warning(&format!("Failed to load config: {}", e));
            }
            // Return empty config or default
            config::manager::ProfileConfig::default()
        }
    };

    // Initialize API client
    let client = match config.create_client(cli.profile.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            display::ui::error(&format!("Error initializing client: {}", e));
            process::exit(1);
        }
    };

    // Create AppContext
    let ctx = context::AppContext {
        config,
        client,
        repo_override: cli.repo,
        remote_override: cli.remote,
        json: cli.json,
    };

    let result = match cli.command {
        Commands::Pr(args) => commands::pr::handle(&ctx, args).await,
        Commands::Auth(args) => commands::auth::handle(&ctx, args).await,
        Commands::Config(args) => commands::config::handle(&ctx, args).await,
    };

    if let Err(e) = result {
        display::ui::error(&format!("{:#}", e));
        process::exit(1);
    }
}
