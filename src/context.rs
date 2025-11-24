use crate::api::client::BitbucketClient;
use crate::cli::Cli;
use crate::config::manager::ProfileConfig;
use crate::{display, git, utils};
use anyhow::{Context, Result};

pub struct AppContext {
    pub client: BitbucketClient,
    pub json: bool,
    pub workspace: Option<String>,
    pub repo: Option<String>,
}

impl AppContext {
    pub fn new(cli: &Cli) -> Result<Self> {
        // 1. Load Global Config (Preferences & Auth)
        let global_config = match ProfileConfig::load_global() {
            Ok(c) => c,
            Err(e) => {
                if !cli.quiet {
                    display::ui::warning(&format!("Failed to load global config: {}", e));
                }
                ProfileConfig::default()
            }
        };

        // 2. Load Local Config (Project overrides)
        let local_config = match ProfileConfig::load_local() {
            Ok(c) => c,
            Err(e) => {
                if !cli.quiet {
                    display::ui::warning(&format!("Failed to load local config: {}", e));
                }
                None
            }
        };

        // 3. Get Git Context
        let git_info = if let Ok(branch) = git::get_current_branch() {
            // We are in a git repo
            let remote_name = cli.remote.as_deref().or(local_config
                .as_ref()
                .and_then(|c| c.get_active_profile())
                .and_then(|p| p.remote.as_deref()));

            match git::get_repo_info(remote_name) {
                Ok((ws, repo)) => Some((ws, repo, branch)),
                Err(e) => {
                    utils::debug::log(&format!("Failed to get git repo info: {}", e));
                    None
                }
            }
        } else {
            None
        };

        // 4. Resolve Workspace
        // Priority: CLI > Local Config > Git Remote > Global Config
        let workspace = cli
            .repo
            .as_ref()
            .and_then(|r| r.split_once('/').map(|(ws, _)| ws.to_string()))
            .or_else(|| {
                local_config
                    .as_ref()
                    .and_then(|c| c.get_active_profile())
                    .and_then(|p| p.workspace.clone())
            })
            .or_else(|| git_info.as_ref().map(|(ws, _, _)| ws.clone()))
            .or_else(|| {
                global_config
                    .get_active_profile()
                    .and_then(|p| p.workspace.clone())
            });

        // 5. Resolve Repository
        // Priority: CLI > Local Config > Git Remote
        let repo = cli
            .repo
            .as_ref()
            .and_then(|r| r.split_once('/').map(|(_, r)| r.to_string()))
            .or_else(|| {
                local_config
                    .as_ref()
                    .and_then(|c| c.get_active_profile())
                    .and_then(|p| p.repository.clone())
            })
            .or_else(|| git_info.as_ref().map(|(_, r, _)| r.clone()));

        // Initialize API client
        let client = global_config
            .create_client(cli.profile.as_deref())
            .context("Error initializing client")?;

        Ok(Self {
            client,
            json: cli.json,
            workspace,
            repo,
        })
    }
}
