//! Tests for the Modrinth client helpers (build_search_url, facet construction)

use mcl_rs::core::modrinth::build_search_url;

// ============================================================================
// build_search_url: basic queries
// ============================================================================

#[test]
fn build_search_url_basic_query() {
    let url = build_search_url("sodium", None, None, 0, 20);

    assert!(url.starts_with("https://api.modrinth.com/v2/search?"));
    assert!(url.contains("query=sodium"));
    assert!(url.contains("offset=0"));
    assert!(url.contains("limit=20"));
}

#[test]
fn build_search_url_with_offset_and_limit() {
    let url = build_search_url("lithium", None, None, 40, 10);

    assert!(url.contains("offset=40"));
    assert!(url.contains("limit=10"));
    assert!(url.contains("query=lithium"));
}

#[test]
fn build_search_url_empty_query() {
    let url = build_search_url("", None, None, 0, 20);

    assert!(url.contains("query="));
    assert!(url.starts_with("https://api.modrinth.com/v2/search?"));
}

// ============================================================================
// build_search_url: facet construction
// ============================================================================

#[test]
fn build_search_url_always_has_project_type_mod_facet() {
    let url = build_search_url("test", None, None, 0, 10);

    // The URL should contain the facets parameter with project_type:mod
    assert!(url.contains("facets="));

    // Decode the facets to check the content
    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    assert!(facets_decoded.contains("project_type:mod"));
}

#[test]
fn build_search_url_loader_facet() {
    let url = build_search_url("test", None, Some("fabric"), 0, 10);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    assert!(facets_decoded.contains("categories:fabric"));
    assert!(facets_decoded.contains("project_type:mod"));
}

#[test]
fn build_search_url_game_version_facet() {
    let url = build_search_url("test", Some("1.21"), None, 0, 10);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    assert!(facets_decoded.contains("versions:1.21"));
    assert!(facets_decoded.contains("project_type:mod"));
}

#[test]
fn build_search_url_both_facets() {
    let url = build_search_url("test", Some("1.20.1"), Some("forge"), 0, 20);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    assert!(facets_decoded.contains("project_type:mod"));
    assert!(facets_decoded.contains("categories:forge"));
    assert!(facets_decoded.contains("versions:1.20.1"));
}

// ============================================================================
// build_search_url: facet JSON structure
// ============================================================================

#[test]
fn build_search_url_facets_are_json_array_of_arrays() {
    let url = build_search_url("test", Some("1.21"), Some("fabric"), 0, 20);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    // Should start with [[ and end with ]]
    // Each facet should be a JSON array: ["key:value"]
    let facets_value: serde_json::Value =
        serde_json::from_str(&facets_decoded).expect("facets should be valid JSON");

    assert!(facets_value.is_array());

    let arr = facets_value.as_array().unwrap();
    // Should have 3 facets: project_type:mod, categories:fabric, versions:1.21
    assert_eq!(arr.len(), 3);

    for item in arr {
        assert!(item.is_array(), "each facet should be a JSON array");
        let inner = item.as_array().unwrap();
        assert_eq!(inner.len(), 1, "each facet array should have exactly one element");
        assert!(inner[0].is_string());
    }
}

#[test]
fn build_search_url_facet_values_correct() {
    let url = build_search_url("test", Some("1.21"), Some("neoforge"), 0, 20);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    let facets_value: serde_json::Value = serde_json::from_str(&facets_decoded).unwrap();
    let arr = facets_value.as_array().unwrap();

    let facet_strs: Vec<String> = arr
        .iter()
        .map(|v| v.as_array().unwrap()[0].as_str().unwrap().to_string())
        .collect();

    assert!(facet_strs.contains(&"project_type:mod".to_string()));
    assert!(facet_strs.contains(&"categories:neoforge".to_string()));
    assert!(facet_strs.contains(&"versions:1.21".to_string()));
}

#[test]
fn build_search_url_no_loader_no_version_only_project_type() {
    let url = build_search_url("test", None, None, 0, 20);

    let facets_part = url.split("facets=").nth(1).unwrap();
    let facets_encoded = facets_part.split('&').next().unwrap();
    let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

    let facets_value: serde_json::Value = serde_json::from_str(&facets_decoded).unwrap();
    let arr = facets_value.as_array().unwrap();

    // Only project_type:mod
    assert_eq!(arr.len(), 1);
    assert_eq!(
        arr[0].as_array().unwrap()[0].as_str().unwrap(),
        "project_type:mod"
    );
}

// ============================================================================
// build_search_url: URL encoding
// ============================================================================

#[test]
fn build_search_url_query_is_encoded() {
    let url = build_search_url("hello world", None, None, 0, 20);

    // "hello world" should be encoded as "hello%20world"
    assert!(url.contains("query=hello%20world"));
}

#[test]
fn build_search_url_special_characters_encoded() {
    let url = build_search_url("mod&name=test", None, None, 0, 20);

    // The query part should be encoded
    let query_part = url.split("query=").nth(1).unwrap();
    assert!(query_part.contains("%26") || query_part.contains("mod%26name%3Dtest"));
}

// ============================================================================
// build_search_url: URL structure
// ============================================================================

#[test]
fn build_search_url_base_url() {
    let url = build_search_url("test", None, None, 0, 20);
    assert!(url.starts_with("https://api.modrinth.com/v2/search?"));
}

#[test]
fn build_search_url_parameter_order() {
    let url = build_search_url("test", Some("1.21"), Some("fabric"), 0, 20);

    let facets_pos = url.find("facets=").unwrap();
    let offset_pos = url.find("offset=").unwrap();
    let limit_pos = url.find("limit=").unwrap();
    let query_pos = url.find("query=").unwrap();

    // Parameters should appear in this order: facets, offset, limit, query
    assert!(facets_pos < offset_pos);
    assert!(offset_pos < limit_pos);
    assert!(limit_pos < query_pos);
}

// ============================================================================
// build_search_url: edge cases
// ============================================================================

#[test]
fn build_search_url_zero_offset_zero_limit() {
    let url = build_search_url("test", None, None, 0, 0);
    assert!(url.contains("offset=0"));
    assert!(url.contains("limit=0"));
}

#[test]
fn build_search_url_large_offset() {
    let url = build_search_url("test", None, None, 10000, 20);
    assert!(url.contains("offset=10000"));
}

#[test]
fn build_search_url_all_loader_types() {
    for loader in &["fabric", "forge", "neoforge", "quilt", "rift"] {
        let url = build_search_url("test", None, Some(loader), 0, 10);
        let facets_part = url.split("facets=").nth(1).unwrap();
        let facets_encoded = facets_part.split('&').next().unwrap();
        let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

        assert!(
            facets_decoded.contains(&format!("categories:{}", loader)),
            "Expected facets to contain categories:{} for loader '{}'",
            loader,
            loader
        );
    }
}

#[test]
fn build_search_url_various_game_versions() {
    for version in &["1.20.1", "1.21", "1.21.4", "1.19.2"] {
        let url = build_search_url("test", Some(version), None, 0, 10);
        let facets_part = url.split("facets=").nth(1).unwrap();
        let facets_encoded = facets_part.split('&').next().unwrap();
        let facets_decoded = urlencoding::decode(facets_encoded).unwrap();

        assert!(
            facets_decoded.contains(&format!("versions:{}", version)),
            "Expected facets to contain versions:{}",
            version
        );
    }
}
