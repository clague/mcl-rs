// Version Module
// Handles Minecraft version manifest fetching and version data structures
// Version manifest is fetched from Mojang's official API

use serde::{Deserialize, Serialize};
use log::{info, error, debug};
use rand::RngExt;

/// Mojang version manifest API endpoint
const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

/// Top-level version manifest structure from Mojang API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Latest release and snapshot version IDs
    pub latest: LatestVersion,
    /// List of all available versions
    pub versions: Vec<VersionInfo>,
}

/// Contains the IDs of the latest release and snapshot versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersion {
    /// Latest release version ID (e.g., "1.20.4")
    pub release: String,
    /// Latest snapshot version ID (e.g., "24w03b")
    pub snapshot: String,
}

/// Basic version information from the manifest list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version ID (e.g., "1.20.4", "24w03b")
    pub id: String,
    /// User-defined display name (defaults to version ID)
    #[serde(default)]
    pub display_name: String,
    /// Minecraft item name for version icon (e.g., "diamond", "emerald")
    #[serde(default = "random_icon")]
    pub icon_name: String,
    /// Version type: "release", "snapshot", "old_beta", or "old_alpha"
    #[serde(rename = "type")]
    pub version_type: String,
    /// URL to the version's detailed JSON file
    pub url: String,
    /// Last modification time (ISO 8601 format)
    pub time: String,
    /// Release time (ISO 8601 format)
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

/// Iconic Minecraft mob/entity heads for version icons
pub const MINECRAFT_ICONS: &[&str] = &[
    "creeper", "zombie", "skeleton", "enderman", "spider",
    "blaze", "ghast", "witch", "wither_skeleton", "ender_dragon",
    "steve", "alex", "piglin", "warden", "breeze",
    "pig", "cow", "chicken", "sheep", "villager",
];

/// Get a random icon name from the list
pub fn random_icon() -> String {
    let mut rng = rand::rng();
    let index = rng.random_range(0..MINECRAFT_ICONS.len());
    MINECRAFT_ICONS[index].to_string()
}

/// Detailed version information fetched from the version JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Version ID
    pub id: String,
    /// Display name (same as ID, may be missing in JSON)
    #[serde(default)]
    pub name: String,
    /// Version type (Release, Snapshot, etc.)
    #[serde(rename = "type")]
    pub version_type_raw: String,
    /// URL to this version's JSON
    #[serde(default)]
    pub url: String,
    /// Assets version ID
    #[serde(default)]
    pub assets: String,
    /// Main Java class to launch
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Legacy Minecraft arguments (pre-1.13)
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    /// Modern arguments structure (1.13+)
    #[serde(rename = "arguments")]
    pub arguments: Option<Arguments>,
    /// Required libraries
    pub libraries: Vec<Library>,
    /// Download URLs for client/server JARs
    pub downloads: Downloads,
    /// Asset index information
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    /// Compliance level
    #[serde(rename = "complianceLevel", default)]
    pub compliance_level: Option<u32>,
    /// Java version requirement
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    /// Logging configuration
    #[serde(default)]
    pub logging: Option<LoggingConfig>,
    /// Release time
    #[serde(rename = "releaseTime", default)]
    pub release_time: String,
    /// Last modified time
    #[serde(default)]
    pub time: String,
    /// Minimum launcher version
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: Option<u32>,
}

/// Java version requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    /// Java component name (e.g., "java-runtime-gamma")
    pub component: String,
    /// Major version number
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Client logging configuration
    pub client: Option<LoggingClient>,
}

/// Client logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    /// Logging argument template
    pub argument: String,
    /// Logging configuration file
    pub file: LoggingFile,
    /// Logging type
    #[serde(rename = "type")]
    pub logging_type: String,
}

/// Logging configuration file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    /// File ID
    pub id: String,
    /// SHA1 hash
    pub sha1: String,
    /// File size
    pub size: u64,
    /// Download URL
    pub url: String,
}

