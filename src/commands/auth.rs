use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use std::io::{self, Write};

use crate::api::models::User;
use crate::config::manager::Profile;
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
async fn get_authenticated_user(profile: Option<&Profile>) -> Result<User> {
    let username = profile
        .and_then(|p| p.user.as_ref())
        .ok_or_else(|| anyhow!("No user configured in active profile"))?;

    // Verify password exists in keyring
    let api_token = crate::utils::auth::get_credentials(username)?;

    let base_url = profile
        .and_then(|p| p.api_url.clone())
        .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

    // Verify credentials against API
    let client =
        crate::api::client::BitbucketClient::new(base_url, Some((username.clone(), api_token)))?;
    client
        .get_current_user()
        .await
        .context("API authentication failed")
}

/// Attempt to log in with provided credentials
async fn check_login(profile: Option<&Profile>, username: &str, api_token: &str) -> Result<User> {
    let base_url = profile
        .and_then(|p| p.api_url.clone())
        .unwrap_or_else(|| crate::constants::DEFAULT_API_URL.to_string());

    // Verify credentials work with API first
    let client = crate::api::client::BitbucketClient::new(
        base_url,
        Some((username.to_string(), api_token.to_string())),
    )?;
    let user = client
        .get_current_user()
        .await
        .context("Authentication failed - check username and password")?;

    // Save to keyring after verification
    crate::utils::auth::save_credentials(username, api_token)?;

    Ok(user)
}

/// Delete credentials from keyring
fn check_logout(username: &str) -> Result<()> {
    crate::utils::auth::delete_credentials(username)?;

    Ok(())
}

mod messages;
use messages::auth as msg;

// TODO: Improve view layer of this command.
use crate::context::AppContext;

pub async fn handle(_ctx: &AppContext, args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommands::Login => {
            print!("Email: ");
            io::stdout().flush()?;
            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let username = username.trim();

            if username.is_empty() {
                ui::error(msg::EMPTY_EMAIL);
                return Ok(());
            }

            print!("API Token: ");
            io::stdout().flush()?;
            let mut api_token = String::new();
            io::stdin().read_line(&mut api_token)?;
            let api_token = api_token.trim();

            if api_token.is_empty() {
                ui::error(msg::EMPTY_API_TOKEN);
                return Ok(());
            }

            ui::info(msg::VERIFYING_CREDENTIALS);

            let config = crate::config::manager::ProfileConfig::load().ok();
            let profile = config.as_ref().and_then(|c| c.get_active_profile());

            match check_login(profile, username, api_token).await {
                Ok(user) => {
                    ui::success(msg::AUTH_SUCCESS);
                    ui::info(&msg::CREDENTIALS_SAVED.replace("{}", username));

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
            let config = crate::config::manager::ProfileConfig::load().ok();
            let default_user = config.as_ref().and_then(|c| c.get_default_user());

            let username = if let Some(user) = default_user.as_ref() {
                ui::info(&msg::LOGOUT_USER.replace("{}", user));
                user.clone()
            } else {
                print!("Username to logout: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let user = input.trim();

                if user.is_empty() {
                    ui::error(msg::NO_USERNAME);
                    return Ok(());
                }
                user.to_string()
            };

            match check_logout(&username) {
                Ok(_) => ui::success(&msg::LOGGED_OUT.replace("{}", &username)),
                Err(e) => ui::error(&format!("Logout failed: {:#}", e)),
            }
        }
        AuthCommands::Status => {
            ui::info(msg::CHECKING_STATUS);

            let config = crate::config::manager::ProfileConfig::load()?;
            let profile = config.get_active_profile();

            match get_authenticated_user(profile).await {
                Ok(user) => {
                    ui::success(msg::AUTHENTICATED);
                    let mut user_info =
                        vec![("Display Name", user.display_name), ("UUID", user.uuid)];
                    if let Some(nickname) = user.nickname {
                        user_info.push(("Nickname", nickname));
                    }

                    crate::utils::formatting::print_key_value_table(user_info);
                }
                Err(e) => {
                    ui::error(&format!("{}: {:#}", msg::NOT_AUTHENTICATED, e));
                    ui::info(msg::LOGIN_REQUIRED);
                }
            }
        }
    }

    Ok(())
}
