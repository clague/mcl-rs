// Launch Module
// Handles launching Minecraft game process
// Downloads required files and starts the game

use log::{info, error, warn, debug};
use std::path::PathBuf;
use std::process::Command;

use crate::core::auth::AccountSession;
use crate::core::version::{Version, VersionInfo, Library};
use crate::core::download::DownloadManager;
use crate::config::config::Config;

/// Launch configuration for a Minecraft version
#[derive(Debug)]
pub struct LaunchConfig {
    /// Version to launch
    pub version_id: String,
    /// Path to Java executable
    pub java_path: String,
    /// Memory allocation in MB
    pub memory: u32,
    /// Game directory
    pub game_dir: PathBuf,
    /// Versions directory
    pub versions_dir: PathBuf,
    /// Assets directory
    pub assets_dir: PathBuf,
    /// Libraries directory
    pub libraries_dir: PathBuf,
}

impl LaunchConfig {
    /// Creates a LaunchConfig from application config and version info
    pub fn from_config(config: &Config, version_id: &str) -> Self {
        Self {
            version_id: version_id.to_string(),
            java_path: config.java_path.clone().unwrap_or_else(|| "java".to_string()),
            memory: config.memory,
            game_dir: config.game_dir.clone(),
            versions_dir: config.versions_dir.clone(),
            assets_dir: config.assets_dir.clone(),
            libraries_dir: config.game_dir.join("libraries"),
        }
    }
}

/// Result of a launch attempt
#[derive(Debug, Clone)]
pub enum LaunchResult {
    /// Game launched successfully
    Success,
    /// Missing files need to be downloaded
    NeedsDownload(Vec<String>),
    /// Launch failed with error
    Error(String),
}

/// Checks if a version is ready to launch.
/// Verifies that required files exist including libraries.
///
/// # Arguments
/// * `config` - Launch configuration
///
/// # Returns
/// * `LaunchResult` - Status of the version
pub async fn check_version_ready(config: &LaunchConfig) -> LaunchResult {
    let version_dir = config.versions_dir.join(&config.version_id);
    let jar_path = version_dir.join(format!("{}.jar", config.version_id));
    let json_path = version_dir.join(format!("{}.json", config.version_id));
    
    let mut missing_files = Vec::new();
    
    // Check if version JSON exists
    if !json_path.exists() {
        missing_files.push(format!("{}.json", config.version_id));
        return LaunchResult::NeedsDownload(missing_files);
    }
    
    // Check if client JAR exists
    if !jar_path.exists() {
        missing_files.push(format!("{}.jar", config.version_id));
    }
    
    // Read version JSON to check libraries
    if let Ok(json_content) = std::fs::read_to_string(&json_path) {
        if let Ok(version) = serde_json::from_str::<Version>(&json_content) {
            // Check each library
            for library in &version.libraries {
                if let Some(library_path) = get_library_path(library, &config.libraries_dir) {
                    if !library_path.exists() {
                        missing_files.push(format!("library: {}", library.name));
                    }
                }
            }
        }
    }
    
    if missing_files.is_empty() {
        LaunchResult::Success
    } else {
        info!("Missing {} files for version {}", missing_files.len(), config.version_id);
        LaunchResult::NeedsDownload(missing_files)
    }
}

/// Gets the local path for a library based on its Maven coordinates
fn get_library_path(library: &Library, libraries_dir: &PathBuf) -> Option<PathBuf> {
    // Check if library has explicit download info
    if let Some(downloads) = &library.downloads {
        if let Some(artifact) = &downloads.artifact {
            return Some(libraries_dir.join(&artifact.path));
        }
    }
    
    // Parse Maven coordinates (group:artifact:version)
    let parts: Vec<&str> = library.name.split(':').collect();
    if parts.len() >= 3 {
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        
        // Check if there's a classifier (e.g., natives-linux)
        let classifier = if parts.len() > 3 { parts[3] } else { "" };
        
        let filename = if classifier.is_empty() {
            format!("{}-{}.jar", artifact, version)
        } else {
            format!("{}-{}-{}.jar", artifact, version, classifier)
        };
        
        let path = libraries_dir.join(&group).join(artifact).join(version).join(filename);
        return Some(path);
    }
    
    None
}

