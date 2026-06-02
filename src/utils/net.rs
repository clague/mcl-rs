// Network Utilities
// HTTP client helpers for downloading game files and API calls

use reqwest::Client;
use std::time::Duration;

/// Creates a configured HTTP client with reasonable defaults.
///
/// # Returns
/// * `Client` - Configured reqwest client
pub fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Downloads content from a URL with progress tracking.
///
/// # Arguments
/// * `client` - HTTP client to use
/// * `url` - URL to download from
/// * `progress_callback` - Callback for progress updates (0.0 to 1.0)
///
/// # Returns
/// * `Ok(Vec<u8>)` - Downloaded content as bytes
/// * `Err(String)` - Error message if download fails
pub async fn download_with_progress(
    client: &Client,
    url: &str,
    progress_callback: impl Fn(f32),
) -> Result<Vec<u8>, String> {
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        bytes.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            progress_callback(downloaded as f32 / total_size as f32);
        }
    }
    
    Ok(bytes)
}