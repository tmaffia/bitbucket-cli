use clap::{Parser, Subcommand};
use std::process;

mod api;
mod commands;
mod config;
mod constants;
mod git;
mod utils;

#[derive(Parser)]
#[command(name = "bb", about = "Bitbucket CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose mode
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Override profile
    #[arg(long, global = true)]
    profile: Option<String>,

    /// Override repository (format: workspace/repo)
    #[arg(short = 'R', long, global = true)]
    repo: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull request operations
    Pr(commands::pr::PrArgs),
    /// Authentication
    Auth(commands::auth::AuthArgs),
    /// Configuration
    Config(commands::config::ConfigArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Load configuration
    let config = match config::manager::AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            if !cli.quiet {
                utils::display::warning(&format!("Failed to load config: {}", e));
            }
            // Return empty config or default
            config::manager::AppConfig {
                default_profile: None,
                profiles: None,
            }
        }
    };

    // Determine profile
    let profile_name = cli
        .profile
        .or(config.default_profile.clone())
        .unwrap_or_else(|| "default".to_string());
    let profile = config.profiles.as_ref().and_then(|p| p.get(&profile_name));

    // Initialize API client
    let client = match api::client::BitbucketClient::new(profile, None) {
        Ok(c) => c,
        Err(e) => {
            utils::display::error(&format!("Error initializing client: {}", e));
            process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Pr(args) => commands::pr::handle(args, cli.repo, &client).await,
        Commands::Auth(args) => commands::auth::handle(args).await,
        Commands::Config(args) => commands::config::handle(args).await,
    };

    if let Err(e) = result {
        utils::display::error(&format!("{:#}", e));
        process::exit(1);
    }
}
