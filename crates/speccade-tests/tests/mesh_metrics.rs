//! Integration tests for mesh metrics collection (MESHVER-001).
//!
//! Tests the geometric metrics extraction from GLB/glTF files for Tier-2
//! mesh/character/animation assets.

use speccade_cli::analysis::mesh::{analyze_glb, metrics_to_btree};
use speccade_spec::OutputMetrics;

/// Create a minimal valid GLB file with a single triangle for testing.
///
/// This creates a GLB with:
/// - 1 mesh with 1 primitive
/// - 3 vertices forming a triangle
/// - 3 indices
fn create_minimal_glb() -> Vec<u8> {
    // A minimal GLB file containing a single triangle
    // This is a valid glTF 2.0 binary format

    // JSON chunk content (minimal scene with one mesh)
    let json = r#"{
        "asset": {"version": "2.0", "generator": "speccade-test"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{
            "primitives": [{
                "attributes": {"POSITION": 0},
                "indices": 1
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": 3,
                "type": "VEC3",
                "max": [1.0, 1.0, 0.0],
                "min": [0.0, 0.0, 0.0]
            },
            {
                "bufferView": 1,
                "componentType": 5123,
                "count": 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 36},
            {"buffer": 0, "byteOffset": 36, "byteLength": 6}
        ],
        "buffers": [{"byteLength": 44}]
    }"#;

    // Pad JSON to 4-byte alignment
    let json_bytes = json.as_bytes();
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    // Binary data: 3 vertices (3 * 3 * 4 = 36 bytes) + 3 indices (3 * 2 = 6 bytes)
    // Pad to 4-byte alignment
    let mut bin_data = Vec::new();
    // Vertex 0: (0, 0, 0)
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Vertex 1: (1, 0, 0)
    bin_data.extend_from_slice(&1.0f32.to_le_bytes());
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Vertex 2: (0, 1, 0)
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    bin_data.extend_from_slice(&1.0f32.to_le_bytes());
    bin_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Indices: 0, 1, 2
    bin_data.extend_from_slice(&0u16.to_le_bytes());
    bin_data.extend_from_slice(&1u16.to_le_bytes());
    bin_data.extend_from_slice(&2u16.to_le_bytes());

    // Pad binary data
    let bin_padding = (4 - (bin_data.len() % 4)) % 4;
    for _ in 0..bin_padding {
        bin_data.push(0);
    }
    let bin_chunk_length = bin_data.len();

    // Build GLB file
    let mut glb = Vec::new();

    // GLB header (12 bytes)
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    let total_length = 12 + 8 + json_chunk_length + 8 + bin_chunk_length;
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // length

    // JSON chunk header (8 bytes)
    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(b"JSON"); // chunk type

    // JSON chunk data
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(0x20); // space padding for JSON
    }

    // BIN chunk header (8 bytes)
    glb.extend_from_slice(&(bin_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(b"BIN\0"); // chunk type

    // BIN chunk data
    glb.extend_from_slice(&bin_data);

    glb
}

#[test]
fn test_analyze_minimal_glb() {
    let glb_data = create_minimal_glb();
    let metrics = analyze_glb(&glb_data).expect("Failed to analyze minimal GLB");

    // Check format metadata
    assert_eq!(metrics.format.format, "glb");
    assert_eq!(metrics.format.gltf_version, "2.0");
    assert_eq!(metrics.format.mesh_count, 1);
    assert_eq!(metrics.format.node_count, 1);

    // Check topology metrics
    assert_eq!(metrics.topology.vertex_count, 3);
    assert_eq!(metrics.topology.triangle_count, 1);
    assert_eq!(metrics.topology.face_count, 1);
    assert_eq!(metrics.topology.quad_count, 0);
    assert_eq!(metrics.topology.quad_percentage, 0.0);

    // Check manifold metrics
    // Single triangle is not manifold (boundary edges)
    assert_eq!(metrics.manifold.non_manifold_edge_count, 3); // All edges are boundary
    assert_eq!(metrics.manifold.degenerate_face_count, 0);

    // Check bounds
    assert_eq!(metrics.bounds.bounds_min, [0.0, 0.0, 0.0]);
    assert_eq!(metrics.bounds.bounds_max, [1.0, 1.0, 0.0]);
    assert_eq!(metrics.bounds.size, [1.0, 1.0, 0.0]);

    // No skeleton or animation
    assert!(metrics.skeleton.is_none());
    assert!(metrics.animation.is_none());
}

#[test]
fn test_metrics_to_btree_deterministic() {
    let glb_data = create_minimal_glb();

    let metrics1 = analyze_glb(&glb_data).unwrap();
    let metrics2 = analyze_glb(&glb_data).unwrap();

    let btree1 = metrics_to_btree(&metrics1);
    let btree2 = metrics_to_btree(&metrics2);

    let json1 = serde_json::to_string(&btree1).unwrap();
    let json2 = serde_json::to_string(&btree2).unwrap();

    assert_eq!(json1, json2, "Mesh metrics should be deterministic");
}

#[test]
fn test_btree_has_expected_keys() {
    let glb_data = create_minimal_glb();
    let metrics = analyze_glb(&glb_data).unwrap();
    let btree = metrics_to_btree(&metrics);

    // Check all expected top-level keys are present
    assert!(btree.contains_key("format"), "Missing 'format' key");
    assert!(btree.contains_key("topology"), "Missing 'topology' key");
    assert!(btree.contains_key("manifold"), "Missing 'manifold' key");
    assert!(btree.contains_key("uv"), "Missing 'uv' key");
    assert!(btree.contains_key("bounds"), "Missing 'bounds' key");
    assert!(btree.contains_key("materials"), "Missing 'materials' key");

    // Keys should be alphabetically sorted
    let keys: Vec<_> = btree.keys().cloned().collect();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys, "Keys should be alphabetically sorted");
}

#[test]
fn test_output_metrics_new_fields() {
    // Test that OutputMetrics has all the new MESHVER-001 fields
    let metrics = OutputMetrics::new()
        .with_vertex_count(100)
        .with_face_count(50)
        .with_edge_count(150)
        .with_triangle_count(100)
        .with_quad_count(0)
        .with_quad_percentage(0.0)
        .with_manifold(true)
        .with_non_manifold_edge_count(0)
        .with_degenerate_face_count(0)
        .with_uv_coverage(0.85)
        .with_uv_overlap_percentage(2.5)
        .with_bounds_min([-1.0, -1.0, -1.0])
        .with_bounds_max([1.0, 1.0, 1.0]);

    assert_eq!(metrics.vertex_count, Some(100));
    assert_eq!(metrics.face_count, Some(50));
    assert_eq!(metrics.edge_count, Some(150));
    assert_eq!(metrics.triangle_count, Some(100));
    assert_eq!(metrics.quad_count, Some(0));
    assert_eq!(metrics.quad_percentage, Some(0.0));
    assert_eq!(metrics.manifold, Some(true));
    assert_eq!(metrics.non_manifold_edge_count, Some(0));
    assert_eq!(metrics.degenerate_face_count, Some(0));
    assert_eq!(metrics.uv_coverage, Some(0.85));
    assert_eq!(metrics.uv_overlap_percentage, Some(2.5));
    assert_eq!(metrics.bounds_min, Some([-1.0, -1.0, -1.0]));
    assert_eq!(metrics.bounds_max, Some([1.0, 1.0, 1.0]));
}

#[test]
fn test_output_metrics_serialization() {
    let metrics = OutputMetrics::new()
        .with_vertex_count(100)
        .with_triangle_count(50)
        .with_manifold(true)
        .with_bounds_min([0.0, 0.0, 0.0])
        .with_bounds_max([1.0, 1.0, 1.0]);

    // Serialize and deserialize
    let json = serde_json::to_string(&metrics).unwrap();
    let parsed: OutputMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.vertex_count, Some(100));
    assert_eq!(parsed.triangle_count, Some(50));
    assert_eq!(parsed.manifold, Some(true));
    assert_eq!(parsed.bounds_min, Some([0.0, 0.0, 0.0]));
    assert_eq!(parsed.bounds_max, Some([1.0, 1.0, 1.0]));
}

