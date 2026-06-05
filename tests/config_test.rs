//! Tests for config operations (InstalledModInfo, Config mod management)

use mcl_rs::config::config::{Config, InstalledModInfo};

// ============================================================================
// Helper: create a test InstalledModInfo
// ============================================================================

fn make_mod(project_id: &str, title: &str) -> InstalledModInfo {
    InstalledModInfo {
        project_id: project_id.to_string(),
        slug: title.to_lowercase().replace(' ', "-"),
        title: title.to_string(),
        version_id: format!("{}-ver-1", project_id),
        version_number: "1.0.0".to_string(),
        filename: format!("{}.jar", project_id),
        enabled: true,
        loaders: vec!["fabric".to_string()],
    }
}

fn make_disabled_mod(project_id: &str, title: &str) -> InstalledModInfo {
    let mut m = make_mod(project_id, title);
    m.enabled = false;
    m
}

// ============================================================================
// InstalledModInfo serialization / deserialization
// ============================================================================

#[test]
fn installed_mod_info_serde_roundtrip() {
    let info = InstalledModInfo {
        project_id: "P7dR8mSH".to_string(),
        slug: "sodium".to_string(),
        title: "Sodium".to_string(),
        version_id: "ver-abc".to_string(),
        version_number: "0.5.8".to_string(),
        filename: "sodium-0.5.8.jar".to_string(),
        enabled: true,
        loaders: vec!["fabric".to_string(), "neoforge".to_string()],
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: InstalledModInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.project_id, "P7dR8mSH");
    assert_eq!(deserialized.slug, "sodium");
    assert_eq!(deserialized.title, "Sodium");
    assert_eq!(deserialized.version_id, "ver-abc");
    assert_eq!(deserialized.version_number, "0.5.8");
    assert_eq!(deserialized.filename, "sodium-0.5.8.jar");
    assert!(deserialized.enabled);
    assert_eq!(deserialized.loaders, vec!["fabric", "neoforge"]);
}

#[test]
fn installed_mod_info_serde_disabled() {
    let json = r#"{
        "project_id": "test",
        "slug": "test",
        "title": "Test",
        "version_id": "v1",
        "version_number": "1.0.0",
        "filename": "test.jar",
        "enabled": false,
        "loaders": ["forge"]
    }"#;

    let info: InstalledModInfo = serde_json::from_str(json).unwrap();
    assert!(!info.enabled);
    assert_eq!(info.loaders, vec!["forge"]);
}

#[test]
fn installed_mod_info_serde_empty_loaders() {
    let info = InstalledModInfo {
        project_id: "p1".to_string(),
        slug: "s1".to_string(),
        title: "T1".to_string(),
        version_id: "v1".to_string(),
        version_number: "1.0.0".to_string(),
        filename: "f.jar".to_string(),
        enabled: true,
        loaders: vec![],
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: InstalledModInfo = serde_json::from_str(&json).unwrap();
    assert!(deserialized.loaders.is_empty());
}

// ============================================================================
// Config default values
// ============================================================================

#[test]
fn config_default_java_path() {
    let config = Config::default();
    assert!(config.java_path.is_none());
}

#[test]
fn config_default_memory() {
    let config = Config::default();
    assert!(config.memory.is_none());
}

#[test]
fn config_default_auto_update() {
    let config = Config::default();
    assert!(config.auto_update);
}

#[test]
fn config_default_language() {
    let config = Config::default();
    assert_eq!(config.language, "en");
}

#[test]
fn config_default_max_connections() {
    let config = Config::default();
    assert!(config.max_connections.is_none());
}

#[test]
fn config_default_mods_dir() {
    let config = Config::default();
    assert!(config.mods_dir.is_some());
    let mods_dir = config.mods_dir.unwrap();
    assert!(mods_dir.ends_with("mods"));
}

#[test]
fn config_default_installed_mods_empty() {
    let config = Config::default();
    assert!(config.installed_mods.is_empty());
}

#[test]
fn config_default_saved_session_none() {
    let config = Config::default();
    assert!(config.saved_session.is_none());
}

#[test]
fn config_default_added_versions_empty() {
    let config = Config::default();
    assert!(config.added_versions.is_empty());
}

#[test]
fn config_default_modrinth_user_agent_none() {
    let config = Config::default();
    assert!(config.modrinth_user_agent.is_none());
}

#[test]
fn config_default_game_dir_path() {
    let config = Config::default();
    // game_dir should end with "mcl-rs"
    assert!(config.game_dir.ends_with("mcl-rs"));
}

#[test]
fn config_default_versions_dir_under_game_dir() {
    let config = Config::default();
    assert_eq!(config.versions_dir, config.game_dir.join("versions"));
}

#[test]
fn config_default_assets_dir_under_game_dir() {
    let config = Config::default();
    assert_eq!(config.assets_dir, config.game_dir.join("assets"));
}

// ============================================================================
// Config add_mod
// ============================================================================

#[test]
fn config_add_mod_basic() {
    let mut config = Config::default();
    let mod1 = make_mod("project-alpha", "Alpha Mod");

    config.add_mod("version-1", mod1.clone());

    let mods = config.get_mods("version-1");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].project_id, "project-alpha");
    assert_eq!(mods[0].title, "Alpha Mod");
}

