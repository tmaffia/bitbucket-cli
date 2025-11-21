use anyhow::Result;
use clap::{Args, Subcommand};
use keyring::Entry;
use std::io::{self, Write};

#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Login to Bitbucket
    Login {
        /// Workspace/Username
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Logout
    Logout,
    /// Check auth status
    Status,
}

pub async fn handle(args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommands::Login { username } => {
            let username = match username {
                Some(u) => u,
                None => {
                    print!("Bitbucket Username: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            let password = rpassword::prompt_password("App Password: ")?;

            // Verify credentials
            println!("Verifying credentials...");
            let client = crate::api::client::BitbucketClient::new(
                None,
                Some((username.clone(), password.clone())),
            )?;

            match client.get_current_user().await {
                Ok(user) => {
                    println!(
                        "Successfully authenticated as: {} ({})",
                        user.display_name, user.uuid
                    );

                    let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, &username)?;
                    entry.set_password(&password)?;

                    println!("Credentials saved to keyring for user: '{}'", username);

                    // Verify write
                    match entry.get_password() {
                        Ok(_) => {
                            println!("Verification: Password successfully retrieved from keyring.")
                        }
                        Err(e) => println!(
                            "Verification: Failed to retrieve password from keyring immediately: {}",
                            e
                        ),
                    }

                    // Update config.toml
                    crate::config::manager::update_global_user(&username)?;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Authentication failed: {}", e));
                }
            }
        }
        AuthCommands::Logout => {
            // Try to get username from config if not provided (though logout doesn't take args currently)
            // The current implementation asks for username.
            // Let's try to get the current user from config first as default.

            let config = crate::config::manager::AppConfig::load().ok();
            let _default_user = config.as_ref().and_then(|c| c.get_default_user());

            // Actually, let's look at the struct:
            // AuthCommands::Logout takes no args.

            print!("Username to logout: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let user = input.trim();

            if user.is_empty() {
                println!("No username provided.");
                return Ok(());
            }

            let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, user)?;
            match entry.delete_credential() {
                Ok(_) => println!("Logged out {}", user),
                Err(e) => println!("Error logging out: {}", e),
            }
        }
        AuthCommands::Status => {
            println!("Checking status...");
            let config = crate::config::manager::AppConfig::load()?;
            let profile = config.get_active_profile();

            if let Some(username) = profile.and_then(|p| p.user.as_ref()) {
                println!("Configured user: '{}'", username);
                // Check if we have password
                if let Ok(entry) = Entry::new(crate::constants::KEYRING_SERVICE_NAME, username) {
                    match entry.get_password() {
                        Ok(_) => {
                            println!("Credentials found in keyring.");
                            // Verify
                            let client = crate::api::client::BitbucketClient::new(profile, None)?;
                            match client.get_current_user().await {
                                Ok(user) => println!(
                                    "Status: Authenticated as {} ({})",
                                    user.display_name, user.uuid
                                ),
                                Err(e) => println!("Status: Authentication failed: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("Status: No password in keyring. Error: {}", e);
                        }
                    }
                } else {
                    println!("Status: Keyring error.");
                }
            } else {
                println!("Status: No user configured in active profile.");
            }
        }
    }
    Ok(())
}
