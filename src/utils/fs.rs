// File System Utilities
// Async file operations for reading/writing game files and configuration

use std::path::Path;
use tokio::fs;

/// Ensures a directory exists, creating it and any parent directories if needed.
///
/// # Arguments
/// * `path` - Directory path to ensure exists
///
/// # Returns
/// * `Ok(())` - Directory exists or was created
/// * `Err(String)` - Error message if creation fails
pub async fn ensure_dir_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Reads and parses a JSON file.
///
/// # Arguments
/// * `path` - Path to the JSON file
///
/// # Returns
/// * `Ok(T)` - Parsed JSON data
/// * `Err(String)` - Error message if read or parse fails
pub async fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

/// Serializes data to JSON and writes to a file.
///
/// # Arguments
/// * `path` - Path to write the JSON file
/// * `data` - Data to serialize
///
/// # Returns
/// * `Ok(())` - File written successfully
/// * `Err(String)` - Error message if write fails
pub async fn write_json_file<T: serde::Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(path, content).await.map_err(|e| e.to_string())
}

/// Computes SHA1 hash of a file.
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Ok(String)` - Hex-encoded SHA1 hash
/// * `Err(String)` - Error message if read fails
pub async fn file_hash(path: &Path) -> Result<String, String> {
    use sha1::{Sha1, Digest};
    let content = fs::read(path).await.map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}