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

pub fn get_repo_info(remote_name: Option<&str>) -> Result<(String, String)> {
    let remote = remote_name.unwrap_or("origin");
    // Get remote URL
    let output = Command::new("git")
        .args(&["remote", "get-url", remote])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("No remote '{}' found", remote));
    }

    let url_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in remote URL")?
        .trim()
        .to_string();

    parse_git_url(&url_str)
}

fn parse_git_url(url: &str) -> Result<(String, String)> {
    // Basic support for ssh:// and user@ formats
    // This handles:
    // - git@bitbucket.org:workspace/repo.git
    // - https://bitbucket.org/workspace/repo.git
    // - https://username@bitbucket.org/workspace/repo.git
    // - ssh://git@bitbucket.org/workspace/repo.git
    
    let cleaned = url
        .trim_start_matches("ssh://")
        .trim_start_matches("git@")
        .trim_start_matches("https://")
        .trim_start_matches("http://");
        
    // If there is an '@' now, it's likely "username@host", so take everything after the last '@'
    let cleaned = cleaned.split('@').last().unwrap_or(cleaned);

    // Handle bitbucket.org prefix
    let path = cleaned
        .strip_prefix("bitbucket.org/")
        .or_else(|| cleaned.strip_prefix("bitbucket.org:")) // Handle scp-like syntax
        .ok_or_else(|| anyhow::anyhow!("Could not parse Bitbucket URL: {}", url))?;

    // 2. Parse: Split into components efficiently
    let (workspace, repo_with_ext) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid repository format"))?;

    let repo = repo_with_ext.trim_end_matches(".git");

    Ok((workspace.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_url() {
        let cases = vec![
            (
                "https://bitbucket.org/workspace/repo.git",
                ("workspace", "repo"),
            ),
            (
                "git@bitbucket.org:workspace/repo.git",
                ("workspace", "repo"),
            ),
            (
                "https://username@bitbucket.org/workspace/repo.git",
                ("workspace", "repo"),
            ),
            (
                "ssh://git@bitbucket.org/workspace/repo.git",
                ("workspace", "repo"),
            ),
            (
                "git@bitbucket.org:workspace/repo",
                ("workspace", "repo"),
            ),
            (
                "https://bitbucket.org/workspace/repo",
                ("workspace", "repo"),
            ),
        ];

        for (url, (expected_workspace, expected_repo)) in cases {
            let (workspace, repo) = parse_git_url(url).expect(&format!("Failed to parse {}", url));
            assert_eq!(workspace, expected_workspace, "Workspace mismatch for {}", url);
            assert_eq!(repo, expected_repo, "Repo mismatch for {}", url);
        }
    }

    #[test]
    fn test_parse_git_url_errors() {
        let invalid_urls = vec![
            "https://github.com/workspace/repo.git",
            "git@github.com:workspace/repo.git",
            "invalid_url",
        ];

        for url in invalid_urls {
            assert!(parse_git_url(url).is_err(), "Should fail for {}", url);
        }
    }
}
