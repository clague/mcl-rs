//! Tests for mod management data structures (ModLoader, ModInfo, ModVersion, etc.)

use std::collections::HashMap;
use std::path::PathBuf;

use mcl_rs::core::{
    parse_mod_loader, ModDependency, ModFile, ModInfo, ModLoader, ModSearchHit, ModSearchResult,
    ModVersion,
};

// ============================================================================
// ModLoader::from_str (inherent method – defaults to Fabric on unknown input)
// ============================================================================

#[test]
fn mod_loader_from_str_fabric() {
    assert_eq!(ModLoader::from_str("fabric"), ModLoader::Fabric);
}

#[test]
fn mod_loader_from_str_forge() {
    assert_eq!(ModLoader::from_str("forge"), ModLoader::Forge);
}

#[test]
fn mod_loader_from_str_neoforge() {
    assert_eq!(ModLoader::from_str("neoforge"), ModLoader::NeoForge);
}

#[test]
fn mod_loader_from_str_quilt() {
    assert_eq!(ModLoader::from_str("quilt"), ModLoader::Quilt);
}

#[test]
fn mod_loader_from_str_rift() {
    assert_eq!(ModLoader::from_str("rift"), ModLoader::Rift);
}

#[test]
fn mod_loader_from_str_case_insensitive() {
    assert_eq!(ModLoader::from_str("FABRIC"), ModLoader::Fabric);
    assert_eq!(ModLoader::from_str("Forge"), ModLoader::Forge);
    assert_eq!(ModLoader::from_str("NEOFORGE"), ModLoader::NeoForge);
}

#[test]
fn mod_loader_from_str_invalid_defaults_to_fabric() {
    assert_eq!(ModLoader::from_str(""), ModLoader::Fabric);
    assert_eq!(ModLoader::from_str("bukkit"), ModLoader::Fabric);
    assert_eq!(ModLoader::from_str("unknown_loader"), ModLoader::Fabric);
}

// ============================================================================
// ModLoader FromStr trait (returns Result)
// ============================================================================

#[test]
fn mod_loader_from_str_trait_valid() {
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("fabric").unwrap(),
        ModLoader::Fabric
    );
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("forge").unwrap(),
        ModLoader::Forge
    );
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("neoforge").unwrap(),
        ModLoader::NeoForge
    );
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("quilt").unwrap(),
        ModLoader::Quilt
    );
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("rift").unwrap(),
        ModLoader::Rift
    );
}

#[test]
fn mod_loader_from_str_trait_invalid() {
    let result = <ModLoader as std::str::FromStr>::from_str("unknown");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown mod loader"));
}

#[test]
fn mod_loader_from_str_trait_case_insensitive() {
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("FABRIC").unwrap(),
        ModLoader::Fabric
    );
    assert_eq!(
        <ModLoader as std::str::FromStr>::from_str("NeoForge").unwrap(),
        ModLoader::NeoForge
    );
}

// ============================================================================
// ModLoader Display trait
// ============================================================================

#[test]
fn mod_loader_display() {
    assert_eq!(ModLoader::Fabric.to_string(), "Fabric");
    assert_eq!(ModLoader::Forge.to_string(), "Forge");
    assert_eq!(ModLoader::NeoForge.to_string(), "NeoForge");
    assert_eq!(ModLoader::Quilt.to_string(), "Quilt");
    assert_eq!(ModLoader::Rift.to_string(), "Rift");
}

// ============================================================================
// ModLoader::all_variants
// ============================================================================

#[test]
fn mod_loader_all_variants() {
    let variants = ModLoader::all_variants();
    assert_eq!(variants.len(), 5);
    assert!(variants.contains(&"fabric"));
    assert!(variants.contains(&"forge"));
    assert!(variants.contains(&"neoforge"));
    assert!(variants.contains(&"quilt"));
    assert!(variants.contains(&"rift"));
}

// ============================================================================
// parse_mod_loader free function
// ============================================================================

#[test]
fn parse_mod_loader_valid() {
    assert_eq!(parse_mod_loader("fabric"), Some(ModLoader::Fabric));
    assert_eq!(parse_mod_loader("forge"), Some(ModLoader::Forge));
    assert_eq!(parse_mod_loader("neoforge"), Some(ModLoader::NeoForge));
    assert_eq!(parse_mod_loader("quilt"), Some(ModLoader::Quilt));
    assert_eq!(parse_mod_loader("rift"), Some(ModLoader::Rift));
}

#[test]
fn parse_mod_loader_invalid() {
    assert_eq!(parse_mod_loader("unknown"), None);
    assert_eq!(parse_mod_loader(""), None);
    assert_eq!(parse_mod_loader("bukkit"), None);
}

#[test]
fn parse_mod_loader_case_insensitive() {
    assert_eq!(parse_mod_loader("FABRIC"), Some(ModLoader::Fabric));
    assert_eq!(parse_mod_loader("Forge"), Some(ModLoader::Forge));
}

// ============================================================================
// ModLoader serde (serialize / deserialize)
// ============================================================================

