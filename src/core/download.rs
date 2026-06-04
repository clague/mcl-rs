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
    pub fn new(download_dir: PathBuf) -> Self {
        let client = crate::utils::net::shared_client();
        info!("Download manager initialized, directory: {:?}", download_dir);
        Self {
            client,
            download_dir,
        }
    }

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