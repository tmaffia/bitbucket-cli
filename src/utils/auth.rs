/// Keyring authentication utilities
use anyhow::{Context, Result};
use keyring::Entry;

/// Save credentials to the system keyring
pub fn save_credentials(username: &str, password: &str) -> Result<()> {
    let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)
        .context("Failed to create keyring entry")?;

    entry
        .set_password(password)
        .context("Failed to save password to keyring")?;

    Ok(())
}

/// Retrieve credentials from the system keyring
pub fn get_credentials(username: &str) -> Result<String> {
    let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)
        .context("Failed to create keyring entry")?;

    let password = entry
        .get_password()
        .context("No password found in keyring")?;

    Ok(password)
}

/// Delete credentials from the system keyring
pub fn delete_credentials(username: &str) -> Result<()> {
    let entry = Entry::new(crate::constants::KEYRING_SERVICE_NAME, username)
        .context("Failed to create keyring entry")?;

    entry
        .delete_credential()
        .context("Failed to delete credentials from keyring")
}

/// Check if credentials exist for a user
pub fn has_credentials(username: &str) -> bool {
    get_credentials(username).is_ok()
}
