use super::*;
use crate::report::{AssetType, Severity};
use crate::rules::AssetData;
use std::path::Path;

/// Create minimal valid GLB data for testing.
fn create_test_glb() -> Vec<u8> {
    // Minimal valid GLB with a simple triangle
    // This is a hand-crafted minimal GLB file
    let json = r#"{"asset":{"version":"2.0"},"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","max":[1,1,0],"min":[0,0,0]},{"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":6,"byteOffset":36}],"buffers":[{"byteLength":44}]}"#;

    let json_bytes = json.as_bytes();
    let json_len = json_bytes.len();
    // Pad to 4-byte alignment
    let json_padding = (4 - (json_len % 4)) % 4;
    let padded_json_len = json_len + json_padding;

    // Binary chunk: 3 vertices (36 bytes) + 3 indices (6 bytes) = 42 bytes
    // Padded to 44 bytes
    let positions: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices: [u16; 3] = [0, 1, 2];

    let mut bin_data = Vec::new();
    for pos in positions {
        for coord in pos {
            bin_data.extend_from_slice(&coord.to_le_bytes());
        }
    }
    for idx in indices {
        bin_data.extend_from_slice(&idx.to_le_bytes());
    }
    // Pad to 4-byte alignment
    while bin_data.len() % 4 != 0 {
        bin_data.push(0);
    }

    let total_len = 12 + 8 + padded_json_len + 8 + bin_data.len();

    let mut glb = Vec::new();
    // Header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_len as u32).to_le_bytes()); // length

    // JSON chunk
    glb.extend_from_slice(&(padded_json_len as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(0x20); // space padding
    }

    // Binary chunk
    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
    glb.extend_from_slice(&bin_data);

    glb
}

#[test]
fn test_all_rules_registered() {
    let rules = all_rules();
    assert_eq!(rules.len(), 12);

    // Verify all expected rule IDs are present
    let ids: Vec<_> = rules.iter().map(|r| r.id()).collect();
    assert!(ids.contains(&"mesh/non-manifold"));
    assert!(ids.contains(&"mesh/degenerate-faces"));
    assert!(ids.contains(&"mesh/unweighted-verts"));
    assert!(ids.contains(&"mesh/inverted-normals"));
    assert!(ids.contains(&"mesh/humanoid-proportions"));
    assert!(ids.contains(&"mesh/uv-overlap"));
    assert!(ids.contains(&"mesh/uv-stretch"));
    assert!(ids.contains(&"mesh/missing-material"));
    assert!(ids.contains(&"mesh/excessive-ngons"));
    assert!(ids.contains(&"mesh/isolated-verts"));
    assert!(ids.contains(&"mesh/high-poly"));
    assert!(ids.contains(&"mesh/no-uvs"));
}

#[test]
fn test_rules_apply_to_mesh() {
    for rule in all_rules() {
        assert!(
            rule.applies_to().contains(&AssetType::Mesh),
            "Rule {} should apply to Mesh",
            rule.id()
        );
    }
}

#[test]
fn test_error_rules_have_error_severity() {
    let error_ids = [
        "mesh/non-manifold",
        "mesh/degenerate-faces",
        "mesh/unweighted-verts",
        "mesh/inverted-normals",
    ];

    for rule in all_rules() {
        if error_ids.contains(&rule.id()) {
            assert_eq!(
                rule.default_severity(),
                Severity::Error,
                "Rule {} should have Error severity",
                rule.id()
            );
        }
    }
}

#[test]
fn test_warning_rules_have_warning_severity() {
    let warning_ids = [
        "mesh/humanoid-proportions",
        "mesh/uv-overlap",
        "mesh/uv-stretch",
        "mesh/missing-material",
        "mesh/excessive-ngons",
        "mesh/isolated-verts",
    ];

    for rule in all_rules() {
        if warning_ids.contains(&rule.id()) {
            assert_eq!(
                rule.default_severity(),
                Severity::Warning,
                "Rule {} should have Warning severity",
                rule.id()
            );
        }
    }
}

#[test]
fn test_info_rules_have_info_severity() {
    let info_ids = ["mesh/high-poly", "mesh/no-uvs"];

    for rule in all_rules() {
        if info_ids.contains(&rule.id()) {
            assert_eq!(
                rule.default_severity(),
                Severity::Info,
                "Rule {} should have Info severity",
                rule.id()
            );
        }
    }
}

#[test]
fn test_triangle_area_3d() {
    use super::parsing::triangle_area_3d;
    // Unit right triangle in XY plane
    let p0 = [0.0f32, 0.0, 0.0];
    let p1 = [1.0f32, 0.0, 0.0];
    let p2 = [0.0f32, 1.0, 0.0];
    let area = triangle_area_3d(p0, p1, p2);
    assert!((area - 0.5).abs() < 1e-10);
}

#[test]
fn test_triangle_area_2d() {
    use super::parsing::triangle_area_2d;
    let p0 = [0.0f32, 0.0];
    let p1 = [1.0f32, 0.0];
    let p2 = [0.0f32, 1.0];
    let area = triangle_area_2d(p0, p1, p2);
    assert!((area - 0.5).abs() < 1e-10);
}

#[test]
fn test_calculate_centroid() {
    use super::parsing::calculate_centroid;
    let positions = vec![[0.0f32, 0.0, 0.0], [2.0f32, 0.0, 0.0], [1.0f32, 2.0, 0.0]];
    let centroid = calculate_centroid(&positions);
    assert!((centroid[0] - 1.0).abs() < 1e-6);
    assert!((centroid[1] - 2.0 / 3.0).abs() < 1e-6);
    assert!((centroid[2] - 0.0).abs() < 1e-6);
}

#[test]
fn test_parse_valid_glb() {
    use super::parsing::parse_mesh_data;
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let mesh = parse_mesh_data(&asset);
    assert!(mesh.is_some());
    let mesh = mesh.unwrap();
    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.indices.len(), 3);
}

#[test]
fn test_no_uvs_rule() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = NoUvsRule;
    let issues = rule.check(&asset, None);
    // The test GLB has no UVs
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "mesh/no-uvs");
}

#[test]
fn test_non_manifold_rule_clean_mesh() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = NonManifoldRule;
    let issues = rule.check(&asset, None);
    // Simple triangle is manifold
    assert!(issues.is_empty());
}

#[test]
fn test_degenerate_faces_rule_clean_mesh() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = DegenerateFacesRule;
    let issues = rule.check(&asset, None);
    // Test triangle has non-zero area
    assert!(issues.is_empty());
}

#[test]
fn test_high_poly_rule_small_mesh() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = HighPolyRule;
    let issues = rule.check(&asset, None);
    // 1 triangle is well under 50k threshold
    assert!(issues.is_empty());
}

#[test]
fn test_isolated_verts_rule_clean_mesh() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = IsolatedVertsRule;
    let issues = rule.check(&asset, None);
    // All 3 vertices are used by the triangle
    assert!(issues.is_empty());
}

#[test]
fn test_unweighted_verts_rule_non_skinned() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = UnweightedVertsRule;
    let issues = rule.check(&asset, None);
    // Non-skinned mesh shouldn't trigger this rule
    assert!(issues.is_empty());
}

#[test]
fn test_excessive_ngons_rule_glb() {
    let glb_data = create_test_glb();
    let asset = AssetData {
        path: Path::new("test.glb"),
        bytes: &glb_data,
    };
    let rule = ExcessiveNgonsRule;
    let issues = rule.check(&asset, None);
    // GLB is always triangulated, so no ngon warnings
    assert!(issues.is_empty());
}
