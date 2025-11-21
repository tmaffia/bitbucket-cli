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
        AuthCommands::Login { username: _ } => {
            print!("Bitbucket Username: ");
            io::stdout().flush()?;
            let mut username_input = String::new();
            io::stdin().read_line(&mut username_input)?;
            let username = username_input.trim();

            let password = rpassword::prompt_password("App Password: ")?;

            let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)?;
            entry.set_password(&password)?;

            println!("Credentials saved to keyring for user: {}", username);

            // Update config.toml
            crate::config::manager::update_global_user(&username)?;
        }
        AuthCommands::Logout => {
            print!("Username to logout: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let user = input.trim();

            let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, user)?;
            match entry.delete_credential() {
                Ok(_) => println!("Logged out {}", user),
                Err(e) => println!("Error logging out: {}", e),
            }
        }
        AuthCommands::Status => {
            println!("Checking status...");
            // TODO: Check if we have valid credentials
        }
    }
    Ok(())
}
