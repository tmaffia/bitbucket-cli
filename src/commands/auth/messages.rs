/// Authentication-related user-facing messages
pub mod auth {
    pub const EMPTY_EMAIL: &str = "Email cannot be empty";
    pub const EMPTY_API_TOKEN: &str = "API Token cannot be empty";

    pub const LOGIN_REQUIRED: &str = "Run 'bb auth login' to authenticate";
    pub const VERIFYING_CREDENTIALS: &str = "Verifying credentials...";
    pub const AUTH_SUCCESS: &str = "Authentication successful!";
    pub const CREDENTIALS_SAVED: &str = "Credentials saved for user '{}'";
    pub const LOGOUT_USER: &str = "Logging out user: {}";
    pub const LOGGED_OUT: &str = "Logged out {}";
    pub const NO_USERNAME: &str = "No username provided";
    pub const CHECKING_STATUS: &str = "Checking authentication status...";
    pub const AUTHENTICATED: &str = "Authenticated";
    pub const NOT_AUTHENTICATED: &str = "Not authenticated";
}
