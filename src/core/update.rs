// Update Module
// Handles checking for Minecraft version updates on startup
// Only checks if auto_update is enabled in config

use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::core::version::{Version, VersionInfo};
use crate::config::config::Config;

/// Status of an update check
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// No updates available
    UpToDate,
    /// Updates found, contains list of new versions
    UpdatesAvailable(Vec<VersionInfo>),
    /// Update check failed
    Error(String),
    /// Update check was skipped (auto_update disabled)
    Skipped,
}

/// Result of checking a single version for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionUpdateInfo {
    /// Version ID
    pub version_id: String,
    /// Whether the version needs updating
    pub needs_update: bool,
    /// Local file hash (if exists)
    pub local_hash: Option<String>,
    /// Remote file hash
    pub remote_hash: Option<String>,
}

/// Converts SystemTime to seconds since UNIX epoch
fn system_time_to_secs(time: std::time::SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parses ISO 8601 datetime string to seconds since epoch
fn parse_iso_to_secs(iso_str: &str) -> Option<u64> {
    // Simple parsing: extract timestamp from ISO string
    // Format: 2024-01-01T00:00:00+00:00
    chrono::NaiveDateTime::parse_from_str(
        iso_str.trim_end_matches('Z').trim_end_matches("+00:00"),
        "%Y-%m-%dT%H:%M:%S"
    )
    .ok()
    .map(|dt| dt.and_utc().timestamp() as u64)
}

/// Checks if any updates are available for installed versions.
/// Only runs if auto_update is enabled in config.
///
/// # Arguments
/// * `config` - Application configuration
///
/// # Returns
/// * `UpdateStatus` - Result of the update check
pub async fn check_for_updates(config: &Config) -> UpdateStatus {
    // Skip update check if auto_update is disabled
    if !config.auto_update {
        info!("Auto-update disabled, skipping update check");
        return UpdateStatus::Skipped;
    }

    info!("Checking for updates...");
    
    // Fetch version manifest from Mojang
    let manifest = match Version::fetch_manifest().await {
        Ok(manifest) => {
            info!("Version manifest fetched: {} versions", manifest.versions.len());
            manifest
        }
        Err(e) => {
            error!("Failed to fetch version manifest: {}", e);
            return UpdateStatus::Error(format!("Failed to fetch version manifest: {}", e));
        }
    };

    // Get installed versions from local storage
    let installed_versions = get_installed_versions(&config.versions_dir);
    
    if installed_versions.is_empty() {
        debug!("No installed versions found");
        return UpdateStatus::UpToDate;
    }

    // Check if any installed versions have updates
    let mut updates_available = Vec::new();
    
    for installed_id in &installed_versions {
        // Find the version in the manifest
        if let Some(remote_version) = manifest.versions.iter().find(|v| v.version == *installed_id) {
            // Check if version JSON needs updating
            let version_json_path = config.versions_dir.join(installed_id).join(format!("{}.json", installed_id));
            
            if version_json_path.exists() {
                // Compare local and remote timestamps
                if let Ok(local_metadata) = std::fs::metadata(&version_json_path) {
                    let local_secs = local_metadata.modified()
                        .map(system_time_to_secs)
                        .unwrap_or(0);
                    
                    let remote_secs = parse_iso_to_secs(&remote_version.release_time)
                        .unwrap_or(0);
                    
                    if remote_secs > local_secs {
                        info!("Update available for version {}: remote is newer", installed_id);
                        updates_available.push(remote_version.clone());
                    }
                }
            } else {
                // Version JSON doesn't exist, might need re-download
                debug!("Version {} JSON not found, may need download", installed_id);
            }
        }
    }

    if updates_available.is_empty() {
        info!("All installed versions are up to date");
        UpdateStatus::UpToDate
    } else {
        info!("{} version(s) have updates available", updates_available.len());
        UpdateStatus::UpdatesAvailable(updates_available)
    }
}

/// Gets list of installed version IDs from the versions directory.
///
/// # Arguments
/// * `versions_dir` - Path to the versions directory
///
/// # Returns
/// * `Vec<String>` - List of installed version IDs
fn get_installed_versions(versions_dir: &PathBuf) -> Vec<String> {
    let mut versions = Vec::new();
    
    if !versions_dir.exists() {
        return versions;
    }
    
    if let Ok(entries) = std::fs::read_dir(versions_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Check if version JSON exists
                    let json_path = entry.path().join(format!("{}.json", name));
                    if json_path.exists() {
                        versions.push(name.to_string());
                    }
                }
            }
        }
    }
    
    debug!("Found {} installed versions", versions.len());
    versions
}

/// Checks if a specific version needs updating.
///
/// # Arguments
/// * `version_id` - Version ID to check
/// * `config` - Application configuration
///
/// # Returns
/// * `Option<VersionUpdateInfo>` - Update info if version exists
pub async fn check_version_update(version_id: &str, config: &Config) -> Option<VersionUpdateInfo> {
    let version_dir = config.versions_dir.join(version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));
    
    // Check if version is installed
    if !json_path.exists() {
        return None;
    }
    
    // Fetch latest manifest
    let manifest = match Version::fetch_manifest().await {
        Ok(m) => m,
        Err(_) => return None,
    };
    
    // Find version in manifest
    let remote_version = manifest.versions.iter().find(|v| v.version == version_id)?;
    
    // Compare timestamps
    let needs_update = if let Ok(local_metadata) = std::fs::metadata(&json_path) {
        let local_secs = local_metadata.modified()
            .map(system_time_to_secs)
            .unwrap_or(0);
        
        let remote_secs = parse_iso_to_secs(&remote_version.release_time)
            .unwrap_or(0);
        
        remote_secs > local_secs
    } else {
        false
    };
    
    Some(VersionUpdateInfo {
        version_id: version_id.to_string(),
        needs_update,
        local_hash: None, // TODO: Implement hash comparison
        remote_hash: None,
    })
}