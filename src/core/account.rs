// Account Module
// Stores Minecraft account information and session data

use serde::{Deserialize, Serialize};

/// Represents a Minecraft account with authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Minecraft username
    pub username: String,
    /// Player UUID (without dashes)
    pub uuid: String,
    /// Minecraft access token for API calls
    pub access_token: String,
    /// Microsoft refresh token for re-authentication
    pub refresh_token: String,
}

impl Account {
    /// Creates a new Account instance
    ///
    /// # Arguments
    /// * `username` - Minecraft username
    /// * `uuid` - Player UUID
    /// * `access_token` - Minecraft access token
    /// * `refresh_token` - Microsoft refresh token
    pub fn new(username: String, uuid: String, access_token: String, refresh_token: String) -> Self {
        Self {
            username,
            uuid,
            access_token,
            refresh_token,
        }
    }
}