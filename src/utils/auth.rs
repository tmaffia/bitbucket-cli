/// Keyring authentication utilities
use anyhow::{Context, Result};
use keyring::Entry;

/// Create a keyring entry for the given username
fn create_entry(username: &str) -> Result<Entry> {
    Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)
        .context("Failed to create keyring entry")
}

/// Save credentials to the system keyring
///
/// # Arguments
///
/// * `username` - The username to save credentials for
/// * `password` - The password/token to save
///
/// # Example
///
/// ```no_run
/// use bb_cli::utils::auth;
/// auth::save_credentials("user@example.com", "secret_token").unwrap();
/// ```
pub fn save_credentials(username: &str, password: &str) -> Result<()> {
    let entry = create_entry(username)?;

    entry
        .set_password(password)
        .context("Failed to save password to keyring")?;

    Ok(())
}

/// Retrieve credentials from the system keyring
///
/// # Arguments
///
/// * `username` - The username to retrieve credentials for
///
/// # Returns
///
/// Returns the password/token if found, or an error if not found or keyring is inaccessible.
pub fn get_credentials(username: &str) -> Result<String> {
    let entry = create_entry(username)?;

    let password = entry
        .get_password()
        .context("No password found in keyring")?;

    Ok(password)
}

/// Delete credentials from the system keyring
///
/// # Arguments
///
/// * `username` - The username to delete credentials for
pub fn delete_credentials(username: &str) -> Result<()> {
    let entry = create_entry(username)?;

    entry
        .delete_credential()
        .context("Failed to delete credentials from keyring")
}
