// Core Module
// Contains business logic and data structures for the launcher

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Account management and storage
pub mod account;

/// Minecraft version manifest and version data structures
pub mod version;

/// File download management with progress tracking
pub mod download;

/// Microsoft/Xbox OAuth authentication flow
pub mod auth;

/// Update checking and version comparison
pub mod update;

/// Game launching logic
pub mod launch;

/// Modrinth API client for mod search, project lookup, and downloads
pub mod modrinth;

/// High-level mod management orchestrator (search, install, uninstall, enable/disable, update)
pub mod mod_manager;

/// Recursive dependency resolver for mod installation
pub mod dependency;

// ============================================================================
// Mod Management Data Structures
// ============================================================================

/// Supported Minecraft mod loaders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Fabric,
    Forge,
    NeoForge,
    Quilt,
    Rift,
}

impl fmt::Display for ModLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModLoader::Fabric => write!(f, "Fabric"),
            ModLoader::Forge => write!(f, "Forge"),
            ModLoader::NeoForge => write!(f, "NeoForge"),
            ModLoader::Quilt => write!(f, "Quilt"),
            ModLoader::Rift => write!(f, "Rift"),
        }
    }
}

impl FromStr for ModLoader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fabric" => Ok(ModLoader::Fabric),
            "forge" => Ok(ModLoader::Forge),
            "neoforge" => Ok(ModLoader::NeoForge),
            "quilt" => Ok(ModLoader::Quilt),
            "rift" => Ok(ModLoader::Rift),
            _ => Err(format!("Unknown mod loader: '{}'", s)),
        }
    }
}

impl ModLoader {
    /// Converts a string to a `ModLoader`, defaulting to `Fabric` on unknown input.
    /// Matches the pattern used by [`VersionType::from_str`].
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fabric" => ModLoader::Fabric,
            "forge" => ModLoader::Forge,
            "neoforge" => ModLoader::NeoForge,
            "quilt" => ModLoader::Quilt,
            "rift" => ModLoader::Rift,
            _ => ModLoader::Fabric,
        }
    }

    /// Returns all known loader identifier strings.
    pub fn all_variants() -> &'static [&'static str] {
        &["fabric", "forge", "neoforge", "quilt", "rift"]
    }
}

// ---------------------------------------------------------------------------
// ModInfo
// ---------------------------------------------------------------------------

/// Represents an installed or tracked Minecraft mod.
///
/// Combines project metadata from Modrinth with local installation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// Modrinth project ID (e.g. "P7dR8mSH")
    pub id: String,
    /// Project URL slug
    pub slug: String,
    /// Display title
    pub title: String,
    /// Project description
    pub description: String,
    /// URL to the project's icon
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Compatible mod loaders
    #[serde(default)]
    pub loaders: Vec<ModLoader>,
    /// Compatible Minecraft version strings
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// Local file system path if installed
    #[serde(default)]
    pub installed_path: Option<PathBuf>,
    /// Whether the mod is currently enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Installed version ID (from Modrinth version object)
    pub version_id: String,
    /// Human-readable version number (e.g. "1.4.7")
    pub version_number: String,
}

/// Default value for [`ModInfo::enabled`].
const fn default_enabled() -> bool {
    true
}

// ---------------------------------------------------------------------------
// ModVersion
// ---------------------------------------------------------------------------

/// A specific version of a mod project from Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    /// Unique version ID
    pub id: String,
    /// The project this version belongs to
    pub project_id: String,
    /// Human-readable version number (e.g. "1.4.7")
    pub version_number: String,
    /// Compatible game version strings (e.g. "1.20.1")
    pub game_versions: Vec<String>,
    /// Compatible mod loaders for this version
    pub loaders: Vec<ModLoader>,
    /// Downloadable files for this version
    #[serde(default)]
    pub files: Vec<ModFile>,
    /// Dependency requirements for this version
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    /// Version type (e.g. "release", "beta", "alpha")
    pub version_type: String,
}

// ---------------------------------------------------------------------------
// ModFile
// ---------------------------------------------------------------------------

/// A downloadable file attached to a mod version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    /// Download URL for the file
    pub url: String,
    /// File name (e.g. "my-mod-1.4.7.jar")
    pub filename: String,
    /// Whether this is the primary file for the version
    #[serde(default)]
    pub primary: bool,
    /// File size in bytes
    pub size: u64,
    /// File hashes keyed by algorithm name (e.g. "sha1", "sha512")
    pub hashes: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// ModDependency
// ---------------------------------------------------------------------------

/// A dependency of a mod version on another project or version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// The dependent project's Modrinth ID
    pub project_id: String,
    /// Specific version ID if pinned (None = any version)
    pub version_id: Option<String>,
    /// Type of dependency relationship:
    /// "required", "optional", "incompatible", or "embedded"
    pub dependency_type: String,
}

// ---------------------------------------------------------------------------
// ModSearchResult
// ---------------------------------------------------------------------------

/// A paginated search result from the Modrinth search API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    /// Search hit entries for this page
    pub hits: Vec<ModSearchHit>,
    /// Number of results skipped (pagination offset)
    pub offset: usize,
    /// Maximum results per page
    pub limit: usize,
    /// Total number of matching results across all pages
    pub total_hits: usize,
}

// ---------------------------------------------------------------------------
// ModSearchHit
// ---------------------------------------------------------------------------

/// A single project entry from the Modrinth search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchHit {
    /// Modrinth project ID
    pub project_id: String,
    /// Project URL slug
    pub slug: String,
    /// Display title
    pub title: String,
    /// Project description
    pub description: String,
    /// Total download count
    pub downloads: u64,
    /// URL to the project icon
    pub icon_url: Option<String>,
    /// Recent version strings
    #[serde(default)]
    pub versions: Vec<String>,
    /// Compatible mod loader names (e.g. "fabric", "forge")
    #[serde(default)]
    pub loaders: Vec<String>,
    /// Project categories / tags
    #[serde(default)]
    pub categories: Vec<String>,
    /// Type of project (e.g. "mod", "modpack", "resourcepack")
    pub project_type: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Maps a string to its corresponding [`ModLoader`].
///
/// This is a case-insensitive free-function variant. Returns `None` for
/// unrecognised loader names.
pub fn parse_mod_loader(s: &str) -> Option<ModLoader> {
    match s.to_lowercase().as_str() {
        "fabric" => Some(ModLoader::Fabric),
        "forge" => Some(ModLoader::Forge),
        "neoforge" => Some(ModLoader::NeoForge),
        "quilt" => Some(ModLoader::Quilt),
        "rift" => Some(ModLoader::Rift),
        _ => None,
    }
}