#[test]
fn test_invalid_glb_returns_error() {
    let invalid_data = b"not a valid GLB file";
    let result = analyze_glb(invalid_data);
    assert!(result.is_err(), "Should fail on invalid GLB data");
}

#[test]
fn test_empty_glb_returns_error() {
    let result = analyze_glb(&[]);
    assert!(result.is_err(), "Should fail on empty data");
}

// =============================================================================
// LOD Chain Tests (MESH-004)
// =============================================================================

/// Test that a spec with LOD chain parses correctly.
#[test]
fn test_lod_chain_spec_parses() {
    use speccade_spec::recipe::mesh::{
        LodDecimateMethod, StaticMeshBlenderPrimitivesV1Params,
    };

    let json = r#"{
        "base_primitive": "ico_sphere",
        "dimensions": [1.0, 1.0, 1.0],
        "lod_chain": {
            "levels": [
                { "level": 0, "target_tris": null },
                { "level": 1, "target_tris": 500 },
                { "level": 2, "target_tris": 100 }
            ],
            "decimate_method": "collapse"
        }
    }"#;

    let params: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(json).unwrap();

    assert!(params.lod_chain.is_some());
    let lod_chain = params.lod_chain.unwrap();
    assert_eq!(lod_chain.levels.len(), 3);
    assert_eq!(lod_chain.levels[0].level, 0);
    assert_eq!(lod_chain.levels[0].target_tris, None);
    assert_eq!(lod_chain.levels[1].level, 1);
    assert_eq!(lod_chain.levels[1].target_tris, Some(500));
    assert_eq!(lod_chain.levels[2].level, 2);
    assert_eq!(lod_chain.levels[2].target_tris, Some(100));
    assert_eq!(lod_chain.decimate_method, LodDecimateMethod::Collapse);
}

