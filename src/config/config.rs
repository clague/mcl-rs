// Configuration Module
// Handles application settings, session, and version list persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use dirs::config_dir;
use log::{info, error};

use crate::core::version::VersionInfo;

/// Saved session data for auto-login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    /// Minecraft username
    pub username: String,
    /// Player UUID
    pub uuid: String,
    /// Minecraft access token
    pub access_token: String,
    /// Microsoft refresh token for re-authentication
    pub refresh_token: String,
}

/// Information about an installed mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModInfo {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub version_id: String,
    pub version_number: String,
    pub filename: String,
    pub enabled: bool,
    pub loaders: Vec<String>,
}

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to Java executable (None = auto-detect)
    pub java_path: Option<String>,
    /// Memory allocation in MB (None = use default 2048)
    #[serde(default)]
    pub memory: Option<u32>,
    /// Whether to auto-update game files
    pub auto_update: bool,
    /// Base game directory
    pub game_dir: PathBuf,
    /// Directory for version-specific files
    pub versions_dir: PathBuf,
    /// Directory for shared assets
    pub assets_dir: PathBuf,
    /// Saved session for auto-login (None if not logged in)
    #[serde(default)]
    pub saved_session: Option<SavedSession>,
    /// List of added Minecraft instances
    #[serde(default, alias = "added_versions")]
    pub added_instances: Vec<VersionInfo>,
    /// UI language preference ("en" or "zh")
    #[serde(default = "default_language")]
    pub language: String,
    /// Maximum concurrent HTTP connections (None = use default 32)
    #[serde(default)]
    pub max_connections: Option<usize>,
    /// Directory for mod files (defaults to game_dir/mods)
    #[serde(default)]
    pub mods_dir: Option<PathBuf>,
    /// Per-version installed mods (version_uuid -> list of mods)
    #[serde(default)]
    pub installed_mods: HashMap<String, Vec<InstalledModInfo>>,
    /// Optional user agent for Modrinth API requests
    #[serde(default)]
    pub modrinth_user_agent: Option<String>,
}

/// Default language is English
fn default_language() -> String {
    "en".to_string()
}

impl Config {
    /// Returns the path to the config directory
    fn config_dir_path() -> PathBuf {
        config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mcl-rs")
    }

    /// Returns the path to the config file
    fn config_file_path() -> PathBuf {
        Self::config_dir_path().join("config.json")
    }

    /// Loads configuration from the config file.
    /// Returns default config if file doesn't exist.
    pub fn load() -> Self {
        let config_path = Self::config_file_path();
        
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            info!("Configuration loaded from {:?}", config_path);
                            config
                        }
                        Err(e) => {
                            error!("Failed to parse config file: {}, using defaults", e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read config file: {}, using defaults", e);
                    Self::default()
                }
            }
        } else {
            info!("No config file found, using defaults");
            Self::default()
        }
    }

    /// Saves configuration to the config file.
    ///
    /// # Returns
    /// * `Ok(())` - Configuration saved successfully
    /// * `Err(String)` - Error message if save fails
    pub fn save(&self) -> Result<(), String> {
        let config_dir = Self::config_dir_path();
        let config_path = Self::config_file_path();
        
        // Create parent directories if they don't exist
        std::fs::create_dir_all(&config_dir).map_err(|e| {
            error!("Failed to create config directory: {}", e);
            e.to_string()
        })?;
        
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            error!("Failed to serialize config: {}", e);
            e.to_string()
        })?;
        
        std::fs::write(&config_path, content).map_err(|e| {
            error!("Failed to write config file: {}", e);
            e.to_string()
        })?;
        
        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Saves the current session for auto-login
    pub fn save_session(&mut self, username: String, uuid: String, access_token: String, refresh_token: String) {
        self.saved_session = Some(SavedSession {
            username,
            uuid,
            access_token,
            refresh_token,
        });
        if let Err(e) = self.save() {
            error!("Failed to save session: {}", e);
        }
    }

    /// Clears the saved session (logout)
    pub fn clear_session(&mut self) {
        self.saved_session = None;
        if let Err(e) = self.save() {
            error!("Failed to save config after clearing session: {}", e);
        }
    }

    /// Adds an instance to the saved list
    pub fn add_instance(&mut self, version: VersionInfo) {
        if !self.added_instances.iter().any(|v| v.version == version.version) {
            self.added_instances.push(version);
            if let Err(e) = self.save() {
                error!("Failed to save instance list: {}", e);
            }
        }
    }

    /// Removes an instance from the saved list
    pub fn remove_instance(&mut self, version_id: &str) {
        self.added_instances.retain(|v| v.version != version_id);
        if let Err(e) = self.save() {
            error!("Failed to save instance list after removal: {}", e);
        }
    }

    /// Saves the language preference
    pub fn save_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        if let Err(e) = self.save() {
            error!("Failed to save language preference: {}", e);
        }
    }

    /// Adds a mod to the installed list for the given version
    pub fn add_mod(&mut self, version_uuid: &str, mod_info: InstalledModInfo) {
        let mods = self.installed_mods.entry(version_uuid.to_string()).or_default();
        if !mods.iter().any(|m| m.project_id == mod_info.project_id) {
            mods.push(mod_info);
            if let Err(e) = self.save() {
                error!("Failed to save config after adding mod: {}", e);
            }
        }
    }

    /// Removes a mod from the installed list for the given version by project_id
    pub fn remove_mod(&mut self, version_uuid: &str, project_id: &str) {
        if let Some(mods) = self.installed_mods.get_mut(version_uuid) {
            mods.retain(|m| m.project_id != project_id);
            if let Err(e) = self.save() {
                error!("Failed to save config after removing mod: {}", e);
            }
        }
    }

    /// Toggles the enabled state of a mod for the given version by project_id
    pub fn toggle_mod(&mut self, version_uuid: &str, project_id: &str) {
        if let Some(mods) = self.installed_mods.get_mut(version_uuid) {
            if let Some(mod_info) = mods.iter_mut().find(|m| m.project_id == project_id) {
                mod_info.enabled = !mod_info.enabled;
                if let Err(e) = self.save() {
                    error!("Failed to save config after toggling mod: {}", e);
                }
            }
        }
    }

    /// Returns the list of installed mods for the given version
    pub fn get_mods(&self, version_uuid: &str) -> Vec<InstalledModInfo> {
        self.installed_mods
            .get(version_uuid)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for Config {
    /// Creates default configuration
    fn default() -> Self {
        let game_dir = config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mcl-rs");
        
        Self {
            java_path: None,
            memory: None,
            auto_update: true,
            game_dir: game_dir.clone(),
            versions_dir: game_dir.join("versions"),
            assets_dir: game_dir.join("assets"),
            saved_session: None,
            added_instances: Vec::new(),
            language: default_language(),
            max_connections: None,
            mods_dir: Some(game_dir.join("mods")),
            installed_mods: HashMap::new(),
            modrinth_user_agent: None,
        }
    }
}