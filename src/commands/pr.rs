use anyhow::Result;
use clap::{Args, Subcommand};

use crate::utils::display;

#[derive(Args)]
pub struct PrArgs {
    #[command(subcommand)]
    pub command: PrCommands,
}

#[derive(Subcommand)]
pub enum PrCommands {
    /// List pull requests
    List {
        /// Filter by state
        #[arg(long, default_value = "OPEN")]
        state: String,
    },
    /// View a pull request
    View {
        /// PR ID (optional, infers from branch if missing)
        id: Option<u32>,
        /// Open in browser
        #[arg(long)]
        web: bool,
    },
    /// Show diff
    Diff {
        /// PR ID (optional, infers from branch if missing)
        id: Option<u32>,
    },
    /// Show comments
    Comments {
        /// PR ID (optional, infers from branch if missing)
        id: Option<u32>,
    },
}

use crate::api::client::BitbucketClient;

pub async fn handle(
    args: PrArgs,
    repo_override: Option<String>,
    client: &BitbucketClient,
) -> Result<()> {
    match args.command {
        PrCommands::List { state } => {
            let (workspace, repo) = if let Some(r) = repo_override {
                let parts: Vec<&str> = r.split('/').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Invalid repo format. Expected workspace/repo"
                    ));
                }
                (parts[0].to_string(), parts[1].to_string())
            } else {
                crate::git::get_repo_info()?
            };

            let prs = client.list_pull_requests(&workspace, &repo, &state).await?;

            if prs.is_empty() {
                display::info(&format!(
                    "No pull requests found in {}/{} with state {}",
                    workspace, repo, state
                ));
                return Ok(());
            }

            let mut table = comfy_table::Table::new();
            table.set_header(vec!["ID", "Title", "Author", "Source", "State", "Updated"]);

            for pr in prs {
                table.add_row(vec![
                    pr.id.to_string(),
                    pr.title,
                    pr.author.display_name,
                    pr.source.branch.name,
                    pr.state,
                    pr.updated_on,
                ]);
            }

            println!("{}", table);
        }
        PrCommands::View { id, web } => {
            let (workspace, repo) = if let Some(r) = repo_override {
                let parts: Vec<&str> = r.split('/').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Invalid repo format. Expected workspace/repo"
                    ));
                }
                (parts[0].to_string(), parts[1].to_string())
            } else {
                crate::git::get_repo_info()?
            };

            let pr_id = resolve_pr_id(id, client, &workspace, &repo).await?;

            let pr = client.get_pull_request(&workspace, &repo, pr_id).await?;

            if web {
                open::that(pr.links.html.href)?;
                display::success(&format!("Opened PR #{} in browser", pr.id));
            } else {
                println!("PR #{} - {}", pr.id, pr.title);
                println!("State: {}", pr.state);
                println!("Author: {}", pr.author.display_name);
                println!("Source: {}", pr.source.branch.name);
                println!("Link: {}", pr.links.html.href);
                if let Some(desc) = pr.description {
                    println!("\n{}", desc);
                }
            }
        }
        PrCommands::Diff { id } => {
            let (workspace, repo) = if let Some(r) = repo_override {
                let parts: Vec<&str> = r.split('/').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Invalid repo format. Expected workspace/repo"
                    ));
                }
                (parts[0].to_string(), parts[1].to_string())
            } else {
                crate::git::get_repo_info()?
            };

            let pr_id = resolve_pr_id(id, client, &workspace, &repo).await?;

            let diff = client
                .get_pull_request_diff(&workspace, &repo, pr_id)
                .await?;
            println!("{}", diff);
        }
        PrCommands::Comments { id } => {
            let (workspace, repo) = if let Some(r) = repo_override {
                let parts: Vec<&str> = r.split('/').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Invalid repo format. Expected workspace/repo"
                    ));
                }
                (parts[0].to_string(), parts[1].to_string())
            } else {
                crate::git::get_repo_info()?
            };

            let pr_id = resolve_pr_id(id, client, &workspace, &repo).await?;

            let comments = client
                .get_pull_request_comments(&workspace, &repo, pr_id)
                .await?;

            if comments.is_empty() {
                display::info(&format!("No comments found for PR #{}", pr_id));
                return Ok(());
            }

            for comment in comments {
                println!("--------------------------------------------------");
                println!(
                    "Author: {} ({})",
                    comment.user.display_name, comment.created_on
                );
                if let Some(inline) = comment.inline {
                    println!("File: {}", inline.path);
                    if let Some(line) = inline.to.or(inline.from) {
                        println!("Line: {}", line);
                    }
                }
                println!("\n{}", comment.content.raw);
            }
        }
    }
    Ok(())
}

async fn resolve_pr_id(
    id: Option<u32>,
    client: &BitbucketClient,
    workspace: &str,
    repo: &str,
) -> Result<u32> {
    if let Some(i) = id {
        return Ok(i);
    }
    let branch = crate::git::get_current_branch()?;
    let pr = client
        .find_pull_request_by_branch(workspace, repo, &branch)
        .await?;
    match pr {
        Some(p) => Ok(p.id),
        None => Err(anyhow::anyhow!("No open PR found for branch '{}'", branch)),
    }
}
