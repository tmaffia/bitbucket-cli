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

        /// Max number of PRs to fetch
        #[arg(long, default_value = "50")]
        limit: u32,
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
        /// Display only names of changed files
        #[arg(long)]
        name_only: bool,
        /// Open the pull request diff in the browser
        #[arg(long, short = 'w')]
        web: bool,
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
        PrCommands::List { state, limit } => {
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

            let prs = client
                .list_pull_requests(&workspace, &repo, &state, Some(limit))
                .await?;

            if prs.is_empty() {
                ui::info(&format!(
                    "No pull requests found in {}/{} with state {}",
                    workspace, repo, state
                ));
                return Ok(());
            }

            let table = pr_display::format_pr_list(&prs);
            if ui::should_use_pager() {
                ui::display_in_pager(&table)?;
            } else {
                println!("{}", table);
            }
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
        PrCommands::Diff { id, name_only, web } => {
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

            // Handle --web flag (open in browser)
            if web {
                let pr = client.get_pull_request(&workspace, &repo, pr_id).await?;
                let diff_url = format!("{}/diff", pr.links.html.href);
                open::that(diff_url)?;
                ui::success(&format!("Opened PR #{} diff in browser", pr_id));
                return Ok(());
            }

            let diff = client
                .get_pull_request_diff(&workspace, &repo, pr_id)
                .await?;

            // Handle --name-only flag
            if name_only {
                crate::display::diff::print_filenames_only(&diff);
            } else {
                // TODO: Add support for filtering (--exclude, --exclude-lockfiles, path patterns)
                // TODO: Add support for collapsing large diffs (--collapse-large)
                // TODO: Add --stat flag for git-style statistics
                crate::display::diff::print_diff(&diff)?;
            }
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

            if json {
                ui::print_json(&comments)?;
            } else {
                pr_display::print_comments(&comments);
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