/// Game and JVM arguments for launching (1.13+ format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    /// Game arguments
    pub game: Vec<serde_json::Value>,
    /// JVM arguments
    pub jvm: Vec<serde_json::Value>,
    /// Default user JVM arguments (new format)
    #[serde(rename = "default-user-jvm", default)]
    pub default_user_jvm: Option<Vec<serde_json::Value>>,
}

/// Asset index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    /// Asset index ID (e.g., "12")
    pub id: String,
    /// URL to the asset index JSON
    pub url: String,
    /// SHA1 hash of the asset index
    #[serde(rename = "sha1")]
    pub sha1: String,
    /// Size in bytes
    pub size: u64,
    /// Total size of all assets
    #[serde(rename = "totalSize", default)]
    pub total_size: Option<u64>,
}

/// Version type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionType {
    /// Stable release
    Release,
    /// Development snapshot
    Snapshot,
    /// Legacy beta version
    OldBeta,
    /// Legacy alpha version
    OldAlpha,
}

impl VersionType {
    /// Converts a string to VersionType
    pub fn from_str(s: &str) -> Self {
        match s {
            "release" => VersionType::Release,
            "snapshot" => VersionType::Snapshot,
            "old_beta" => VersionType::OldBeta,
            "old_alpha" => VersionType::OldAlpha,
            _ => VersionType::Release,
        }
    }

    /// Returns the display name for this version type
    pub fn display_name(&self) -> &str {
        match self {
            VersionType::Release => "Release",
            VersionType::Snapshot => "Snapshot",
            VersionType::OldBeta => "Old Beta",
            VersionType::OldAlpha => "Old Alpha",
        }
    }
}

/// Library dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// Maven-style library name (e.g., "com.example:library:1.0")
    pub name: String,
    /// Download information for the library
    pub downloads: Option<LibraryDownloads>,
    /// Rules for when this library should be loaded
    pub rules: Option<Vec<Rule>>,
    /// Native library classifiers for different platforms
    pub natives: Option<std::collections::HashMap<String, String>>,
    /// Extract rules
    pub extract: Option<ExtractRules>,
}

/// Extract rules for native libraries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRules {
    /// Files to exclude during extraction
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Library download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    /// Main artifact JAR
    pub artifact: Option<Artifact>,
    /// Platform-specific classifiers (e.g., natives-linux, natives-windows)
    pub classifiers: Option<std::collections::HashMap<String, Artifact>>,
}

/// A downloadable artifact (JAR file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Path within the libraries directory
    pub path: String,
    /// Download URL
    pub url: String,
    /// SHA1 hash for verification
    pub sha1: String,
    /// File size in bytes
    pub size: u64,
}

/// Rule for conditional library loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Action: "allow" or "disallow"
    pub action: String,
    /// OS-specific conditions (optional)
    pub os: Option<OsRule>,
    /// Feature-specific conditions (optional)
    pub features: Option<FeatureRule>,
}

/// OS-specific rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    /// OS name: "windows", "linux", "osx"
    pub name: Option<String>,
    /// Architecture: "x86", "x86_64"
    pub arch: Option<String>,
    /// Version range
    #[serde(rename = "versionRange")]
    pub version_range: Option<VersionRange>,
}

/// Version range for OS rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    /// Minimum version
    pub min: Option<String>,
    /// Maximum version
    pub max: Option<String>,
}

/// Feature-specific rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRule {
    /// Is demo user
    #[serde(rename = "is_demo_user")]
    pub is_demo_user: Option<bool>,
    /// Has custom resolution
    #[serde(rename = "has_custom_resolution")]
    pub has_custom_resolution: Option<bool>,
    /// Has quick plays support
    #[serde(rename = "has_quick_plays_support")]
    pub has_quick_plays_support: Option<bool>,
    /// Is quick play singleplayer
    #[serde(rename = "is_quick_play_singleplayer")]
    pub is_quick_play_singleplayer: Option<bool>,
    /// Is quick play multiplayer
    #[serde(rename = "is_quick_play_multiplayer")]
    pub is_quick_play_multiplayer: Option<bool>,
    /// Is quick play realms
    #[serde(rename = "is_quick_play_realms")]
    pub is_quick_play_realms: Option<bool>,
}

