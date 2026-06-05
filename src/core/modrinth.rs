// Modrinth API Client
// Provides search, project lookup, version listing, and download capabilities
// for mods hosted on https://modrinth.com

use std::path::PathBuf;

use log::{debug, error, info};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::core::{ModSearchHit, ModSearchResult, ModVersion};

/// HTTP client for the Modrinth v2 API.
pub struct ModrinthClient {
    client: Client,
    base_url: String,
    user_agent: String,
}

impl ModrinthClient {
    /// Creates a new `ModrinthClient` using the global shared HTTP client.
    ///
    /// The shared client must already be initialised via
    /// [`crate::utils::net::init_client`].
    pub fn new() -> Self {
        let client = crate::utils::net::shared_client();
        Self {
            client,
            base_url: "https://api.modrinth.com/v2".to_string(),
            user_agent: "mcl-rs/0.1.0 (github.com/user/mcl-rs)".to_string(),
        }
    }

    /// Searches for projects on Modrinth.
    ///
    /// # Arguments
    /// * `query` - Free-text search string.
    /// * `game_version` - Optional Minecraft version filter (e.g. `"1.21"`).
    /// * `loader` - Optional mod loader filter (e.g. `"fabric"`).
    /// * `offset` - Pagination offset (number of results to skip).
    /// * `limit` - Maximum number of results to return (capped at 100 by the API).
    pub async fn search(
        &self,
        query: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<ModSearchResult, String> {
        let url = build_search_url(query, game_version, loader, offset, limit);
        debug!("Modrinth search: {}", url);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| {
                error!("Modrinth search request failed: {}", e);
                e.to_string()
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let msg = format!("Modrinth search returned {}: {}", status, body);
            error!("{}", msg);
            return Err(msg);
        }

        let result: ModSearchResult = response.json().await.map_err(|e| {
            error!("Failed to parse Modrinth search response: {}", e);
            e.to_string()
        })?;

        debug!(
            "Modrinth search returned {} hits (total: {})",
            result.hits.len(),
            result.total_hits
        );
        Ok(result)
    }

    /// Fetches full project details by Modrinth project ID or slug.
    ///
    /// The returned [`ModSearchHit`] contains the same fields as a search
    /// hit but with up-to-date metadata.
    pub async fn get_project(&self, project_id: &str) -> Result<ModSearchHit, String> {
        let url = format!("{}/project/{}", self.base_url, project_id);
        debug!("Modrinth get_project: {}", url);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| {
                error!("Modrinth get_project request failed: {}", e);
                e.to_string()
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let msg = format!("Modrinth get_project returned {}: {}", status, body);
            error!("{}", msg);
            return Err(msg);
        }

        let hit: ModSearchHit = response.json().await.map_err(|e| {
            error!("Failed to parse Modrinth project response: {}", e);
            e.to_string()
        })?;

        debug!("Modrinth project fetched: {}", hit.title);
        Ok(hit)
    }

    /// Lists versions of a project, optionally filtered by game version and/or
    /// loader.
    ///
    /// # Arguments
    /// * `project_id` - Modrinth project ID or slug.
    /// * `game_version` - Optional Minecraft version filter.
    /// * `loader` - Optional mod loader filter.
    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ModVersion>, String> {
        let mut url = format!("{}/project/{}/version", self.base_url, project_id);

        let mut params: Vec<String> = Vec::new();

        if let Some(lv) = loader {
            // Modrinth expects JSON-encoded arrays: loaders=["fabric"]
            params.push(format!("loaders=[\"{}\"]", lv));
        }
        if let Some(gv) = game_version {
            params.push(format!("game_versions=[\"{}\"]", gv));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        debug!("Modrinth get_versions: {}", url);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| {
                error!("Modrinth get_versions request failed: {}", e);
                e.to_string()
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let msg = format!("Modrinth get_versions returned {}: {}", status, body);
            error!("{}", msg);
            return Err(msg);
        }

        let versions: Vec<ModVersion> = response.json().await.map_err(|e| {
            error!("Failed to parse Modrinth versions response: {}", e);
            e.to_string()
        })?;

        debug!("Modrinth get_versions returned {} versions", versions.len());
        Ok(versions)
    }

    /// Downloads the primary file of a [`ModVersion`] into `download_dir`.
    ///
    /// Returns the local file path on success.
    ///
    /// # Arguments
    /// * `version` - Version object containing at least one file entry.
    /// * `download_dir` - Destination directory (created if it does not exist).
    pub async fn download_mod(
        &self,
        version: &ModVersion,
        download_dir: &PathBuf,
    ) -> Result<String, String> {
        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| {
                let msg = format!(
                    "No files found for version {} of project {}",
                    version.version_number, version.project_id
                );
                error!("{}", msg);
                msg
            })?;

        info!(
            "Downloading mod file: {} ({} bytes)",
            file.filename, file.size
        );

        tokio::fs::create_dir_all(download_dir)
            .await
            .map_err(|e| {
                let msg = format!("Failed to create download directory {:?}: {}", download_dir, e);
                error!("{}", msg);
                msg
            })?;

        let response = self
            .client
            .get(&file.url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| {
                error!("Download request failed for {}: {}", file.filename, e);
                e.to_string()
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let msg = format!(
                "Download of {} returned {}: {}",
                file.filename, status, body
            );
            error!("{}", msg);
            return Err(msg);
        }

        let filepath = download_dir.join(&file.filename);
        let mut out = File::create(&filepath).await.map_err(|e| {
            error!("Failed to create file {:?}: {}", filepath, e);
            e.to_string()
        })?;

        let mut stream = response.bytes_stream();
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                error!("Download stream error for {}: {}", file.filename, e);
                e.to_string()
            })?;
            out.write_all(&chunk).await.map_err(|e| {
                error!("Failed to write to {:?}: {}", filepath, e);
                e.to_string()
            })?;
        }

        let path_str = filepath.to_string_lossy().to_string();
        info!("Mod downloaded: {}", path_str);
        Ok(path_str)
    }
}

/// Constructs the Modrinth search URL with facet filters.
///
/// Facets are encoded as a JSON array-of-arrays, e.g.:
/// ```text
/// [["categories:fabric"],["versions:1.21"],["project_type:mod"]]
/// ```
///
/// The `project_type:mod` facet is always appended.
pub fn build_search_url(
    query: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    offset: usize,
    limit: usize,
) -> String {
    let mut facets: Vec<String> = Vec::new();
    facets.push(r#"["project_type:mod"]"#.to_string());

    if let Some(lv) = loader {
        facets.push(format!(r#"["categories:{}"]"#, lv));
    }
    if let Some(gv) = game_version {
        facets.push(format!(r#"["versions:{}"]"#, gv));
    }

    let facets_json = format!("[{}]", facets.join(","));
    let encoded_facets = urlencoding::encode(&facets_json);

    format!(
        "https://api.modrinth.com/v2/search?facets={}&offset={}&limit={}&query={}",
        encoded_facets,
        offset,
        limit,
        urlencoding::encode(query),
    )
}
