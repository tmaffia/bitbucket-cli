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

    // Initialize AppContext
    let ctx = match context::AppContext::new(&cli) {
        Ok(c) => c,
        Err(e) => {
            display::ui::error(&format!("Error initializing context: {}", e));
            process::exit(1);
        }
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
