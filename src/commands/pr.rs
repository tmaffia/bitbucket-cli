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

use crate::context::AppContext;

pub async fn handle(ctx: &AppContext, args: PrArgs) -> Result<()> {
    match args.command {
        PrCommands::List { state, limit } => {
            let (workspace, repo) = resolve_repo_info(ctx)?;

            let prs = ctx
                .client
                .list_pull_requests(&workspace, &repo, &state, Some(limit))
                .await?;

            if ctx.json {
                ui::print_json(&prs)?;
                return Ok(());
            }

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
            let (workspace, repo) = resolve_repo_info(ctx)?;

            let pr_id = resolve_pr_id(id, &ctx.client, &workspace, &repo).await?;
            let pr = ctx
                .client
                .get_pull_request(&workspace, &repo, pr_id)
                .await?;

            if web {
                open::that(pr.links.html.href)?;
                ui::success(&format!("Opened PR #{} in browser", pr.id));
                return Ok(());
            }

            let pr_comments = if comments || ctx.json {
                Some(
                    ctx.client
                        .get_pull_request_comments(&workspace, &repo, pr_id)
                        .await?,
                )
            } else {
                None
            };

            if ctx.json {
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
                ctx.client
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
            let (workspace, repo) = resolve_repo_info(ctx)?;

            let pr_id = resolve_pr_id(id, &ctx.client, &workspace, &repo).await?;

            // Handle --web flag (open in browser)
            if web {
                let pr = ctx
                    .client
                    .get_pull_request(&workspace, &repo, pr_id)
                    .await?;
                let diff_url = format!("{}/diff", pr.links.html.href);
                open::that(diff_url)?;
                ui::success(&format!("Opened PR #{} diff in browser", pr_id));
                return Ok(());
            }

            let diff = ctx
                .client
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
            let (workspace, repo) = resolve_repo_info(ctx)?;

            let pr_id = resolve_pr_id(id, &ctx.client, &workspace, &repo).await?;

            let comments = ctx
                .client
                .get_pull_request_comments(&workspace, &repo, pr_id)
                .await?;

            if comments.is_empty() {
                ui::info(&format!("No comments found for PR #{}", pr_id));
                return Ok(());
            }

            if ctx.json {
                ui::print_json(&comments)?;
            } else {
                pr_display::print_comments(&comments);
            }
        }
    }
    Ok(())
}

/// Resolve repository information from overrides, git configuration, or config file
///
/// Tries in order:
/// 1. Explicit CLI override (`--repo workspace/repo`)
/// 2. Git remote detection
/// 3. Config file default repository
fn resolve_repo_info(ctx: &AppContext) -> Result<(String, String)> {
    resolve_from_override(ctx)
        .or_else(|| resolve_from_git(ctx))
        .or_else(|| resolve_from_config(ctx))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not determine repository. Use --repo or configure a default repository."
            )
        })
}

/// Try to resolve repository info from CLI override
fn resolve_from_override(ctx: &AppContext) -> Option<(String, String)> {
    ctx.repo_override.as_ref().and_then(|r| {
        let parts: Vec<_> = r.split('/').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            // Invalid format - should we warn here?
            // For now, return None to allow fallback to other methods
            None
        }
    })
}

/// Try to resolve repository info from git remote
fn resolve_from_git(ctx: &AppContext) -> Option<(String, String)> {
    crate::git::get_repo_info(ctx.remote_override.as_deref()).ok()
}

/// Try to resolve repository info from config file
fn resolve_from_config(ctx: &AppContext) -> Option<(String, String)> {
    ctx.config.get_active_profile().and_then(|profile| {
        profile
            .repository
            .as_ref()
            .map(|r| (profile.workspace.clone(), r.clone()))
    })
}

