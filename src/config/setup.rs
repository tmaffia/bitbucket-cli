use anyhow::Result;
use std::io::{self, Write};

use crate::display::ui;

pub fn interactive_init() -> Result<()> {
    ui::info("Initializing config...");
    // Interactive setup
    let mut input = String::new();

    print!("Initialize configuration in current directory? (y/n) [n]: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let local_init = input.trim().to_lowercase() == "y";

    input.clear();
    print!("Workspace (e.g., myworkspace): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let workspace = input.trim().to_string();

    input.clear();
    print!("Default repository (optional): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let repo = input.trim().to_string();

    input.clear();
    print!("Default remote [origin]: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let input_remote = input.trim().to_string();
    let remote = if input_remote.is_empty() {
        "origin".to_string()
    } else {
        input_remote
    };

    if local_init {
        // Try to find git repo root, otherwise use current dir
        let target_dir = crate::git::get_repo_root()
            .unwrap_or_else(|_| std::env::current_dir().expect("Failed to get current directory"));

        crate::config::manager::init_local_config(&target_dir, &workspace, &repo, &remote)?;
        ui::success(&format!(
            "Local configuration initialized at {:?}",
            target_dir
        ));
    } else {
        input.clear();
        print!("Default user email (optional): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let user = input.trim().to_string();

        let profile_name = if user.is_empty() {
            "default".to_string()
        } else {
            user.clone()
        };

        // 1. Set the active user (global)
        crate::config::manager::set_config_value("user", &profile_name)?;

        // 2. Set profile values
        crate::config::manager::set_config_value(
            &format!("profile.{}.workspace", profile_name),
            &workspace,
        )?;

        if !repo.is_empty() {
            crate::config::manager::set_config_value(
                &format!("profile.{}.repository", profile_name),
                &repo,
            )?;
        }

        crate::config::manager::set_config_value(
            &format!("profile.{}.remote", profile_name),
            &remote,
        )?;

        if !user.is_empty() {
            crate::config::manager::set_config_value(
                &format!("profile.{}.user", profile_name),
                &user,
            )?;
        }

        ui::success("Configuration initialized");
    }

    Ok(())
}
