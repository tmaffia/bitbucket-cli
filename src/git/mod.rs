use anyhow::{Context, Result};
use std::process::Command;

pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Not a git repository"));
    }

    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in branch name")?
        .trim()
        .to_string();

    Ok(branch)
}

pub fn get_repo_info() -> Result<(String, String)> {
    // Get remote URL (assume 'origin' for now)
    let output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("No remote 'origin' found"));
    }

    let url_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in remote URL")?
        .trim()
        .to_string();

    parse_git_url(&url_str)
}

fn parse_git_url(url: &str) -> Result<(String, String)> {
    // 1. Normalize: Try to strip prefixes to get just "workspace/repo.git"
    let path = url
        .strip_prefix("git@bitbucket.org:")
        .or_else(|| url.strip_prefix("https://bitbucket.org/"))
        .ok_or_else(|| anyhow::anyhow!("Could not parse Bitbucket URL: {}", url))?;

    // 2. Parse: Split into components efficiently
    let (workspace, repo_with_ext) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid repository format"))?;

    let repo = repo_with_ext.trim_end_matches(".git");

    Ok((workspace.to_string(), repo.to_string()))
}
