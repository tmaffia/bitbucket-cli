use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use std::io::{self, Write};

use crate::api::models::User;
use crate::config::manager::ProfileConfig;
use crate::display::ui;

#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Login to Bitbucket
    Login,
    /// Logout
    Logout,
    /// Check authentication status
    Status,
}

/// Check if user is authenticated by verifying credentials and API access
async fn get_authenticated_user(profile: Option<&ProfileConfig>) -> Result<User> {
    let username = profile
        .and_then(|p| p.user.as_ref())
        .ok_or_else(|| anyhow!("No user configured in active profile"))?;

    // Verify password exists in keyring
    let password = crate::utils::auth::get_credentials(username)?;

    let base_url = profile
        .and_then(|p| p.api_url.clone())
        .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

    // Verify credentials against API
    let client = crate::api::client::BitbucketClient::new(
        base_url,
        Some((username.clone(), password)),
    )?;
    client
        .get_current_user()
        .await
        .context("API authentication failed")
}

/// Attempt to log in with provided credentials
async fn check_login(
    profile: Option<&ProfileConfig>,
    username: &str,
    password: &str,
) -> Result<User> {
    let base_url = profile
        .and_then(|p| p.api_url.clone())
        .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

    // Verify credentials work with API first
    let client = crate::api::client::BitbucketClient::new(
        base_url,
        Some((username.to_string(), password.to_string())),
    )?;
    let user = client
        .get_current_user()
        .await
        .context("Authentication failed - check username and password")?;

    // Save to keyring after verification
    crate::utils::auth::save_credentials(username, password)?;

    Ok(user)
}

/// Delete credentials from keyring
fn check_logout(username: &str) -> Result<()> {
    crate::utils::auth::delete_credentials(username)?;

    Ok(())
}

// TODO: Improve view layer of this command.
pub async fn handle(args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommands::Login => {
            print!("Email: ");
            io::stdout().flush()?;
            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let username = username.trim();

            if username.is_empty() {
                ui::error("Email cannot be empty");
                return Ok(());
            }

            print!("App Password: ");
            io::stdout().flush()?;
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            if password.is_empty() {
                ui::error("App Password cannot be empty");
                return Ok(());
            }

            ui::info("Verifying credentials...");

            let config = crate::config::manager::AppConfig::load().ok();
            let profile = config.as_ref().and_then(|c| c.get_active_profile());

            match check_login(profile, username, &password).await {
                Ok(user) => {
                    ui::success("Authentication successful!");
                    ui::info(&format!("Credentials saved for user '{}'", username));

                    let mut user_info =
                        vec![("Display Name", user.display_name), ("UUID", user.uuid)];
                    if let Some(nickname) = user.nickname {
                        user_info.push(("Nickname", nickname));
                    }

                    crate::utils::formatting::print_key_value_table(user_info);
                }
                Err(e) => {
                    ui::error(&format!("Login failed: {:#}", e));
                }
            }
        }
        AuthCommands::Logout => {
            let config = crate::config::manager::AppConfig::load().ok();
            let default_user = config.as_ref().and_then(|c| c.get_default_user());

            let username = if let Some(user) = default_user.as_ref() {
                ui::info(&format!("Logging out user: {}", user));
                user.clone()
            } else {
                print!("Username to logout: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let user = input.trim();

                if user.is_empty() {
                    ui::error("No username provided");
                    return Ok(());
                }
                user.to_string()
            };

            match check_logout(&username) {
                Ok(_) => ui::success(&format!("Logged out {}", username)),
                Err(e) => ui::error(&format!("Logout failed: {:#}", e)),
            }
        }
        AuthCommands::Status => {
            ui::info("Checking authentication status...");

            let config = crate::config::manager::AppConfig::load()?;
            let profile = config.get_active_profile();

            match get_authenticated_user(profile).await {
                Ok(user) => {
                    ui::success("Authenticated");
                    let mut user_info =
                        vec![("Display Name", user.display_name), ("UUID", user.uuid)];
                    if let Some(nickname) = user.nickname {
                        user_info.push(("Nickname", nickname));
                    }

                    crate::utils::formatting::print_key_value_table(user_info);
                }
                Err(e) => {
                    ui::error(&format!("Not authenticated: {:#}", e));
                    ui::info("Run 'bb auth login' to authenticate");
                }
            }
        }
    }

    Ok(())
}