#[test]
fn config_add_mod_multiple() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    config.add_mod("v1", make_mod("p2", "Mod Two"));
    config.add_mod("v1", make_mod("p3", "Mod Three"));

    let mods = config.get_mods("v1");
    assert_eq!(mods.len(), 3);
}

#[test]
fn config_add_mod_different_versions() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod A"));
    config.add_mod("v2", make_mod("p2", "Mod B"));

    assert_eq!(config.get_mods("v1").len(), 1);
    assert_eq!(config.get_mods("v2").len(), 1);
    assert_eq!(config.get_mods("v1")[0].project_id, "p1");
    assert_eq!(config.get_mods("v2")[0].project_id, "p2");
}

#[test]
fn config_add_mod_no_duplicates() {
    let mut config = Config::default();
    let mod1 = make_mod("p1", "Mod A");
    let mod2 = make_mod("p1", "Mod A Updated");

    config.add_mod("v1", mod1);
    config.add_mod("v1", mod2); // should NOT be added (same project_id)

    let mods = config.get_mods("v1");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].title, "Mod A"); // original preserved
}

#[test]
fn config_add_mod_creates_version_entry() {
    let mut config = Config::default();
    assert!(config.get_mods("nonexistent").is_empty());

    config.add_mod("v1", make_mod("p1", "Mod"));

    assert!(config.installed_mods.contains_key("v1"));
}

// ============================================================================
// Config remove_mod
// ============================================================================

#[test]
fn config_remove_mod_basic() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    config.add_mod("v1", make_mod("p2", "Mod Two"));

    config.remove_mod("v1", "p1");

    let mods = config.get_mods("v1");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].project_id, "p2");
}

#[test]
fn config_remove_mod_all() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    config.remove_mod("v1", "p1");

    let mods = config.get_mods("v1");
    assert!(mods.is_empty());
}

#[test]
fn config_remove_mod_nonexistent_project() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    // Removing a project that doesn't exist should be a no-op
    config.remove_mod("v1", "nonexistent");

    let mods = config.get_mods("v1");
    assert_eq!(mods.len(), 1);
}

#[test]
fn config_remove_mod_nonexistent_version() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    // Removing from a version that doesn't exist should be a no-op
    config.remove_mod("v999", "p1");

    let mods = config.get_mods("v1");
    assert_eq!(mods.len(), 1);
}

// ============================================================================
// Config toggle_mod
// ============================================================================

#[test]
fn config_toggle_mod_enable_to_disable() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    assert!(config.get_mods("v1")[0].enabled);

    config.toggle_mod("v1", "p1");

    let mods = config.get_mods("v1");
    assert!(!mods[0].enabled);
}

#[test]
fn config_toggle_mod_disable_to_enable() {
    let mut config = Config::default();
    config.add_mod("v1", make_disabled_mod("p1", "Mod One"));
    assert!(!config.get_mods("v1")[0].enabled);

    config.toggle_mod("v1", "p1");

    let mods = config.get_mods("v1");
    assert!(mods[0].enabled);
}

#[test]
fn config_toggle_mod_double_toggle() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    let original_enabled = config.get_mods("v1")[0].enabled;

    config.toggle_mod("v1", "p1");
    config.toggle_mod("v1", "p1");

    let mods = config.get_mods("v1");
    assert_eq!(mods[0].enabled, original_enabled);
}

#[test]
fn config_toggle_mod_only_affects_target() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    config.add_mod("v1", make_mod("p2", "Mod Two"));

    config.toggle_mod("v1", "p1");

    let mods = config.get_mods("v1");
    let p1 = mods.iter().find(|m| m.project_id == "p1").unwrap();
    let p2 = mods.iter().find(|m| m.project_id == "p2").unwrap();
    assert!(!p1.enabled);
    assert!(p2.enabled);
}

#[test]
fn config_toggle_mod_nonexistent_no_panic() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    // Should be a no-op, no panic
    config.toggle_mod("v1", "nonexistent");

    let mods = config.get_mods("v1");
    assert!(mods[0].enabled); // unchanged
}

#[test]
fn config_toggle_mod_nonexistent_version_no_panic() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    // Should be a no-op, no panic
    config.toggle_mod("v999", "p1");

    let mods = config.get_mods("v1");
    assert!(mods[0].enabled); // unchanged
}