/// Downloads required files for a version including all libraries.
///
/// # Arguments
/// * `version_info` - Version info from manifest
/// * `config` - Application configuration
/// * `progress_callback` - Callback for download progress
///
/// # Returns
/// * `Result<(), String>` - Success or error message
pub async fn download_version_files(
    version_info: &VersionInfo,
    config: &Config,
    progress_callback: impl Fn(f32),
) -> Result<(), String> {
    info!("Downloading version files for: {}", version_info.id);
    
    let version_dir = config.versions_dir.join(&version_info.id);
    let libraries_dir = config.game_dir.join("libraries");
    std::fs::create_dir_all(&version_dir)
        .map_err(|e| format!("Failed to create version directory: {}", e))?;
    std::fs::create_dir_all(&libraries_dir)
        .map_err(|e| format!("Failed to create libraries directory: {}", e))?;
    
    // Download version JSON if not exists
    let json_path = version_dir.join(format!("{}.json", version_info.id));
    let version_detail = if !json_path.exists() {
        info!("Downloading version JSON...");
        let detail = Version::fetch_version_detail(version_info).await?;
        let json_content = serde_json::to_string_pretty(&detail)
            .map_err(|e| format!("Failed to serialize version JSON: {}", e))?;
        std::fs::write(&json_path, json_content)
            .map_err(|e| format!("Failed to write version JSON: {}", e))?;
        detail
    } else {
        // Read existing JSON
        let json_content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?
    };
    
    // Download client JAR
    let jar_path = version_dir.join(format!("{}.jar", version_info.id));
    if !jar_path.exists() {
        info!("Downloading client JAR...");
        let downloader = DownloadManager::new(version_dir.clone());
        let jar_url = &version_detail.downloads.client.url;
        downloader.download_file(jar_url, &format!("{}.jar", version_info.id), &progress_callback).await?;
    }
    
    // Download libraries
    info!("Downloading {} libraries...", version_detail.libraries.len());
    let downloader = DownloadManager::new(libraries_dir.clone());
    let mut downloaded_count = 0;
    let total_libraries = version_detail.libraries.len();
    
    for (i, library) in version_detail.libraries.iter().enumerate() {
        // Check if library should be loaded based on rules
        if !should_load_library(library) {
            debug!("Skipping library {} (not applicable)", library.name);
            continue;
        }
        
        // Get library path and download URL
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let lib_path = libraries_dir.join(&artifact.path);
                
                if !lib_path.exists() {
                    // Create parent directories
                    if let Some(parent) = lib_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create library directory: {}", e))?;
                    }
                    
                    info!("Downloading library: {}", library.name);
                    downloader.download_file(&artifact.url, &artifact.path, |_| {}).await
                        .map_err(|e| {
                            error!("Failed to download library {}: {}", library.name, e);
                            format!("Failed to download library {}: {}", library.name, e)
                        })?;
                    downloaded_count += 1;
                }
            }
            
            // Download native libraries
            if let Some(classifiers) = &downloads.classifiers {
                let os_name = std::env::consts::OS;
                let arch = std::env::consts::ARCH;
                
                let classifier_key = match os_name {
                    "linux" => format!("natives-linux"),
                    "macos" => {
                        if arch == "aarch64" {
                            "natives-macos-arm64".to_string()
                        } else {
                            "natives-macos".to_string()
                        }
                    }
                    "windows" => "natives-windows".to_string(),
                    _ => continue,
                };
                
                if let Some(native) = classifiers.get(&classifier_key) {
                    let native_path = libraries_dir.join(&native.path);
                    
                    if !native_path.exists() {
                        if let Some(parent) = native_path.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| format!("Failed to create native directory: {}", e))?;
                        }
                        
                        info!("Downloading native library: {}", library.name);
                        downloader.download_file(&native.url, &native.path, |_| {}).await
                            .map_err(|e| format!("Failed to download native {}: {}", library.name, e))?;
                        downloaded_count += 1;
                    }
                }
            }
        }
        
        // Update progress
        let progress = (i + 1) as f32 / total_libraries as f32;
        progress_callback(progress);
    }
    
    info!("Downloaded {} libraries for version {}", downloaded_count, version_info.id);
    
    // Download logging config
    if let Some(logging) = &version_detail.logging {
        if let Some(client_logging) = &logging.client {
            let logging_dir = version_dir.join("logging");
            std::fs::create_dir_all(&logging_dir)
                .map_err(|e| format!("Failed to create logging directory: {}", e))?;
            
            let logging_path = logging_dir.join(&client_logging.file.id);
            if !logging_path.exists() {
                info!("Downloading logging config: {}", client_logging.file.id);
                let downloader = DownloadManager::new(logging_dir);
                downloader.download_file(&client_logging.file.url, &client_logging.file.id, |_| {}).await?;
            }
        }
    }
    
    // Download assets
    if let Some(asset_index_info) = &version_detail.asset_index {
        info!("Downloading assets...");
        download_assets(asset_index_info, &config.assets_dir).await?;
    }
    
    info!("Version files downloaded successfully");
    Ok(())
}