/// Test that LOD chain with planar decimate parses correctly.
#[test]
fn test_lod_chain_planar_decimate() {
    use speccade_spec::recipe::mesh::{LodChainSettings, LodDecimateMethod};

    let json = r#"{
        "levels": [{ "level": 0 }, { "level": 1, "target_tris": 200 }],
        "decimate_method": "planar"
    }"#;

    let chain: LodChainSettings = serde_json::from_str(json).unwrap();
    assert_eq!(chain.decimate_method, LodDecimateMethod::Planar);
}

/// Test that LOD chain defaults decimate_method to collapse.
#[test]
fn test_lod_chain_default_decimate_method() {
    use speccade_spec::recipe::mesh::{LodChainSettings, LodDecimateMethod};

    let json = r#"{
        "levels": [{ "level": 0 }]
    }"#;

    let chain: LodChainSettings = serde_json::from_str(json).unwrap();
    assert_eq!(chain.decimate_method, LodDecimateMethod::Collapse);
}

/// Test that a complete spec with LOD chain validates.
#[test]
fn test_lod_chain_full_spec_validates() {
    use speccade_spec::validation::validate_spec;
    use speccade_spec::Spec;

    let spec_json = r#"{
        "spec_version": 1,
        "asset_id": "mesh-with-lods",
        "asset_type": "static_mesh",
        "license": "CC0-1.0",
        "seed": 12345,
        "outputs": [
            { "kind": "primary", "format": "glb", "path": "mesh_with_lods.glb" }
        ],
        "recipe": {
            "kind": "static_mesh.blender_primitives_v1",
            "params": {
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0],
                "modifiers": [
                    { "type": "subdivision", "levels": 2, "render_levels": 2 }
                ],
                "lod_chain": {
                    "levels": [
                        { "level": 0, "target_tris": null },
                        { "level": 1, "target_tris": 500 },
                        { "level": 2, "target_tris": 100 }
                    ]
                }
            }
        }
    }"#;

    let spec: Spec = serde_json::from_str(spec_json).unwrap();
    let result = validate_spec(&spec);

    assert!(
        result.errors.is_empty(),
        "LOD chain spec should validate without errors: {:?}",
        result.errors
    );
}
