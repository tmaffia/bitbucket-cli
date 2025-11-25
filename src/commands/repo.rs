use crate::context::AppContext;
use crate::display::ui;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommands,
}

#[derive(Subcommand)]
pub enum RepoCommands {
    /// List repositories in the workspace
    List {
        /// Workspace to list repositories from (defaults to configured workspace)
        #[arg(long, short)]
        workspace: Option<String>,

        /// Limit the number of repositories to return (default: 100)
        #[arg(long, default_value = "100")]
        limit: u32,
    },
}

pub async fn handle(ctx: &AppContext, args: RepoArgs) -> Result<()> {
    match args.command {
        RepoCommands::List { workspace, limit } => {
            let ws = workspace
                .or_else(|| ctx.workspace.clone())
                .context("No workspace configured. Please set a default workspace with 'bb config set workspace <NAME>' or provide --workspace")?;

            let client = ctx.client.clone(); // Use client from context which is already initialized with auth

            ui::info(&format!("Fetching repositories for workspace '{}'...", ws));

            let repos = client.list_repositories(&ws, Some(limit)).await?;

            if ctx.json {
                ui::print_json(&repos)?;
            } else {
                crate::display::repo::print_repo_list(&repos);
            }
        }
    }
    Ok(())
}