/// Checks if a library should be loaded based on its rules
fn should_load_library(library: &Library) -> bool {
    if let Some(rules) = &library.rules {
        let os_name = std::env::consts::OS;
        
        // Check each rule
        let mut allowed = true;
        for rule in rules {
            match rule.action.as_str() {
                "allow" => {
                    if let Some(os) = &rule.os {
                        if let Some(name) = &os.name {
                            allowed = name == os_name;
                        }
                    }
                    // If no OS specified, it's allowed for all
                }
                "disallow" => {
                    if let Some(os) = &rule.os {
                        if let Some(name) = &os.name {
                            if name == os_name {
                                allowed = false;
                            }
                        }
                    } else {
                        allowed = false;
                    }
                }
                _ => {}
            }
        }
        allowed
    } else {
        // No rules means always allowed
        true
    }
}

/// Launches the Minecraft game.
///
/// # Arguments
/// * `config` - Launch configuration
/// * `session` - Authenticated account session
///
/// # Returns
/// * `Result<(), String>` - Success or error message
pub fn launch_game(config: &LaunchConfig, session: &AccountSession) -> Result<(), String> {
    info!("Launching Minecraft {}...", config.version_id);
    
    let version_dir = config.versions_dir.join(&config.version_id);
    let json_path = version_dir.join(format!("{}.json", config.version_id));
    let jar_path = version_dir.join(format!("{}.jar", config.version_id));
    
    // Read version JSON
    let json_content = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read version JSON: {}", e))?;
    
    let version: Version = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse version JSON: {}", e))?;
    
    // Create natives directory
    let natives_dir = version_dir.join("natives");
    std::fs::create_dir_all(&natives_dir)
        .map_err(|e| format!("Failed to create natives directory: {}", e))?;
    
    // Extract native libraries
    extract_natives(&version, &config.libraries_dir, &natives_dir)?;
    
    // Build classpath
    let classpath = build_classpath(&version, &config.libraries_dir, &jar_path);
    
    // Get asset index ID
    let asset_index_id = version.asset_index.as_ref()
        .map(|ai| ai.id.clone())
        .unwrap_or_else(|| config.version_id.clone());
    
    // Build JVM arguments
    let mut jvm_args = build_jvm_args(config, &classpath, &natives_dir);
    
    // Add logging config if available
    let logging_path = version_dir.join("logging").join(
        version.logging.as_ref()
            .and_then(|l| l.client.as_ref())
            .map(|c| c.file.id.clone())
            .unwrap_or_default()
    );
    
    if logging_path.exists() {
        if let Some(logging) = &version.logging {
            if let Some(client) = &logging.client {
                let log_arg = client.argument.replace("${path}", &logging_path.to_string_lossy());
                jvm_args.push(log_arg);
            }
        }
    }
    
    // Build game arguments
    let game_args = build_game_args(config, session, &version, &asset_index_id);
    
    // Build full command
    let mut cmd = Command::new(&config.java_path);
    cmd.args(&jvm_args);
    cmd.arg(&version.main_class);
    cmd.args(&game_args);
    cmd.current_dir(&config.game_dir);
    
    info!("Java path: {}", config.java_path);
    info!("Main class: {}", version.main_class);
    debug!("Command: {:?}", cmd);
    
    // Launch the game
    match cmd.spawn() {
        Ok(child) => {
            info!("Game launched successfully (PID: {})", child.id());
            Ok(())
        }
        Err(e) => {
            error!("Failed to launch game: {}", e);
            Err(format!("Failed to launch game: {}", e))
        }
    }
}

