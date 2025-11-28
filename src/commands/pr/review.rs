use crate::context::AppContext;
use anyhow::{Context, Result};
use clap::Args;
use dialoguer::{Input, Select};

#[derive(Args, Debug)]
pub struct ReviewArgs {
    /// The ID of the pull request to review
    pub id: Option<u32>,

    /// Approve the pull request
    #[arg(short, long)]
    pub approve: bool,

    /// Request changes on the pull request
    #[arg(short, long)]
    pub request_changes: bool,

    /// Comment on the pull request
    #[arg(short, long)]
    pub comment: bool,

    /// The body of the review or comment
    #[arg(short, long)]
    pub body: Option<String>,
}

pub async fn pr_review(ctx: &AppContext, args: &ReviewArgs) -> Result<()> {
    let workspace = ctx
        .workspace
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No workspace found"))?;
    let repo = ctx
        .repo
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No repository found"))?;

    // Determine PR ID
    let pr_id = match args.id {
        Some(id) => id,
        None => {
            // Try to deduce from current branch
            let branch = crate::git::get_current_branch()?;
            let pr = ctx
                .client
                .find_pull_request_by_branch(workspace, repo, &branch)
                .await?
                .context("No open pull request found for current branch")?;
            pr.id
        }
    };

    // Check if flags are provided
    if args.approve || args.request_changes || args.comment {
        if args.approve {
            ctx.client.approve_pr(workspace, repo, pr_id).await?;
            println!("Approved pull request #{}", pr_id);
        }

        if args.request_changes {
            ctx.client.request_changes(workspace, repo, pr_id).await?;
            println!("Requested changes on pull request #{}", pr_id);
        }

        if args.comment {
            let body = args
                .body
                .clone()
                .context("Comment body is required when using --comment")?;
            ctx.client
                .post_pr_comment(workspace, repo, pr_id, &body)
                .await?;
            println!("Commented on pull request #{}", pr_id);
        }
    } else {
        // Interactive mode
        let selections = &["Approve", "Request Changes", "Comment"];
        let selection = Select::new()
            .with_prompt("Select review action")
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selection {
            0 => {
                // Approve
                ctx.client.approve_pr(workspace, repo, pr_id).await?;
                println!("Approved pull request #{}", pr_id);
            }
            1 => {
                // Request Changes
                ctx.client.request_changes(workspace, repo, pr_id).await?;
                println!("Requested changes on pull request #{}", pr_id);
            }
            2 => {
                // Comment
                let body: String = Input::new().with_prompt("Comment body").interact_text()?;
                ctx.client
                    .post_pr_comment(workspace, repo, pr_id, &body)
                    .await?;
                println!("Commented on pull request #{}", pr_id);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
