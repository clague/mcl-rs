// Download Module
// Handles downloading of Minecraft game files, libraries, and assets

use std::path::PathBuf;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use log::{info, error, debug};

/// Manages file downloads with progress tracking
pub struct DownloadManager {
    /// HTTP client for making requests
    client: Client,
    /// Base directory for downloads
    download_dir: PathBuf,
}

impl DownloadManager {
    /// Creates a new DownloadManager with system proxy support
    ///
    /// # Arguments
    /// * `download_dir` - Base directory for storing downloaded files
    pub fn new(download_dir: PathBuf) -> Self {
        let client = Self::create_client_with_proxy();
        info!("Download manager initialized, directory: {:?}", download_dir);
        Self {
            client,
            download_dir,
        }
    }

    /// Creates an HTTP client with system proxy support
    fn create_client_with_proxy() -> Client {
        let mut builder = Client::builder()
            .pool_max_idle_per_host(16)
            .timeout(std::time::Duration::from_secs(60));
        
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
            info!("Download manager using proxy: {}", proxy_url);
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        } else {
            // Try to detect system proxy
            if let Some(proxy_url) = Self::detect_system_proxy() {
                info!("Download manager detected system proxy: {}", proxy_url);
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }
        
        builder.build().expect("Failed to create HTTP client")
    }

    /// Detects system proxy from desktop environment
    fn detect_system_proxy() -> Option<String> {
        // On Linux, check GNOME proxy settings
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
            {
                let mode = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if mode == "'manual'" {
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
        }
        
        None
    }

    /// Downloads a file from a URL with progress tracking
    ///
    /// # Arguments
    /// * `url` - URL to download from
    /// * `filename` - Local filename to save as
    /// * `progress_callback` - Callback for progress updates (0.0 to 1.0)
    ///
    /// # Returns
    /// * `Ok(())` - Download completed successfully
    /// * `Err(String)` - Error message if download fails
    pub async fn download_file(
        &self,
        url: &str,
        filename: &str,
        progress_callback: impl Fn(f32),
    ) -> Result<(), String> {
        info!("Downloading file: {} -> {}", url, filename);
        
        let response = self.client.get(url).send().await.map_err(|e| {
            error!("Failed to start download for {}: {}", filename, e);
            e.to_string()
        })?;
        
        let total_size = response.content_length().unwrap_or(0);
        debug!("File size: {} bytes", total_size);
        
        let filepath = self.download_dir.join(filename);
        let mut file = File::create(&filepath).await.map_err(|e| {
            error!("Failed to create file {:?}: {}", filepath, e);
            e.to_string()
        })?;
        
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut last_progress = 0.0;
        
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                error!("Download stream error for {}: {}", filename, e);
                e.to_string()
            })?;
            
            file.write_all(&chunk).await.map_err(|e| {
                error!("Failed to write to file {:?}: {}", filepath, e);
                e.to_string()
            })?;
            
            downloaded += chunk.len() as u64;
            
            if total_size > 0 {
                let progress = downloaded as f32 / total_size as f32;
                // Log progress at 25% intervals
                if progress - last_progress >= 0.25 {
                    info!("Download progress for {}: {:.0}%", filename, progress * 100.0);
                    last_progress = progress;
                }
                progress_callback(progress);
            }
        }
        
        info!("Download completed: {} ({} bytes)", filename, downloaded);
        Ok(())
    }

    /// Downloads and parses a JSON file from a URL
    ///
    /// # Arguments
    /// * `url` - URL to download JSON from
    ///
    /// # Returns
    /// * `Ok(T)` - Parsed JSON data
    /// * `Err(String)` - Error message if download or parsing fails
    pub async fn download_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, String> {
        debug!("Downloading JSON from: {}", url);
        
        let response = self.client.get(url).send().await.map_err(|e| {
            error!("Failed to download JSON from {}: {}", url, e);
            e.to_string()
        })?;
        
        let json = response.json::<T>().await.map_err(|e| {
            error!("Failed to parse JSON from {}: {}", url, e);
            e.to_string()
        })?;
        
        debug!("JSON downloaded and parsed successfully");
        Ok(json)
    }
}