/// Extracts native libraries to the natives directory
fn extract_natives(version: &Version, libraries_dir: &PathBuf, natives_dir: &PathBuf) -> Result<(), String> {
    let os_name = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    for library in &version.libraries {
        if let Some(downloads) = &library.downloads {
            let classifier_key = match os_name {
                "linux" => "natives-linux".to_string(),
                "macos" => {
                    if arch == "aarch64" {
                        "natives-macos-arm64".to_string()
                    } else {
                        "natives-macos".to_string()
                    }
                }
                "windows" => "natives-windows".to_string(),
                _ => continue,
            };
            
            if let Some(classifiers) = &downloads.classifiers {
                if let Some(native) = classifiers.get(&classifier_key) {
                    let native_path = libraries_dir.join(&native.path);
                    
                    if native_path.exists() {
                        info!("Extracting native: {}", library.name);
                        extract_jar(&native_path, natives_dir)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Extracts a JAR file to a directory
fn extract_jar(jar_path: &PathBuf, output_dir: &PathBuf) -> Result<(), String> {
    let file = std::fs::File::open(jar_path)
        .map_err(|e| format!("Failed to open JAR: {}", e))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read JAR: {}", e))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read entry: {}", e))?;
        
        let outpath = output_dir.join(file.mangled_name());
        
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    
    Ok(())
}

/// Builds the classpath for the game.
fn build_classpath(version: &Version, libraries_dir: &PathBuf, jar_path: &PathBuf) -> String {
    let mut entries = vec![jar_path.to_string_lossy().to_string()];
    
    for library in &version.libraries {
        if !should_load_library(library) {
            continue;
        }
        
        if let Some(lib_path) = get_library_path(library, libraries_dir) {
            if lib_path.exists() {
                entries.push(lib_path.to_string_lossy().to_string());
            }
        }
    }
    
    entries.join(":")
}

/// Builds JVM arguments.
fn build_jvm_args(config: &LaunchConfig, classpath: &str, natives_dir: &PathBuf) -> Vec<String> {
    vec![
        format!("-Xmx{}M", config.memory),
        format!("-Xms{}M", config.memory / 2),
        format!("-Djava.library.path={}", natives_dir.to_string_lossy()),
        format!("-Dminecraft.launcher.brand=mcl-rs"),
        format!("-Dminecraft.launcher.version={}", env!("CARGO_PKG_VERSION")),
        "-cp".to_string(),
        classpath.to_string(),
    ]
}

/// Builds game arguments.
fn build_game_args(config: &LaunchConfig, session: &AccountSession, version: &Version, asset_index_id: &str) -> Vec<String> {
    let mut args = Vec::new();
    
    // Check if version uses new or old argument format
    if let Some(arguments) = &version.arguments {
        // New format (1.13+)
        for arg in &arguments.game {
            if let Some(s) = arg.as_str() {
                args.push(replace_tokens(s, config, session, asset_index_id));
            }
        }
    } else if let Some(minecraft_arguments) = &version.minecraft_arguments {
        // Old format (pre-1.13)
        for arg in minecraft_arguments.split_whitespace() {
            args.push(replace_tokens(arg, config, session, asset_index_id));
        }
    }
    
    args
}

/// Replaces tokens in argument strings.
fn replace_tokens(arg: &str, config: &LaunchConfig, session: &AccountSession, asset_index_id: &str) -> String {
    arg.replace("${auth_player_name}", &session.minecraft_profile.name)
        .replace("${auth_uuid}", &session.minecraft_profile.id)
        .replace("${auth_access_token}", &session.access_token)
        .replace("${auth_session}", &session.access_token)
        .replace("${version_name}", &config.version_id)
        .replace("${game_directory}", &config.game_dir.to_string_lossy())
        .replace("${game_assets}", &config.assets_dir.to_string_lossy())
        .replace("${assets_root}", &config.assets_dir.to_string_lossy())
        .replace("${assets_index_name}", asset_index_id)
        .replace("${user_type}", "msa")
        .replace("${user_properties}", "{}")
        .replace("${launcher_name}", "mcl-rs")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
}

/// Asset index file structure
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssetIndexFile {
    pub objects: std::collections::HashMap<String, AssetObject>,
}

/// Individual asset object
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

/// Downloads all assets for a version concurrently.
///
/// # Arguments
/// * `asset_index_info` - Asset index information from version JSON
/// * `assets_dir` - Path to the assets directory
///
/// # Returns
/// * `Result<(), String>` - Success or error message
async fn download_assets(asset_index_info: &crate::core::version::AssetIndex, assets_dir: &PathBuf) -> Result<(), String> {
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");
    let virtual_dir = assets_dir.join("virtual").join("legacy");
    
    std::fs::create_dir_all(&indexes_dir)
        .map_err(|e| format!("Failed to create indexes directory: {}", e))?;
    std::fs::create_dir_all(&objects_dir)
        .map_err(|e| format!("Failed to create objects directory: {}", e))?;
    // Create virtual/legacy directory for older versions
    std::fs::create_dir_all(&virtual_dir)
        .map_err(|e| format!("Failed to create virtual directory: {}", e))?;
    
    // Download asset index
    let index_path = indexes_dir.join(format!("{}.json", asset_index_info.id));
    let asset_index: AssetIndexFile = if !index_path.exists() {
        info!("Downloading asset index: {}", asset_index_info.id);
        let client = create_http_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let response = client.get(&asset_index_info.url)
            .send()
            .await
            .map_err(|e| format!("Failed to download asset index: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to download asset index: HTTP {}", response.status()));
        }
        
        let content = response.text().await
            .map_err(|e| format!("Failed to read asset index: {}", e))?;
        
        std::fs::write(&index_path, &content)
            .map_err(|e| format!("Failed to save asset index: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse asset index: {}", e))?
    } else {
        let content = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Failed to read asset index: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse asset index: {}", e))?
    };
    
    // Filter assets that need downloading
    let assets_to_download: Vec<(String, String)> = asset_index.objects.iter()
        .filter_map(|(name, asset)| {
            let hash_prefix = &asset.hash[..2];
            let object_path = objects_dir.join(hash_prefix).join(&asset.hash);
            if object_path.exists() {
                None
            } else {
                Some((name.clone(), asset.hash.clone()))
            }
        })
        .collect();
    
    let total_assets = asset_index.objects.len();
    let skipped_count = total_assets - assets_to_download.len();
    
    if assets_to_download.is_empty() {
        info!("All {} assets already downloaded", total_assets);
        return Ok(());
    }
    
    info!("Downloading {} assets ({} already exists)...", assets_to_download.len(), skipped_count);
    
    // Download assets concurrently with system proxy support
    let client = create_http_client()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let downloaded_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failed_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    
    // Use semaphore to limit concurrent downloads
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(32));
    
    let mut handles = Vec::new();
    
    for (name, hash) in assets_to_download {
        let client = client.clone();
        let objects_dir = objects_dir.clone();
        let semaphore = semaphore.clone();
        let downloaded_count = downloaded_count.clone();
        let failed_count = failed_count.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let hash_prefix = &hash[..2];
            let object_dir = objects_dir.join(hash_prefix);
            let object_path = object_dir.join(&hash);
            let url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
            
            // Create directory
            if let Err(e) = std::fs::create_dir_all(&object_dir) {
                error!("Failed to create directory for {}: {}", name, e);
                failed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return;
            }
            
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.bytes().await {
                            Ok(bytes) => {
                                if let Err(e) = std::fs::write(&object_path, &bytes) {
                                    error!("Failed to save asset {}: {}", name, e);
                                    failed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                } else {
                                    downloaded_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                            Err(e) => {
                                error!("Failed to read asset {}: {}", name, e);
                                failed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    } else {
                        error!("Failed to download asset {}: HTTP {}", name, response.status());
                        failed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                Err(e) => {
                    error!("Failed to download asset {}: {}", name, e);
                    failed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all downloads to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let downloaded = downloaded_count.load(std::sync::atomic::Ordering::Relaxed);
    let failed = failed_count.load(std::sync::atomic::Ordering::Relaxed);
    
    info!("Assets downloaded: {} new, {} skipped, {} failed", downloaded, skipped_count, failed);
    
    if failed > 0 {
        warn!("{} assets failed to download", failed);
    }
    
    Ok(())
}

/// Creates an HTTP client with system proxy support.
/// Reads proxy configuration from environment variables:
/// - HTTP_PROXY / http_proxy
/// - HTTPS_PROXY / https_proxy
/// - ALL_PROXY / all_proxy
/// - NO_PROXY / no_proxy
fn create_http_client() -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .pool_max_idle_per_host(32)
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
    } else {
        // Try to detect system proxy from desktop environment
        if let Some(proxy_url) = detect_system_proxy() {
            info!("Detected system proxy: {}", proxy_url);
            let proxy = reqwest::Proxy::all(&proxy_url)
                .map_err(|e| format!("Invalid proxy URL: {}", e))?;
            builder = builder.proxy(proxy);
        } else {
            debug!("No proxy configured");
        }
    }
    
    builder.build().map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Detects system proxy from desktop environment (Linux/macOS/Windows)
fn detect_system_proxy() -> Option<String> {
    // On Linux, check for common desktop environment proxy settings
    #[cfg(target_os = "linux")]
    {
        // Check GNOME proxy
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.system.proxy", "mode"])
            .output()
        {
            let mode = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if mode == "'manual'" {
                // Get HTTP proxy host and port
                let host = std::process::Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.http", "host"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().trim_matches('\'').to_string());
                
                let port = std::process::Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.http", "port"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string());
                
                if let (Some(host), Some(port)) = (host, port) {
                    if !host.is_empty() && !port.is_empty() {
                        return Some(format!("http://{}:{}", host, port));
                    }
                }
            }
        }
        
        // Check environment variable for KDE
        if let Ok(proxy) = std::env::var("KDE_PROXY") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }
    }
    
    // On macOS, check system preferences
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("networksetup")
            .args(["-getwebproxy", "Wi-Fi"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut host = None;
            let mut port = None;
            let mut enabled = false;
            
            for line in output_str.lines() {
                if line.starts_with("Enabled: Yes") {
                    enabled = true;
                } else if line.starts_with("Server:") {
                    host = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("Port:") {
                    port = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
            }
            
            if enabled {
                if let (Some(host), Some(port)) = (host, port) {
                    if !host.is_empty() && !port.is_empty() {
                        return Some(format!("http://{}:{}", host, port));
                    }
                }
            }
        }
    }
    
    // On Windows, check registry or environment
    #[cfg(target_os = "windows")]
    {
        // Windows proxy is usually set via environment variables or IE settings
        // The environment variables are already checked above
    }
    
    None
}