#[test]
fn mod_loader_serde_roundtrip() {
    let loaders = vec![
        ModLoader::Fabric,
        ModLoader::Forge,
        ModLoader::NeoForge,
        ModLoader::Quilt,
        ModLoader::Rift,
    ];
    let json = serde_json::to_string(&loaders).unwrap();
    let deserialized: Vec<ModLoader> = serde_json::from_str(&json).unwrap();
    assert_eq!(loaders, deserialized);
}

#[test]
fn mod_loader_serde_lowercase_json() {
    // The enum uses #[serde(rename_all = "lowercase")]
    let json = r#""fabric""#;
    let loader: ModLoader = serde_json::from_str(json).unwrap();
    assert_eq!(loader, ModLoader::Fabric);

    let json = r#""neoforge""#;
    let loader: ModLoader = serde_json::from_str(json).unwrap();
    assert_eq!(loader, ModLoader::NeoForge);
}

// ============================================================================
// ModInfo serialization / deserialization
// ============================================================================

#[test]
fn mod_info_serde_roundtrip() {
    let info = ModInfo {
        id: "P7dR8mSH".to_string(),
        slug: "sodium".to_string(),
        title: "Sodium".to_string(),
        description: "A Minecraft mod for improved performance".to_string(),
        icon_url: Some("https://example.com/icon.png".to_string()),
        loaders: vec![ModLoader::Fabric, ModLoader::NeoForge],
        game_versions: vec!["1.20.1".to_string(), "1.21".to_string()],
        installed_path: Some(PathBuf::from("/mods/sodium.jar")),
        enabled: true,
        version_id: "abc123".to_string(),
        version_number: "0.5.8".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ModInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "P7dR8mSH");
    assert_eq!(deserialized.slug, "sodium");
    assert_eq!(deserialized.title, "Sodium");
    assert_eq!(deserialized.description, "A Minecraft mod for improved performance");
    assert_eq!(deserialized.icon_url, Some("https://example.com/icon.png".to_string()));
    assert_eq!(deserialized.loaders, vec![ModLoader::Fabric, ModLoader::NeoForge]);
    assert_eq!(deserialized.game_versions, vec!["1.20.1", "1.21"]);
    assert_eq!(deserialized.installed_path, Some(PathBuf::from("/mods/sodium.jar")));
    assert!(deserialized.enabled);
    assert_eq!(deserialized.version_id, "abc123");
    assert_eq!(deserialized.version_number, "0.5.8");
}

#[test]
fn mod_info_serde_optional_fields_default() {
    let json = r#"{
        "id": "test123",
        "slug": "test-mod",
        "title": "Test Mod",
        "description": "A test mod",
        "version_id": "v1",
        "version_number": "1.0.0"
    }"#;

    let info: ModInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.id, "test123");
    assert!(info.icon_url.is_none());
    assert!(info.loaders.is_empty());
    assert!(info.game_versions.is_empty());
    assert!(info.installed_path.is_none());
    assert!(info.enabled); // default_enabled() returns true
}

#[test]
fn mod_info_serde_disabled_mod() {
    let info = ModInfo {
        id: "test".to_string(),
        slug: "test".to_string(),
        title: "Test".to_string(),
        description: "Test".to_string(),
        icon_url: None,
        loaders: vec![],
        game_versions: vec![],
        installed_path: None,
        enabled: false,
        version_id: "v1".to_string(),
        version_number: "1.0.0".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ModInfo = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.enabled);
}

// ============================================================================
// ModVersion serialization / deserialization
// ============================================================================

#[test]
fn mod_version_serde_roundtrip() {
    let version = ModVersion {
        id: "version-abc".to_string(),
        project_id: "project-123".to_string(),
        version_number: "2.1.0".to_string(),
        game_versions: vec!["1.20.1".to_string()],
        loaders: vec![ModLoader::Fabric],
        files: vec![ModFile {
            url: "https://example.com/mod.jar".to_string(),
            filename: "mod-2.1.0.jar".to_string(),
            primary: true,
            size: 1024000,
            hashes: HashMap::from([
                ("sha1".to_string(), "abc123def456".to_string()),
            ]),
        }],
        dependencies: vec![ModDependency {
            project_id: "dep-project".to_string(),
            version_id: Some("dep-version".to_string()),
            dependency_type: "required".to_string(),
        }],
        version_type: "release".to_string(),
    };

    let json = serde_json::to_string(&version).unwrap();
    let deserialized: ModVersion = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "version-abc");
    assert_eq!(deserialized.project_id, "project-123");
    assert_eq!(deserialized.version_number, "2.1.0");
    assert_eq!(deserialized.game_versions, vec!["1.20.1"]);
    assert_eq!(deserialized.loaders, vec![ModLoader::Fabric]);
    assert_eq!(deserialized.files.len(), 1);
    assert_eq!(deserialized.files[0].filename, "mod-2.1.0.jar");
    assert!(deserialized.files[0].primary);
    assert_eq!(deserialized.files[0].size, 1024000);
    assert_eq!(deserialized.dependencies.len(), 1);
    assert_eq!(deserialized.dependencies[0].dependency_type, "required");
    assert_eq!(deserialized.version_type, "release");
}

