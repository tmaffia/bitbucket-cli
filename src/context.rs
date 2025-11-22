use crate::api::client::BitbucketClient;
use crate::config::manager::ProfileConfig;

pub struct AppContext {
    #[allow(dead_code)]
    pub config: ProfileConfig,
    pub client: BitbucketClient,
    pub repo_override: Option<String>,
    pub remote_override: Option<String>,
    pub json: bool,
}