/// Download URLs for game files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    /// Client JAR download
    pub client: Download,
    /// Server JAR download (optional)
    pub server: Option<Download>,
    /// Client mappings for deobfuscation (optional, 1.14.4+)
    #[serde(rename = "client_mappings")]
    pub client_mappings: Option<Download>,
    /// Server mappings for deobfuscation (optional, 1.14.4+)
    #[serde(rename = "server_mappings")]
    pub server_mappings: Option<Download>,
}

/// A single downloadable file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    /// Download URL
    pub url: String,
    /// SHA1 hash for verification
    pub sha1: String,
    /// File size in bytes
    pub size: u64,
}

impl Version {
    /// Returns the version type as an enum
    pub fn version_type(&self) -> VersionType {
        VersionType::from_str(&self.version_type_raw)
    }
    
    /// Fetches the version manifest from Mojang's API.
    /// Returns a list of all available Minecraft versions.
    ///
    /// # Returns
    /// * `Ok(VersionManifest)` - The complete version manifest
    /// * `Err(String)` - Error message if the fetch fails
    pub async fn fetch_manifest() -> Result<VersionManifest, String> {
        info!("Fetching version manifest from Mojang...");
        let client = create_http_client()?;
        let response = client
            .get(VERSION_MANIFEST_URL)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch version manifest: {}", e);
                format!("Failed to fetch version manifest: {}", e)
            })?;

        let manifest: VersionManifest = response
            .json()
            .await
            .map_err(|e| {
                error!("Failed to parse version manifest: {}", e);
                format!("Failed to parse version manifest: {}", e)
            })?;

        info!("Version manifest fetched: {} versions available", manifest.versions.len());
        info!("Latest release: {}, latest snapshot: {}", manifest.latest.release, manifest.latest.snapshot);
        Ok(manifest)
    }

    /// Fetches detailed version information from a version's JSON URL.
    ///
    /// # Arguments
    /// * `version_info` - Basic version info containing the JSON URL
    ///
    /// # Returns
    /// * `Ok(Version)` - Detailed version information
    /// * `Err(String)` - Error message if the fetch fails
    pub async fn fetch_version_detail(version_info: &VersionInfo) -> Result<Version, String> {
        info!("Fetching version detail for: {} ({})", version_info.id, version_info.version_type);
        debug!("Version URL: {}", version_info.url);
        
        let client = create_http_client()?;
        let response = client
            .get(&version_info.url)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch version detail for {}: {}", version_info.id, e);
                format!("Failed to fetch version detail: {}", e)
            })?;
        
        let status = response.status();
        debug!("Response status: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to fetch version detail: HTTP {} - {}", status, error_text);
            return Err(format!("Failed to fetch version detail: HTTP {}", status));
        }
        
        // Get response text for debugging
        let response_text = response.text().await.map_err(|e| {
            error!("Failed to read response body for {}: {}", version_info.id, e);
            format!("Failed to read response body: {}", e)
        })?;
        
        debug!("Response body length: {} bytes", response_text.len());
        debug!("Response body preview: {}...", &response_text[..std::cmp::min(500, response_text.len())]);
        
        // Parse JSON with detailed error reporting
        let mut version: Version = serde_json::from_str(&response_text).map_err(|e| {
            error!("Failed to parse version detail for {}: {}", version_info.id, e);
            error!("Parse error details: {:?}", e);
            
            // Extract error position and show context
            let error_str = e.to_string();
            if let Some(col_str) = error_str.split("column ").nth(1) {
                if let Ok(col) = col_str.parse::<usize>() {
                    let start = col.saturating_sub(200);
                    let end = std::cmp::min(col + 200, response_text.len());
                    let context = &response_text[start..end];
                    error!("JSON context around column {}:", col);
                    error!("...{}...", context);
                    error!("Error position marked below:");
                    let marker_pos = col - start;
                    let marker = format!("{}^--- ERROR HERE", " ".repeat(marker_pos));
                    error!("{}", marker);
                }
            }
            
            // Also try to pretty print the problematic area
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                // Try to find the path to the error
                if error_str.contains("missing field") {
                    if let Some(field) = error_str.split("missing field `").nth(1) {
                        if let Some(field_name) = field.split("`").next() {
                            error!("Missing field: '{}'", field_name);
                            // Try to find where this field should be
                            find_missing_field_path(&value, field_name, "");
                        }
                    }
                }
            }
            
            format!("Failed to parse version detail: {}", e)
        })?;

        // Fill in basic info from the manifest if not present
        if version.url.is_empty() {
            version.url = version_info.url.clone();
        }
        
        info!("Version detail fetched: {} (main class: {}, type: {})", 
            version.id, version.main_class, version.version_type_raw);
        debug!("Version has {} libraries", version.libraries.len());
        
        Ok(version)
    }

    /// Downloads the version files (placeholder implementation).
    ///
    /// # Arguments
    /// * `_progress_callback` - Callback function for download progress (0.0 to 1.0)
    ///
    /// # Returns
    /// * `Ok(())` - Download completed successfully
    /// * `Err(String)` - Error message if download fails
    pub async fn download(&self, _progress_callback: impl Fn(f32)) -> Result<(), String> {
        // TODO: Implement actual download logic
        // This should download:
        // 1. Client JAR
        // 2. Libraries
        // 3. Assets
        // 4. Logging config
        for i in 0..100 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            _progress_callback(i as f32 / 100.0);
        }
        Ok(())
    }
}

