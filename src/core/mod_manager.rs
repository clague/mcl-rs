use std::path::PathBuf;

use log::{debug, error, info};

use crate::config::config::{Config, InstalledModInfo};
use crate::core::modrinth::ModrinthClient;

pub struct ModManager {
    modrinth: ModrinthClient,
}

impl ModManager {
    pub fn new() -> Self {
        info!("ModManager created");
        Self {
            modrinth: ModrinthClient::new(),
        }
    }

    /// Searches for mods on Modrinth, returning up to 20 results.
    pub async fn search_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<crate::core::ModSearchResult, String> {
        info!(
            "Searching mods: query={:?}, version={:?}, loader={:?}",
            query, game_version, loader
        );

        let result = self
            .modrinth
            .search(query, game_version, loader, 0, 20)
            .await?;

        info!(
            "Search returned {} hits (total: {})",
            result.hits.len(),
            result.total_hits
        );
        Ok(result)
    }

    /// Installs the latest compatible version of a mod to `mods_dir`.
    ///
    /// `version_uuid` is the config key (typically the Minecraft version identifier).
    pub async fn install_mod(
        &self,
        version_uuid: &str,
        project_id: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<InstalledModInfo, String> {
        info!(
            "Installing mod project={} for {} / {} (version_uuid={})",
            project_id, game_version, loader, version_uuid
        );

        let mods_dir = resolve_mods_dir()?;

        let versions = self
            .modrinth
            .get_versions(project_id, Some(game_version), Some(loader))
            .await?;

        if versions.is_empty() {
            let msg = format!(
                "No compatible version found for project {} ({} / {})",
                project_id, game_version, loader
            );
            error!("{}", msg);
            return Err(msg);
        }

        let version = pick_best_version(&versions);
        debug!(
            "Selected version: {} ({})",
            version.version_number, version.id
        );

        let _path_str = self.modrinth.download_mod(version, &mods_dir).await?;

        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| {
                let msg = format!(
                    "No file entry for version {} of project {}",
                    version.version_number, project_id
                );
                error!("{}", msg);
                msg
            })?;

        let installed = InstalledModInfo {
            project_id: project_id.to_string(),
            slug: String::new(),
            title: String::new(),
            version_id: version.id.clone(),
            version_number: version.version_number.clone(),
            filename: file.filename.clone(),
            enabled: true,
            loaders: version
                .loaders
                .iter()
                .map(|l| l.to_string().to_lowercase())
                .collect(),
        };

        let installed = enrich_installed_info(project_id, installed).await;

        let mut config = Config::load();
        config.add_mod(version_uuid, installed.clone());

        info!(
            "Mod installed: {} v{} ({})",
            installed.title, installed.version_number, installed.filename
        );
        Ok(installed)
    }

    /// Deletes the mod file from disk and removes it from config.
    pub async fn uninstall_mod(
        &self,
        version_uuid: &str,
        project_id: &str,
    ) -> Result<(), String> {
        info!(
            "Uninstalling mod project={} from version {}",
            project_id, version_uuid
        );

        let mut config = Config::load();
        let mods = config.get_mods(version_uuid);
        let mod_info = mods
            .iter()
            .find(|m| m.project_id == project_id)
            .ok_or_else(|| {
                let msg = format!(
                    "Mod project={} not found in version {}",
                    project_id, version_uuid
                );
                error!("{}", msg);
                msg
            })?;

        let mods_dir = resolve_mods_dir()?;
        let file_path = mods_dir.join(&mod_info.filename);
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await.map_err(|e| {
                let msg = format!("Failed to delete mod file {:?}: {}", file_path, e);
                error!("{}", msg);
                msg
            })?;
            debug!("Deleted mod file: {:?}", file_path);
        } else {
            debug!("Mod file already absent: {:?}", file_path);
        }

        config.remove_mod(version_uuid, project_id);

