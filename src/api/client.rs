use crate::config::manager::ProfileConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct BitbucketClient {
    client: Client,
    base_url: String,
    auth_header: Option<(String, String)>,
}

impl BitbucketClient {
    pub fn new(
        profile: Option<&ProfileConfig>,
        auth_override: Option<(String, String)>,
    ) -> Result<Self> {
        let base_url = profile
            .and_then(|p| p.api_url.clone())
            .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

        let mut auth_header = auth_override;

        if auth_header.is_none() {
            if let Some(username) = profile.and_then(|p| p.user.as_ref()) {
                if let Ok(entry) =
                    keyring::Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)
                {
                    if let Ok(password) = entry.get_password() {
                        auth_header = Some((username.clone(), password));
                    }
                }
            }
        }

        let client_builder = Client::builder();

        // If we have auth header, add it as default basic auth
        // Note: reqwest::ClientBuilder has .default_headers but for basic auth it's per request usually,
        // or we can use .default_basic_auth() if we want it for all requests.
        // But BitbucketClient wraps Client.
        // Let's store it in the struct and apply it in `get`.
        // Actually, better to use a middleware or just apply it manually.
        // Storing in `auth_header` field is fine.

        let client = client_builder
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        if let Some((username, password)) = &self.auth_header {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await.context("Failed to send request")?;

        if !response.status().is_success() {
            // TODO: Better error handling
            return Err(anyhow::anyhow!("API request failed: {}", response.status()));
        }

        let data = response
            .json::<T>()
            .await
            .context("Failed to parse JSON response")?;
        Ok(data)
    }

    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo: &str,
        state: &str,
    ) -> Result<Vec<crate::api::models::PullRequest>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests?state={}",
            workspace, repo, state
        );
        let response: crate::api::models::PaginatedResponse<crate::api::models::PullRequest> =
            self.get(&path).await?;
        Ok(response.values)
    }

    pub async fn get_pull_request(
        &self,
        workspace: &str,
        repo: &str,
        id: u32,
    ) -> Result<crate::api::models::PullRequest> {
        let path = format!("/repositories/{}/{}/pullrequests/{}", workspace, repo, id);
        self.get(&path).await
    }

    pub async fn get_pull_request_diff(
        &self,
        workspace: &str,
        repo: &str,
        id: u32,
    ) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/diff",
            workspace, repo, id
        );
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        if let Some((username, password)) = &self.auth_header {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await.context("Failed to send request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API request failed: {}", response.status()));
        }

        let text = response.text().await.context("Failed to get diff text")?;
        Ok(text)
    }

    pub async fn get_commit_statuses(
        &self,
        workspace: &str,
        repo: &str,
        commit_hash: &str,
    ) -> Result<Vec<crate::api::models::CommitStatus>> {
        let path = format!(
            "/repositories/{}/{}/commit/{}/statuses",
            workspace, repo, commit_hash
        );
        let response: crate::api::models::PaginatedResponse<crate::api::models::CommitStatus> =
            self.get(&path).await?;
        Ok(response.values)
    }

    pub async fn get_pull_request_comments(
        &self,
        workspace: &str,
        repo: &str,
        id: u32,
    ) -> Result<Vec<crate::api::models::Comment>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments",
            workspace, repo, id
        );
        let response: crate::api::models::PaginatedResponse<crate::api::models::Comment> =
            self.get(&path).await?;
        Ok(response.values)
    }

    pub async fn find_pull_request_by_branch(
        &self,
        workspace: &str,
        repo: &str,
        branch_name: &str,
    ) -> Result<Option<crate::api::models::PullRequest>> {
        // Fetch open PRs
        // TODO: Use server-side filtering with `q` parameter for better performance
        let prs = self.list_pull_requests(workspace, repo, "OPEN").await?;

        // Find matching branch
        let pr = prs
            .into_iter()
            .find(|pr| pr.source.branch.name == branch_name);

        Ok(pr)
    }

    pub async fn get_current_user(&self) -> Result<crate::api::models::User> {
        self.get("/user").await
    }
}