impl VersionManifest {
    /// Filters versions by type and returns a limited number of results.
    ///
    /// # Arguments
    /// * `version_type` - Optional version type filter (None for all)
    /// * `limit` - Optional maximum number of results
    ///
    /// # Returns
    /// * `Vec<&VersionInfo>` - Filtered list of version info references
    pub fn filter_versions(&self, version_type: Option<VersionType>, limit: Option<usize>) -> Vec<&VersionInfo> {
        let filtered: Vec<&VersionInfo> = self.versions.iter()
            .filter(|v| {
                if let Some(ref vtype) = version_type {
                    v.version_type == vtype.display_name().to_lowercase() || 
                    (vtype == &VersionType::Release && v.version_type == "release") ||
                    (vtype == &VersionType::Snapshot && v.version_type == "snapshot")
                } else {
                    true
                }
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect();
        filtered
    }
}

/// Recursively searches for where a missing field should be in the JSON structure
fn find_missing_field_path(value: &serde_json::Value, field_name: &str, path: &str) {
    match value {
        serde_json::Value::Object(map) => {
            // Check if this object has the field
            if map.contains_key(field_name) {
                error!("Field '{}' found at path: {}.{}", field_name, path, field_name);
            }
            // Check nested objects
            for (key, val) in map {
                let new_path = format!("{}.{}", path, key);
                find_missing_field_path(val, field_name, &new_path);
            }
        }
        serde_json::Value::Array(arr) => {
            // Check array elements
            for (i, val) in arr.iter().enumerate() {
                let new_path = format!("{}[{}]", path, i);
                find_missing_field_path(val, field_name, &new_path);
            }
        }
        _ => {}
    }
}

/// Creates an HTTP client with system proxy support
fn create_http_client() -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .pool_max_idle_per_host(16)
        .timeout(std::time::Duration::from_secs(30));
    
    // Check for system proxy settings
    let http_proxy = std::env::var("HTTP_PROXY")
        .or_else(|_| std::env::var("http_proxy"))
        .ok();
    let https_proxy = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .ok();
    let all_proxy = std::env::var("ALL_PROXY")
        .or_else(|_| std::env::var("all_proxy"))
        .ok();
    
    // Apply proxy settings
    if let Some(proxy_url) = all_proxy.or_else(|| https_proxy.or(http_proxy)) {
        info!("Using proxy: {}", proxy_url);
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;
        builder = builder.proxy(proxy);
    }
    
    builder.build().map_err(|e| format!("Failed to build HTTP client: {}", e))
}