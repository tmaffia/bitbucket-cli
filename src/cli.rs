use crate::commands;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bb", about = "Bitbucket CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose mode
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Override profile
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Override repository (format: workspace/repo)
    #[arg(short = 'R', long, global = true)]
    pub repo: Option<String>,

    /// Override remote name
    #[arg(long, global = true)]
    pub remote: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pull request operations
    Pr(commands::pr::PrArgs),
    /// Authentication
    Auth(commands::auth::AuthArgs),
    /// Configuration
    Config(commands::config::ConfigArgs),
}