// ============================================================================
// Config get_mods
// ============================================================================

#[test]
fn config_get_mods_nonexistent_version_returns_empty() {
    let config = Config::default();
    let mods = config.get_mods("nonexistent");
    assert!(mods.is_empty());
}

#[test]
fn config_get_mods_returns_clone() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));

    let mods1 = config.get_mods("v1");
    let mods2 = config.get_mods("v1");

    // Both calls return independent clones
    assert_eq!(mods1.len(), mods2.len());
}

// ============================================================================
// Config serialization (partial — just mod-related fields)
// ============================================================================

#[test]
fn config_serde_installed_mods_roundtrip() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Mod One"));
    config.add_mod("v1", make_mod("p2", "Mod Two"));
    config.add_mod("v2", make_mod("p3", "Mod Three"));

    let json = serde_json::to_string(&config).unwrap();

    // Don't deserialize into Config because Config::default() depends on
    // the platform-specific config_dir. Instead, check the JSON directly
    // for the installed_mods key.
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let installed = &parsed["installed_mods"];

    assert!(installed.is_object());
    let v1_mods = &installed["v1"];
    assert!(v1_mods.is_array());
    assert_eq!(v1_mods.as_array().unwrap().len(), 2);

    let v2_mods = &installed["v2"];
    assert!(v2_mods.is_array());
    assert_eq!(v2_mods.as_array().unwrap().len(), 1);
}

#[test]
fn config_serde_installed_mod_info_fields() {
    let mut config = Config::default();
    config.add_mod("v1", make_mod("p1", "Test Mod"));

    let json = serde_json::to_string(&config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let mod_entry = &parsed["installed_mods"]["v1"][0];
    assert_eq!(mod_entry["project_id"], "p1");
    assert_eq!(mod_entry["title"], "Test Mod");
    assert_eq!(mod_entry["slug"], "test-mod");
    assert_eq!(mod_entry["enabled"], true);
    assert_eq!(mod_entry["filename"], "p1.jar");
}

// ============================================================================
// Config serde roundtrip (deserialization from JSON)
// ============================================================================

#[test]
fn config_serde_deserialize_with_installed_mods() {
    let json = r#"{
        "java_path": null,
        "memory": null,
        "auto_update": true,
        "game_dir": "/tmp/test-mcl",
        "versions_dir": "/tmp/test-mcl/versions",
        "assets_dir": "/tmp/test-mcl/assets",
        "language": "en",
        "installed_mods": {
            "version-uuid-1": [
                {
                    "project_id": "proj-1",
                    "slug": "sodium",
                    "title": "Sodium",
                    "version_id": "ver-1",
                    "version_number": "0.5.8",
                    "filename": "sodium-0.5.8.jar",
                    "enabled": true,
                    "loaders": ["fabric"]
                }
            ]
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    let mods = config.get_mods("version-uuid-1");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].project_id, "proj-1");
    assert_eq!(mods[0].title, "Sodium");
    assert!(mods[0].enabled);
}

#[test]
fn config_serde_deserialize_empty_installed_mods() {
    let json = r#"{
        "java_path": null,
        "auto_update": true,
        "game_dir": "/tmp/test",
        "versions_dir": "/tmp/test/versions",
        "assets_dir": "/tmp/test/assets",
        "installed_mods": {}
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert!(config.installed_mods.is_empty());
}

// ============================================================================
// Config: full add → remove → toggle lifecycle
// ============================================================================

#[test]
fn config_mod_lifecycle() {
    let mut config = Config::default();

    // Add 3 mods
    config.add_mod("v1", make_mod("p1", "Mod A"));
    config.add_mod("v1", make_mod("p2", "Mod B"));
    config.add_mod("v1", make_mod("p3", "Mod C"));
    assert_eq!(config.get_mods("v1").len(), 3);

    // Toggle Mod B off
    config.toggle_mod("v1", "p2");
    let b = config
        .get_mods("v1")
        .into_iter()
        .find(|m| m.project_id == "p2")
        .unwrap();
    assert!(!b.enabled);

    // Remove Mod A
    config.remove_mod("v1", "p1");
    assert_eq!(config.get_mods("v1").len(), 2);

    // Toggle Mod B back on
    config.toggle_mod("v1", "p2");
    let b = config
        .get_mods("v1")
        .into_iter()
        .find(|m| m.project_id == "p2")
        .unwrap();
    assert!(b.enabled);

    // Remove Mod C
    config.remove_mod("v1", "p3");
    assert_eq!(config.get_mods("v1").len(), 1);

    // Only Mod B remains
    assert_eq!(config.get_mods("v1")[0].project_id, "p2");
}