/// Resolve Pull Request ID from argument or current branch
///
/// # Arguments
///
/// * `id` - Optional explicit PR ID
/// * `client` - Bitbucket API client
/// * `workspace` - Workspace ID/slug
/// * `repo` - Repository slug
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::manager::{Profile, ProfileConfig};
    use std::collections::HashMap;

    fn create_test_context(
        repo_override: Option<String>,
        remote_override: Option<String>,
        config_workspace: Option<String>,
        config_repo: Option<String>,
    ) -> AppContext {
        let mut profiles = HashMap::new();

        if let (Some(ws), Some(repo)) = (config_workspace, config_repo) {
            profiles.insert(
                "default".to_string(),
                Profile {
                    workspace: ws,
                    user: None,
                    repository: Some(repo),
                    api_url: None,
                    output_format: None,
                    remote: None,
                },
            );
        }

        let config = ProfileConfig {
            default_profile: None,
            profiles: if profiles.is_empty() {
                None
            } else {
                Some(profiles)
            },
        };

        // Create a dummy client - we won't use it in these tests
        let client = crate::api::client::BitbucketClient::new(
            "https://api.bitbucket.org/2.0".to_string(),
            None,
        )
        .unwrap();

        AppContext {
            config,
            client,
            repo_override,
            remote_override,
            json: false,
        }
    }

    #[test]
    fn test_resolve_from_override_valid() {
        let ctx = create_test_context(Some("myworkspace/myrepo".to_string()), None, None, None);

        let result = resolve_from_override(&ctx);
        assert_eq!(
            result,
            Some(("myworkspace".to_string(), "myrepo".to_string()))
        );
    }

    #[test]
    fn test_resolve_from_override_invalid_format() {
        let ctx = create_test_context(Some("invalid-format".to_string()), None, None, None);

        let result = resolve_from_override(&ctx);
        // Should return None to allow fallback, not error
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_from_override_too_many_slashes() {
        let ctx = create_test_context(Some("workspace/repo/extra".to_string()), None, None, None);

        let result = resolve_from_override(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_from_override_none() {
        let ctx = create_test_context(None, None, None, None);

        let result = resolve_from_override(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_from_config_valid() {
        let ctx = create_test_context(
            None,
            None,
            Some("config-workspace".to_string()),
            Some("config-repo".to_string()),
        );

        let result = resolve_from_config(&ctx);
        assert_eq!(
            result,
            Some(("config-workspace".to_string(), "config-repo".to_string()))
        );
    }

    #[test]
    fn test_resolve_from_config_no_repository() {
        let ctx = create_test_context(None, None, Some("workspace".to_string()), None);

        let result = resolve_from_config(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_from_config_no_profile() {
        let ctx = create_test_context(None, None, None, None);

        let result = resolve_from_config(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_repo_info_from_override() {
        let ctx = create_test_context(
            Some("override-ws/override-repo".to_string()),
            None,
            Some("config-ws".to_string()),
            Some("config-repo".to_string()),
        );

        // Override should take precedence
        let result = resolve_repo_info(&ctx);
        assert!(result.is_ok());
        let (ws, repo) = result.unwrap();
        assert_eq!(ws, "override-ws");
        assert_eq!(repo, "override-repo");
    }

    #[test]
    fn test_resolve_repo_info_from_config() {
        let ctx = create_test_context(
            None,
            None,
            Some("config-ws".to_string()),
            Some("config-repo".to_string()),
        );

        // Should fall through to config since no override or git
        let result = resolve_repo_info(&ctx);
        assert!(result.is_ok());
        let (ws, repo) = result.unwrap();
        assert_eq!(ws, "config-ws");
        assert_eq!(repo, "config-repo");
    }

    #[test]
    fn test_resolve_repo_info_invalid_override_fallback_to_config() {
        let ctx = create_test_context(
            Some("invalid-format".to_string()),
            None,
            Some("config-ws".to_string()),
            Some("config-repo".to_string()),
        );

        // Invalid override should fall back to config
        let result = resolve_repo_info(&ctx);
        assert!(result.is_ok());
        let (ws, repo) = result.unwrap();
        assert_eq!(ws, "config-ws");
        assert_eq!(repo, "config-repo");
    }

    #[test]
    fn test_resolve_repo_info_no_sources_fails() {
        let ctx = create_test_context(None, None, None, None);

        // No sources available, should error
        let result = resolve_repo_info(&ctx);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Could not determine repository")
        );
    }
}
