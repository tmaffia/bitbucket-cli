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
                // If it's a parse error or IO error other than NotFound, we should probably fail?
                // For now, keeping warning behavior but making it more visible if needed.
                // But plan said "Improve error visibility".
                // If the file exists but is invalid, we should error.
                // load_global uses build_global_config which uses config crate.
                // We can't easily distinguish "not found" from "parse error" without inspecting error.
                // But usually config crate handles "not found" by just returning default if we set it up that way,
                // but here we are adding source file.
                // Let's just warn for now as per existing behavior but maybe upgrade to error if it's critical?
                // The user review didn't explicitly demand erroring out, just "Improve error visibility".
                if !cli.quiet {
                    display::ui::warning(&format!("Failed to load global config: {}", e));
                }
                ProfileConfig::default()
            }
        };

        // 2. Get Git Context (Repo Root) - ONCE
        let repo_root = git::get_repo_root().ok();

        // 3. Load Local Config (Project overrides)
        // Pass the already resolved repo_root
        let local_config = match ProfileConfig::load_local(repo_root.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                if !cli.quiet {
                    display::ui::warning(&format!("Failed to load local config: {}", e));
                }
                None
            }
        };

        // 4. Resolve Git Remote Info
        // We need to know which remote to check.
        let remote_name = cli.remote.as_deref().or(local_config
            .as_ref()
            .and_then(|c| c.project.as_ref())
            .and_then(|p| p.remote.as_deref()));

        let git_info = if repo_root.is_some() {
            match git::get_repo_info(remote_name) {
                Ok((ws, repo)) => Some((ws, repo)),
                Err(e) => {
                    utils::debug::log(&format!("Failed to get git repo info: {}", e));
                    None
                }
            }
        } else {
            None
        };

        let cli_coords = if let Some(r) = &cli.repo {
            if let Some((w, r)) = r.split_once('/') {
                Some((Some(w.to_string()), Some(r.to_string())))
            } else {
                // If no slash, treat as just repo name, workspace remains None (to be resolved later)
                Some((None, Some(r.to_string())))
            }
        } else {
            None
        };

        // 5. Resolve Workspace
        // Priority: CLI > Local Config > Git Remote > Global Config
        let workspace = cli_coords
            .as_ref()
            .and_then(|(w, _)| w.clone())
            .or_else(|| {
                local_config
                    .as_ref()
                    .and_then(|c| c.project.as_ref())
                    .and_then(|p| p.workspace.clone())
            })
            .or_else(|| git_info.as_ref().map(|(ws, _)| ws.clone()))
            .or_else(|| {
                global_config
                    .get_active_profile()
                    .and_then(|p| p.workspace.clone())
            });

        // 6. Resolve Repository
        // Priority: CLI > Local Config > Git Remote
        let repo = cli_coords
            .as_ref()
            .and_then(|(_, r)| r.clone())
            .or_else(|| {
                local_config
                    .as_ref()
                    .and_then(|c| c.project.as_ref())
                    .and_then(|p| p.repository.clone())
            })
            .or_else(|| git_info.as_ref().map(|(_, r)| r.clone()));

        // Initialize API client
        let client = global_config
            .create_client(cli.profile.as_deref())
            .context("Error initializing client")?;

        utils::debug::log(&format!(
            "Context resolved - Workspace: {:?}, Repo: {:?}",
            workspace, repo
        ));

        Ok(Self {
            client,
            json: cli.json,
            workspace,
            repo,
        })
    }
}
