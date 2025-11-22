use anyhow::Result;
use clap::{Args, Subcommand};

use crate::display::{pr as pr_display, ui};

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
        /// Show comments
        #[arg(long)]
        comments: bool,
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
    json: bool,
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
                ui::info(&format!(
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
        PrCommands::View { id, web, comments } => {
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
                ui::success(&format!("Opened PR #{} in browser", pr.id));
                return Ok(());
            }

            let pr_comments = if comments || json {
                Some(
                    client
                        .get_pull_request_comments(&workspace, &repo, pr_id)
                        .await?,
                )
            } else {
                None
            };

            if json {
                #[derive(serde::Serialize)]
                struct JsonOutput {
                    pr: crate::api::models::PullRequest,
                    comments: Option<Vec<crate::api::models::Comment>>,
                }

                let output = JsonOutput {
                    pr,
                    comments: pr_comments,
                };

                ui::print_json(&output)?;
                return Ok(());
            }

            // Fetch build statuses
            let statuses = if let Some(commit) = &pr.source.commit {
                client
                    .get_commit_statuses(&workspace, &repo, &commit.hash)
                    .await?
            } else {
                Vec::new()
            };

            pr_display::print_pr_details(&pr, &statuses);

            // Display Comments
            if let Some(comments_list) = pr_comments {
                pr_display::print_comments(&comments_list);
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
                ui::info(&format!("No comments found for PR #{}", pr_id));
                return Ok(());
            }

            pr_display::print_comments(&comments);
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
