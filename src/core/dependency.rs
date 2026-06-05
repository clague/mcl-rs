// Dependency Resolver
// Resolves mod dependencies recursively when installing mods from Modrinth.

use std::collections::HashSet;

use log::{debug, error, info, warn};

use crate::core::modrinth::ModrinthClient;
use crate::core::ModVersion;

/// Resolves mod dependencies for installation.
///
/// Given a mod version, this resolver traverses its dependency tree and returns
/// all required mods that must be installed alongside it.
pub struct DependencyResolver {
    modrinth: ModrinthClient,
}

impl DependencyResolver {
    /// Creates a new `DependencyResolver` backed by the given Modrinth client.
    pub fn new(client: ModrinthClient) -> Self {
        Self { modrinth: client }
    }

    /// Recursively resolves all required dependencies for a mod version.
    ///
    /// Traverses the dependency tree depth-first, filtering to only `"required"`
    /// dependencies. Returns a flat list of all [`ModVersion`]s that need to be
    /// installed (excluding the root version itself).
    ///
    /// Tracks visited project IDs to prevent infinite recursion from circular
    /// dependencies.
    pub async fn resolve_dependencies(
        &self,
        version: &ModVersion,
        game_version: &str,
        loader: &str,
    ) -> Result<Vec<ModVersion>, String> {
        info!(
            "Resolving dependencies for {} v{} ({} / {})",
            version.project_id, version.version_number, game_version, loader
        );

        let mut resolved: Vec<ModVersion> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();

        self.resolve_recursive(
            version,
            game_version,
            loader,
            &mut resolved,
            &mut visited,
        )
        .await?;

        info!(
            "Resolved {} dependencies for project {}",
            resolved.len(),
            version.project_id
        );
        Ok(resolved)
    }

    /// Checks whether a project has any version compatible with the given
    /// game version and loader combination.
    pub async fn check_compatibility(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<bool, String> {
        debug!(
            "Checking compatibility for project {} ({}/{})",
            project_id, game_version, loader
        );

        let versions = self
            .modrinth
            .get_versions(project_id, Some(game_version), Some(loader))
            .await?;

        let compatible = !versions.is_empty();
        debug!(
            "Project {} compatibility: {} ({} matching versions)",
            project_id, compatible, versions.len()
        );
        Ok(compatible)
    }

    /// Recursive helper that resolves dependencies depth-first.
    ///
    /// For each dependency:
    /// 1. Skip if already visited (circular dependency guard).
    /// 2. Fetch compatible versions from the API.
    /// 3. Pick the latest compatible version.
    /// 4. Recursively resolve its own dependencies.
    fn resolve_recursive<'a>(
        &'a self,
        version: &'a ModVersion,
        game_version: &'a str,
        loader: &'a str,
        resolved: &'a mut Vec<ModVersion>,
        visited: &'a mut HashSet<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
        for dep in &version.dependencies {
            if dep.dependency_type != "required" {
                debug!(
                    "Skipping non-required dependency {} (type: {})",
                    dep.project_id, dep.dependency_type
                );
                continue;
            }

            if !visited.insert(dep.project_id.clone()) {
                debug!(
                    "Already visited dependency {}, skipping to avoid cycle",
                    dep.project_id
                );
                continue;
            }

            debug!(
                "Resolving required dependency {} for project {}",
                dep.project_id, version.project_id
            );

            // If a specific version ID is pinned, try to fetch that exact version.
            let dep_version = if let Some(ref pinned_version_id) = dep.version_id {
                match self.resolve_pinned_version(&dep.project_id, pinned_version_id).await {
                    Some(v) => Some(v),
                    None => {
                        warn!(
                            "Pinned version {} not found for project {}, falling back to latest",
                            pinned_version_id, dep.project_id
                        );
                        None
                    }
                }
            } else {
                None
            };

            let dep_version = match dep_version {
                Some(v) => v,
                None => {
                    // Fetch versions filtered by game_version and loader.
                    let versions = self
                        .modrinth
                        .get_versions(&dep.project_id, Some(game_version), Some(loader))
                        .await
                        .map_err(|e| {
                            let msg = format!(
                                "Failed to fetch versions for dependency {}: {}",
                                dep.project_id, e
                            );
                            error!("{}", msg);
                            msg
                        })?;

                    if versions.is_empty() {
                        let msg = format!(
                            "No compatible version found for required dependency {} ({}/{})",
                            dep.project_id, game_version, loader
                        );
                        error!("{}", msg);
                        return Err(msg);
                    }

                    pick_best_version(&versions).clone()
                }
            };

            info!(
                "Dependency resolved: {} v{} ({})",
                dep_version.project_id, dep_version.version_number, dep_version.version_type
            );

            // Recursively resolve this dependency's own dependencies.
            self.resolve_recursive(&dep_version, game_version, loader, resolved, visited)
                .await?;

            resolved.push(dep_version);
        }

        Ok(())
        })
    }

    /// Attempts to fetch a specific version by project and version ID.
    ///
    /// Returns `None` if the version is not found (does not propagate errors,
    /// allowing the caller to fall back to the latest compatible version).
    async fn resolve_pinned_version(
        &self,
        project_id: &str,
        version_id: &str,
    ) -> Option<ModVersion> {
        // Fetch all versions for the project (no filters) and find the exact match.
        let versions = match self.modrinth.get_versions(project_id, None, None).await {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Failed to fetch versions for pinned dependency {}: {}",
                    project_id, e
                );
                return None;
            }
        };

        versions.into_iter().find(|v| v.id == version_id)
    }
}

/// Picks the best version preferring release > beta > alpha.
///
/// Mirrors the version selection logic in `mod_manager.rs`.
fn pick_best_version(versions: &[ModVersion]) -> &ModVersion {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ModDependency, ModLoader};

    fn make_version(
        project_id: &str,
        version_number: &str,
        version_type: &str,
        dependencies: Vec<ModDependency>,
    ) -> ModVersion {
        ModVersion {
            id: format!("v-{}", project_id),
            project_id: project_id.to_string(),
            version_number: version_number.to_string(),
            game_versions: vec!["1.21".to_string()],
            loaders: vec![ModLoader::Fabric],
            files: vec![],
            dependencies,
            version_type: version_type.to_string(),
        }
    }

    #[test]
    fn pick_best_version_prefers_release() {
        let versions = vec![
            make_version("a", "1.0.0-alpha.1", "alpha", vec![]),
            make_version("b", "1.0.0-beta.1", "beta", vec![]),
            make_version("c", "1.0.0", "release", vec![]),
        ];

        let best = pick_best_version(&versions);
        assert_eq!(best.project_id, "c");
    }

    #[test]
    fn pick_best_version_single() {
        let versions = vec![make_version("a", "0.1.0", "release", vec![])];
        let best = pick_best_version(&versions);
        assert_eq!(best.project_id, "a");
    }
}