        info!("Mod uninstalled: project={}", project_id);
        Ok(())
    }

    pub async fn enable_mod(
        &self,
        version_uuid: &str,
        project_id: &str,
    ) -> Result<(), String> {
        info!(
            "Enabling mod project={} in version {}",
            project_id, version_uuid
        );
        set_mod_enabled(version_uuid, project_id, true)
    }

    pub async fn disable_mod(
        &self,
        version_uuid: &str,
        project_id: &str,
    ) -> Result<(), String> {
        info!(
            "Disabling mod project={} in version {}",
            project_id, version_uuid
        );
        set_mod_enabled(version_uuid, project_id, false)
    }

    pub fn get_installed_mods(&self, version_uuid: &str) -> Vec<InstalledModInfo> {
        let config = Config::load();
        let mods = config.get_mods(version_uuid);
        debug!(
            "get_installed_mods({}): found {} mods",
            version_uuid,
            mods.len()
        );
        mods
    }

    /// Uninstalls the old version, fetches the latest compatible version,
    /// and re-installs it. Returns the new `InstalledModInfo`.
    ///
    /// If the update fails, attempts to re-install the original version.
    pub async fn update_mod(
        &self,
        version_uuid: &str,
        project_id: &str,
    ) -> Result<InstalledModInfo, String> {
        info!(
            "Updating mod project={} in version {}",
            project_id, version_uuid
        );

        let config = Config::load();
        let mods = config.get_mods(version_uuid);
        let current = mods
            .iter()
            .find(|m| m.project_id == project_id)
            .ok_or_else(|| {
                let msg = format!(
                    "Mod project={} not found in version {}",
                    project_id, version_uuid
                );
                error!("{}", msg);
                msg
            })?;

        let current_loader = current.loaders.first().cloned().unwrap_or_default();
        let current_filename = current.filename.clone();
        let current_version_id = current.version_id.clone();

        drop(config);
        self.uninstall_mod(version_uuid, project_id).await?;

        let versions = self
            .modrinth
            .get_versions(project_id, None, Some(&current_loader))
            .await?;

        if versions.is_empty() {
            let msg = format!(
                "No compatible version found for project {} with loader {}",
                project_id, current_loader
            );
            error!("{}", msg);
            let _ = self
                .restore_version(version_uuid, project_id, &current_version_id)
                .await;
            return Err(msg);
        }

        let new_version = pick_best_version(&versions);

        if new_version.id == current_version_id {
            info!(
                "Mod {} is already up-to-date (v{})",
                project_id, new_version.version_number
            );
            let _ = self
                .restore_version(version_uuid, project_id, &current_version_id)
                .await;
            let config = Config::load();
            let mods = config.get_mods(version_uuid);
            return mods
                .into_iter()
                .find(|m| m.project_id == project_id)
                .ok_or_else(|| "Mod disappeared during update check".to_string());
        }

        let mods_dir = resolve_mods_dir()?;
        let old_path = mods_dir.join(&current_filename);
        if old_path.exists() {
            let _ = tokio::fs::remove_file(&old_path).await;
        }

        let _path_str = self
            .modrinth
            .download_mod(new_version, &mods_dir)
            .await?;

        let file = new_version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| new_version.files.first())
            .ok_or_else(|| {
                let msg = format!(
                    "No file entry for version {} of project {}",
                    new_version.version_number, project_id
                );
                error!("{}", msg);
                msg
            })?;

        let installed = InstalledModInfo {
            project_id: project_id.to_string(),
            slug: current.slug.clone(),
            title: current.title.clone(),
            version_id: new_version.id.clone(),
            version_number: new_version.version_number.clone(),
            filename: file.filename.clone(),
            enabled: true,
            loaders: new_version
                .loaders
                .iter()
                .map(|l| l.to_string().to_lowercase())
                .collect(),
        };

        let mut config = Config::load();
        config.add_mod(version_uuid, installed.clone());

        info!(
            "Mod updated: {} v{} -> v{}",
            current.title, current.version_number, installed.version_number
        );
        Ok(installed)
    }

    /// Re-downloads and registers a specific version (used for update recovery).
    async fn restore_version(
        &self,
        version_uuid: &str,
        project_id: &str,
        version_id: &str,
    ) -> Result<(), String> {
        let mods_dir = resolve_mods_dir()?;
        let versions = self
            .modrinth
            .get_versions(project_id, None, None)
            .await?;

        let version = versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| {
                format!(
                    "Version {} not found for project {}",
                    version_id, project_id
                )
            })?;

        let _ = self.modrinth.download_mod(version, &mods_dir).await;

        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or("No file entry")?;

        let installed = InstalledModInfo {
            project_id: project_id.to_string(),
            slug: String::new(),
            title: String::new(),
            version_id: version.id.clone(),
            version_number: version.version_number.clone(),
            filename: file.filename.clone(),
            enabled: true,
            loaders: version
                .loaders
                .iter()
                .map(|l| l.to_string().to_lowercase())
                .collect(),
        };

        let installed = enrich_installed_info(project_id, installed).await;

        let mut config = Config::load();
        config.add_mod(version_uuid, installed);

        Ok(())
    }
}

fn resolve_mods_dir() -> Result<PathBuf, String> {
    let config = Config::load();
    config.mods_dir.ok_or_else(|| {
        let msg = "mods_dir is not configured".to_string();
        error!("{}", msg);
        msg
    })
}

/// Sets the enabled flag on a mod in the given version.
fn set_mod_enabled(
    version_uuid: &str,
    project_id: &str,
    enabled: bool,
) -> Result<(), String> {
    let mut config = Config::load();
    let mods = config.get_mods(version_uuid);
    let current = mods
        .iter()
        .find(|m| m.project_id == project_id)
        .ok_or_else(|| {
            let msg = format!(
                "Mod project={} not found in version {}",
                project_id, version_uuid
            );
            error!("{}", msg);
            msg
        })?;

    if current.enabled == enabled {
        debug!(
            "Mod project={} already {} in version {}",
            project_id,
            if enabled { "enabled" } else { "disabled" },
            version_uuid
        );
        return Ok(());
    }

    config.toggle_mod(version_uuid, project_id);

    info!(
        "Mod project={} {} in version {}",
        project_id,
        if enabled { "enabled" } else { "disabled" },
        version_uuid
    );
    Ok(())
}

/// Picks the best version preferring release > beta > alpha.
fn pick_best_version<'a>(versions: &'a [crate::core::ModVersion]) -> &'a crate::core::ModVersion {
    versions
        .iter()
        .min_by_key(|v| match v.version_type.as_str() {
            "release" => 0,
            "beta" => 1,
            "alpha" => 2,
            _ => 3,
        })
        .expect("versions list must not be empty")
}

/// Fetches project title/slug from the API and populates the `InstalledModInfo`.
async fn enrich_installed_info(
    project_id: &str,
    mut installed: InstalledModInfo,
) -> InstalledModInfo {
    let client = ModrinthClient::new();
    match client.get_project(project_id).await {
        Ok(project) => {
            installed.slug = project.slug;
            installed.title = project.title;
            debug!("Enriched mod info: {} ({})", installed.title, installed.slug);
        }
        Err(e) => {
            debug!(
                "Could not enrich mod info for {}: {} (using defaults)",
                project_id, e
            );
            installed.title = project_id.to_string();
            installed.slug = project_id.to_string();
        }
    }
    installed
}
