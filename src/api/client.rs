use anyhow::{Context, Result};
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;

/// Bitbucket API Client
///
/// Handles communication with the Bitbucket Cloud API v2.0.
/// Supports authentication via Basic Auth (App Password).
#[derive(Clone)]
pub struct BitbucketClient {
    client: Client,
    base_url: String,
    auth_header: Option<(String, String)>,
}

impl BitbucketClient {
    /// Create a new Bitbucket API client
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the Bitbucket API
    /// * `base_url` - The base URL for the Bitbucket API
    /// * `auth` - Optional tuple of (username, password/token) for Basic Auth
    pub fn new(base_url: String, auth: Option<(String, String)>) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            auth_header: auth,
        })
    }

    pub(crate) fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!(
                "{}/{}",
                self.base_url.trim_end_matches('/'),
                path.trim_start_matches('/')
            )
        };

        crate::utils::debug::log(&format!("Requesting: {} {}", method, url));

        let mut request = self.client.request(method, &url);

        if let Some((username, api_token)) = &self.auth_header {
            crate::utils::debug::log(&format!("Adding Basic Auth for user: {}", username));
            request = request.basic_auth(username, Some(api_token));
        } else {
            crate::utils::debug::log("No Auth header present for this request.");
        }

        request
    }

    /// Perform a GET request to the Bitbucket API
    ///
    /// # Arguments
    ///
    /// * `path` - The API path (relative to base URL) or full URL
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .build_request(Method::GET, path)
            .send()
            .await
            .context("Failed to send request")?;

        crate::utils::debug::log(&format!("Response status: {}", response.status()));

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(anyhow::anyhow!(
                "API request failed ({}) : {}",
                status,
                error_text
            ));
        }

        let data = response
            .json::<T>()
            .await
            .context("Failed to parse JSON response")?;
        Ok(data)
    }

    /// List pull requests for a repository
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `state` - Filter by PR state (e.g., "OPEN", "MERGED", "DECLINED")
    /// * `limit` - Optional maximum number of PRs to return
    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo: &str,
        state: &str,
        limit: Option<u32>,
    ) -> Result<Vec<crate::api::models::PullRequest>> {
        let mut all_prs = Vec::new();
        let mut path = format!(
            "/repositories/{}/{}/pullrequests?state={}",
            workspace, repo, state
        );

        loop {
            let response: crate::api::models::PaginatedResponse<crate::api::models::PullRequest> =
                self.get(&path).await?;

            all_prs.extend(response.values);

            // Check if we've reached the limit
            let limit_reached = limit.is_some_and(|max| all_prs.len() >= max as usize);

            if limit_reached {
                all_prs.truncate(limit.unwrap() as usize);
                break;
            }

            match response.next {
                Some(next_url) => path = next_url,
                None => break,
            }
        }

        Ok(all_prs)
    }

    /// List repositories in a workspace
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `limit` - Optional maximum number of repositories to return
    pub async fn list_repositories(
        &self,
        workspace: &str,
        limit: Option<u32>,
    ) -> Result<Vec<crate::api::models::Repository>> {
        let mut all_repos = Vec::new();
        let mut path = format!("/repositories/{}", workspace);

        loop {
            let response: crate::api::models::PaginatedResponse<crate::api::models::Repository> =
                self.get(&path).await?;

            all_repos.extend(response.values);

            // Check if we've reached the limit
            let limit_reached = limit.is_some_and(|max| all_repos.len() >= max as usize);

            if limit_reached {
                all_repos.truncate(limit.unwrap() as usize);
                break;
            }

            match response.next {
                Some(next_url) => path = next_url,
                None => break,
            }
        }

        Ok(all_repos)
    }

    /// Get a single pull request by ID
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `id` - The pull request ID
    pub async fn get_pull_request(
        &self,
        workspace: &str,
        repo: &str,
        id: u32,
    ) -> Result<crate::api::models::PullRequest> {
        let path = format!("/repositories/{}/{}/pullrequests/{}", workspace, repo, id);
        self.get(&path).await
    }

    /// Get the diff for a pull request
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `id` - The pull request ID
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
        let response = self
            .build_request(Method::GET, &path)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(anyhow::anyhow!(
                "API request failed ({}) : {}",
                status,
                error_text
            ));
        }

        let text = response.text().await.context("Failed to get diff text")?;
        Ok(text)
    }

    /// Get build/commit statuses for a commit
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `commit_hash` - The commit hash
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

    /// Get comments for a pull request
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `id` - The pull request ID
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

    /// Find a pull request by source branch name
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID or slug
    /// * `repo` - The repository slug
    /// * `branch_name` - The source branch name
    pub async fn find_pull_request_by_branch(
        &self,
        workspace: &str,
        repo: &str,
        branch_name: &str,
    ) -> Result<Option<crate::api::models::PullRequest>> {
        let path = format!("repositories/{}/{}/pullrequests", workspace, repo);

        // Ensure base URL ends with slash for join to work as expected (appending)
        // otherwise /2.0 gets replaced by /repositories
        let base = if self.base_url.ends_with('/') {
            self.base_url.clone()
        } else {
            format!("{}/", self.base_url)
        };

        // Construct URL safely using reqwest::Url to handle query encoding
        let mut url = reqwest::Url::parse(&base)
            .context("Invalid base URL")?
            .join(&path)
            .context("Failed to join path")?;

        let query = format!("source.branch.name=\"{}\"", branch_name);
        url.query_pairs_mut()
            .append_pair("q", &query)
            .append_pair("state", "OPEN");

        let response: crate::api::models::PaginatedResponse<crate::api::models::PullRequest> =
            self.get(url.as_str()).await?;

        Ok(response.values.into_iter().next())
    }

    /// Get the currently authenticated user
    pub async fn get_current_user(&self) -> Result<crate::api::models::User> {
        self.get("/user").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_presence() {
        let client = BitbucketClient::new(
            "https://api.bitbucket.org/2.0".to_string(),
            Some(("user".to_string(), "pass".to_string())),
        )
        .unwrap();

        let request = client.build_request(Method::GET, "/user").build().unwrap();

        let auth_header = request.headers().get(reqwest::header::AUTHORIZATION);
        assert!(
            auth_header.is_some(),
            "Authorization header should be present"
        );

        // Check that it's Basic auth
        let auth_str = auth_header.unwrap().to_str().unwrap();
        assert!(
            auth_str.starts_with("Basic "),
            "Authorization header should be Basic auth"
        );
    }

    #[test]
    fn test_no_auth_header() {
        let client =
            BitbucketClient::new("https://api.bitbucket.org/2.0".to_string(), None).unwrap();

        let request = client.build_request(Method::GET, "/user").build().unwrap();

        let auth_header = request.headers().get(reqwest::header::AUTHORIZATION);
        assert!(
            auth_header.is_none(),
            "Authorization header should NOT be present"
        );
    }
}