#[test]
fn mod_version_serde_optional_dependencies_default() {
    let json = r#"{
        "id": "v1",
        "project_id": "p1",
        "version_number": "1.0.0",
        "game_versions": ["1.20.1"],
        "loaders": ["fabric"],
        "version_type": "release"
    }"#;

    let version: ModVersion = serde_json::from_str(json).unwrap();
    assert!(version.files.is_empty());
    assert!(version.dependencies.is_empty());
}

#[test]
fn mod_version_serde_multiple_files() {
    let json = r#"{
        "id": "v1",
        "project_id": "p1",
        "version_number": "1.0.0",
        "game_versions": [],
        "loaders": [],
        "files": [
            {
                "url": "https://example.com/main.jar",
                "filename": "main.jar",
                "primary": true,
                "size": 1024,
                "hashes": {"sha1": "abc"}
            },
            {
                "url": "https://example.com/sources.jar",
                "filename": "sources.jar",
                "primary": false,
                "size": 2048,
                "hashes": {}
            }
        ],
        "version_type": "release"
    }"#;

    let version: ModVersion = serde_json::from_str(json).unwrap();
    assert_eq!(version.files.len(), 2);
    assert!(version.files[0].primary);
    assert!(!version.files[1].primary);
    assert_eq!(version.files[0].size, 1024);
    assert_eq!(version.files[1].size, 2048);
}

// ============================================================================
// ModSearchResult serialization / deserialization
// ============================================================================

#[test]
fn mod_search_result_serde_roundtrip() {
    let result = ModSearchResult {
        hits: vec![ModSearchHit {
            project_id: "p1".to_string(),
            slug: "my-mod".to_string(),
            title: "My Mod".to_string(),
            description: "A cool mod".to_string(),
            downloads: 50000,
            icon_url: None,
            versions: vec!["1.20.1".to_string()],
            loaders: vec!["fabric".to_string()],
            categories: vec!["performance".to_string()],
            project_type: "mod".to_string(),
        }],
        offset: 0,
        limit: 20,
        total_hits: 1,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ModSearchResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_hits, 1);
    assert_eq!(deserialized.hits.len(), 1);
    assert_eq!(deserialized.hits[0].title, "My Mod");
    assert_eq!(deserialized.hits[0].downloads, 50000);
    assert_eq!(deserialized.hits[0].project_type, "mod");
}

#[test]
fn mod_search_result_empty() {
    let json = r#"{"hits":[],"offset":0,"limit":20,"total_hits":0}"#;
    let result: ModSearchResult = serde_json::from_str(json).unwrap();
    assert!(result.hits.is_empty());
    assert_eq!(result.total_hits, 0);
}

// ============================================================================
// ModSearchHit serialization / deserialization
// ============================================================================

#[test]
fn mod_search_hit_optional_icon_url() {
    let json = r#"{
        "project_id": "p1",
        "slug": "test",
        "title": "Test",
        "description": "Test",
        "downloads": 0,
        "project_type": "mod"
    }"#;

    let hit: ModSearchHit = serde_json::from_str(json).unwrap();
    assert!(hit.icon_url.is_none());
    assert!(hit.versions.is_empty());
    assert!(hit.loaders.is_empty());
    assert!(hit.categories.is_empty());
}

// ============================================================================
// ModFile serialization / deserialization
// ============================================================================

#[test]
fn mod_file_serde_primary_default() {
    let json = r#"{
        "url": "https://example.com/file.jar",
        "filename": "file.jar",
        "size": 512,
        "hashes": {}
    }"#;

    let file: ModFile = serde_json::from_str(json).unwrap();
    assert!(!file.primary); // serde(default) on bool is false
}

// ============================================================================
// ModDependency serialization / deserialization
// ============================================================================

#[test]
fn mod_dependency_optional_version_id() {
    let json = r#"{
        "project_id": "dep1",
        "dependency_type": "optional"
    }"#;

    let dep: ModDependency = serde_json::from_str(json).unwrap();
    assert_eq!(dep.project_id, "dep1");
    assert!(dep.version_id.is_none());
    assert_eq!(dep.dependency_type, "optional");
}

#[test]
fn mod_dependency_all_types() {
    for dep_type in &["required", "optional", "incompatible", "embedded"] {
        let json = format!(
            r#"{{"project_id":"p1","dependency_type":"{}"}}"#,
            dep_type
        );
        let dep: ModDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(dep.dependency_type, *dep_type);
    }
}

// ============================================================================
// ModLoader PartialEq / Debug
// ============================================================================

#[test]
fn mod_loader_equality() {
    assert_eq!(ModLoader::Fabric, ModLoader::Fabric);
    assert_ne!(ModLoader::Fabric, ModLoader::Forge);
    assert_ne!(ModLoader::NeoForge, ModLoader::Quilt);
}

#[test]
fn mod_loader_debug() {
    let debug_str = format!("{:?}", ModLoader::Fabric);
    assert_eq!(debug_str, "Fabric");
}
