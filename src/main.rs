use clap::{Parser, Subcommand};
use std::process;

mod api;
mod commands;
mod config;
mod constants;
mod display;
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
    
        /// Override remote name
        #[arg(long, global = true)]
        remote: Option<String>,
    
        /// Output as JSON
        #[arg(long, global = true)]
        json: bool,
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
        utils::debug::set_enabled(cli.verbose);
    
        // Load configuration
        let config = match config::manager::AppConfig::load() {
            Ok(c) => c,
            Err(e) => {
                if !cli.quiet {
                    display::ui::warning(&format!("Failed to load config: {}", e));
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
    
        utils::debug::log(&format!("Active profile: {}", profile_name));
    
        let profile = config.profiles.as_ref().and_then(|p| p.get(&profile_name));
    
        if let Some(p) = profile {
            utils::debug::log(&format!("Profile loaded. User: {:?}", p.user));
        } else {
            utils::debug::log(&format!("Profile '{}' NOT found in config.", profile_name));
        }
    
        // Resolve Remote
        let configured_remote = profile.and_then(|p| p.remote.clone());
        let remote = cli.remote.or(configured_remote);
    
        // Resolve Base URL
        let base_url = profile
            .and_then(|p| p.api_url.clone())
            .unwrap_or_else(|| constants::DEFAULT_API_URL.to_string());
    
        // Resolve Auth
        let mut auth = None;
        if let Some(username) = profile.and_then(|p| p.user.as_ref()) {
            match utils::auth::get_credentials(username) {
                Ok(password) => {
                    utils::debug::log(&format!("Credentials found for user '{}'", username));
                    auth = Some((username.clone(), password));
                }
                Err(e) => {
                    utils::debug::log(&format!("Failed to load credentials for user '{}': {}", username, e));
                }
            }
        } else {
            utils::debug::log("No user configured in profile. Running unauthenticated.");
        }
    
        // Initialize API client
        let client = match api::client::BitbucketClient::new(base_url, auth) {
            Ok(c) => c,
            Err(e) => {
                display::ui::error(&format!("Error initializing client: {}", e));
                process::exit(1);
            }
        };
    
        let result = match cli.command {
            Commands::Pr(args) => commands::pr::handle(args, cli.repo, remote, cli.json, &client).await,
            Commands::Auth(args) => commands::auth::handle(args).await,
            Commands::Config(args) => commands::config::handle(args).await,
        };
    
        if let Err(e) = result {
            display::ui::error(&format!("{:#}", e));
            process::exit(1);
        }
    }
