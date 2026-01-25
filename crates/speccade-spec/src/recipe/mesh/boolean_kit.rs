//! Boolean kitbashing mesh recipe definitions.
//!
//! This module provides the `static_mesh.boolean_kit_v1` recipe for generating
//! hard-surface meshes using boolean operations (union, difference, intersect)
//! with cleanup for artifact removal using Blender as a Tier 2 backend.

use serde::{Deserialize, Serialize};

use super::common::MeshExportSettings;

/// Maximum number of boolean operations allowed.
pub const MAX_BOOLEAN_OPERATIONS: usize = 50;

/// Default merge distance for cleanup operations.
pub const DEFAULT_MERGE_DISTANCE: f64 = 0.001;

/// Parameters for the `static_mesh.boolean_kit_v1` recipe.
///
/// This recipe combines multiple mesh primitives or referenced meshes using
/// boolean operations (union, difference, intersect) to create hard-surface
/// models suitable for vehicles, buildings, mechanical parts, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMeshBooleanKitV1Params {
    /// The base mesh to apply operations to.
    /// Can be a primitive name (e.g., "cube", "cylinder") or a reference
    /// to another mesh asset.
    pub base: MeshSource,
    /// Boolean operations to apply in order.
    /// Operations are applied sequentially for deterministic results.
    pub operations: Vec<BooleanOperation>,
    /// Boolean solver to use.
    #[serde(default, skip_serializing_if = "BooleanSolver::is_default")]
    pub solver: BooleanSolver,
    /// Post-boolean cleanup settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<BooleanCleanup>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
}

/// Source for a mesh in boolean operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MeshSource {
    /// A primitive mesh with optional transform.
    Primitive(PrimitiveMesh),
    /// Reference to an external mesh asset by asset_id.
    Reference(MeshReference),
}

/// A primitive mesh with optional transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveMesh {
    /// The primitive type (cube, sphere, cylinder, cone, torus, plane, ico_sphere).
    pub primitive: String,
    /// Dimensions [X, Y, Z] in Blender units.
    #[serde(default = "default_dimensions")]
    pub dimensions: [f64; 3],
    /// Position offset [X, Y, Z].
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub position: [f64; 3],
    /// Rotation in degrees [X, Y, Z].
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub rotation: [f64; 3],
    /// Uniform scale factor.
    #[serde(default = "default_one", skip_serializing_if = "is_default_one")]
    pub scale: f64,
}

fn default_dimensions() -> [f64; 3] {
    [1.0, 1.0, 1.0]
}

fn default_one() -> f64 {
    1.0
}

fn is_default_one(v: &f64) -> bool {
    (*v - 1.0).abs() < f64::EPSILON
}

fn is_zero_array(arr: &[f64; 3]) -> bool {
    arr.iter().all(|v| v.abs() < f64::EPSILON)
}

/// Reference to an external mesh asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeshReference {
    /// Asset ID of the referenced mesh.
    pub asset_ref: String,
    /// Position offset [X, Y, Z].
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub position: [f64; 3],
    /// Rotation in degrees [X, Y, Z].
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub rotation: [f64; 3],
    /// Uniform scale factor.
    #[serde(default = "default_one", skip_serializing_if = "is_default_one")]
    pub scale: f64,
}

/// A boolean operation to apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanOperation {
    /// The type of boolean operation.
    pub op: BooleanOperationType,
    /// The target mesh to combine with.
    pub target: MeshSource,
}

/// Type of boolean operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOperationType {
    /// Union: combine meshes, keeping all geometry.
    Union,
    /// Difference: subtract target from base.
    Difference,
    /// Intersect: keep only overlapping geometry.
    Intersect,
}

/// Boolean solver selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanSolver {
    /// Exact solver: slower but more reliable for complex geometry.
    #[default]
    Exact,
    /// Fast solver: quicker but may fail on complex or non-manifold geometry.
    Fast,
}

impl BooleanSolver {
    /// Returns true if this is the default solver (Exact).
    pub fn is_default(&self) -> bool {
        *self == BooleanSolver::Exact
    }
}

/// Post-boolean cleanup settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanCleanup {
    /// Distance threshold for merging duplicate vertices.
    #[serde(
        default = "default_merge_distance",
        skip_serializing_if = "is_default_merge_distance"
    )]
    pub merge_distance: f64,
    /// Remove duplicate/overlapping vertices.
    #[serde(default = "default_true")]
    pub remove_doubles: bool,
    /// Recalculate normals to face outward.
    #[serde(default = "default_true")]
    pub recalc_normals: bool,
    /// Fill holes in the mesh.
    #[serde(default)]
    pub fill_holes: bool,
    /// Dissolve degenerate geometry.
    #[serde(default = "default_true")]
    pub dissolve_degenerate: bool,
}

fn default_merge_distance() -> f64 {
    DEFAULT_MERGE_DISTANCE
}

fn is_default_merge_distance(v: &f64) -> bool {
    (*v - DEFAULT_MERGE_DISTANCE).abs() < f64::EPSILON
}

fn default_true() -> bool {
    true
}

impl Default for BooleanCleanup {
    fn default() -> Self {
        Self {
            merge_distance: DEFAULT_MERGE_DISTANCE,
            remove_doubles: true,
            recalc_normals: true,
            fill_holes: false,
            dissolve_degenerate: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BooleanOperationType Tests
    // ========================================================================

    #[test]
    fn test_boolean_operation_type_union() {
        let op = BooleanOperationType::Union;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"union\"");

        let parsed: BooleanOperationType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BooleanOperationType::Union);
    }

    #[test]
    fn test_boolean_operation_type_difference() {
        let op = BooleanOperationType::Difference;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"difference\"");

        let parsed: BooleanOperationType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BooleanOperationType::Difference);
    }

    #[test]
    fn test_boolean_operation_type_intersect() {
        let op = BooleanOperationType::Intersect;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"intersect\"");

        let parsed: BooleanOperationType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BooleanOperationType::Intersect);
    }

    // ========================================================================
    // BooleanSolver Tests
    // ========================================================================

    #[test]
    fn test_boolean_solver_exact() {
        let solver = BooleanSolver::Exact;
        let json = serde_json::to_string(&solver).unwrap();
        assert_eq!(json, "\"exact\"");

        let parsed: BooleanSolver = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BooleanSolver::Exact);
    }

    #[test]
    fn test_boolean_solver_fast() {
        let solver = BooleanSolver::Fast;
        let json = serde_json::to_string(&solver).unwrap();
        assert_eq!(json, "\"fast\"");

        let parsed: BooleanSolver = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BooleanSolver::Fast);
    }

    #[test]
    fn test_boolean_solver_default() {
        let solver = BooleanSolver::default();
        assert_eq!(solver, BooleanSolver::Exact);
    }

    // ========================================================================
    // BooleanCleanup Tests
    // ========================================================================

    #[test]
    fn test_boolean_cleanup_default() {
        let cleanup = BooleanCleanup::default();
        assert_eq!(cleanup.merge_distance, DEFAULT_MERGE_DISTANCE);
        assert!(cleanup.remove_doubles);
        assert!(cleanup.recalc_normals);
        assert!(!cleanup.fill_holes);
        assert!(cleanup.dissolve_degenerate);
    }

    #[test]
    fn test_boolean_cleanup_serialization() {
        let cleanup = BooleanCleanup::default();
        let json = serde_json::to_string(&cleanup).unwrap();
        // Default values should be omitted
        assert!(!json.contains("merge_distance"));
        assert!(json.contains("\"remove_doubles\":true"));
        assert!(json.contains("\"recalc_normals\":true"));

        let parsed: BooleanCleanup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cleanup);
    }

    #[test]
    fn test_boolean_cleanup_custom() {
        let cleanup = BooleanCleanup {
            merge_distance: 0.01,
            remove_doubles: true,
            recalc_normals: false,
            fill_holes: true,
            dissolve_degenerate: false,
        };

        let json = serde_json::to_string(&cleanup).unwrap();
        assert!(json.contains("\"merge_distance\":0.01"));
        assert!(json.contains("\"fill_holes\":true"));
        assert!(json.contains("\"recalc_normals\":false"));

        let parsed: BooleanCleanup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.merge_distance, 0.01);
        assert!(parsed.fill_holes);
        assert!(!parsed.recalc_normals);
    }

    #[test]
    fn test_boolean_cleanup_from_json_defaults() {
        let json = r#"{}"#;
        let parsed: BooleanCleanup = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.merge_distance, DEFAULT_MERGE_DISTANCE);
        assert!(parsed.remove_doubles);
        assert!(parsed.recalc_normals);
    }

    #[test]
    fn test_boolean_cleanup_rejects_unknown_fields() {
        let json = r#"{"remove_doubles": true, "unknown_field": 123}"#;
        let result: Result<BooleanCleanup, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // PrimitiveMesh Tests
    // ========================================================================

    #[test]
    fn test_primitive_mesh_basic() {
        let mesh = PrimitiveMesh {
            primitive: "cube".to_string(),
            dimensions: [1.0, 1.0, 1.0],
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        };

        let json = serde_json::to_string(&mesh).unwrap();
        assert!(json.contains("\"primitive\":\"cube\""));
        assert!(json.contains("\"dimensions\":[1.0,1.0,1.0]"));
        // Default position/rotation/scale should be omitted
        assert!(!json.contains("position"));
        assert!(!json.contains("rotation"));
        assert!(!json.contains("scale"));

        let parsed: PrimitiveMesh = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.primitive, "cube");
    }

    #[test]
    fn test_primitive_mesh_with_transform() {
        let mesh = PrimitiveMesh {
            primitive: "cylinder".to_string(),
            dimensions: [0.5, 0.5, 2.0],
            position: [1.0, 0.0, 0.0],
            rotation: [90.0, 0.0, 0.0],
            scale: 2.0,
        };

        let json = serde_json::to_string(&mesh).unwrap();
        assert!(json.contains("\"position\":[1.0,0.0,0.0]"));
        assert!(json.contains("\"rotation\":[90.0,0.0,0.0]"));
        assert!(json.contains("\"scale\":2.0"));

        let parsed: PrimitiveMesh = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.position, [1.0, 0.0, 0.0]);
        assert_eq!(parsed.rotation, [90.0, 0.0, 0.0]);
        assert_eq!(parsed.scale, 2.0);
    }

    #[test]
    fn test_primitive_mesh_from_json_defaults() {
        let json = r#"{"primitive": "sphere"}"#;
        let parsed: PrimitiveMesh = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.dimensions, [1.0, 1.0, 1.0]);
        assert_eq!(parsed.position, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.scale, 1.0);
    }

    // ========================================================================
    // MeshReference Tests
    // ========================================================================

    #[test]
    fn test_mesh_reference_basic() {
        let mesh_ref = MeshReference {
            asset_ref: "engine_block".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        };

        let json = serde_json::to_string(&mesh_ref).unwrap();
        assert!(json.contains("\"asset_ref\":\"engine_block\""));
        // Default transforms should be omitted
        assert!(!json.contains("position"));

        let parsed: MeshReference = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.asset_ref, "engine_block");
    }

    #[test]
    fn test_mesh_reference_with_transform() {
        let mesh_ref = MeshReference {
            asset_ref: "wheel".to_string(),
            position: [2.0, 0.0, 0.5],
            rotation: [0.0, 90.0, 0.0],
            scale: 0.5,
        };

        let json = serde_json::to_string(&mesh_ref).unwrap();
        assert!(json.contains("\"position\":[2.0,0.0,0.5]"));

        let parsed: MeshReference = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.scale, 0.5);
    }

    // ========================================================================
    // MeshSource Tests
    // ========================================================================

    #[test]
    fn test_mesh_source_primitive() {
        let source = MeshSource::Primitive(PrimitiveMesh {
            primitive: "cube".to_string(),
            dimensions: [2.0, 2.0, 2.0],
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        });

        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"primitive\":\"cube\""));

        let parsed: MeshSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, MeshSource::Primitive(_)));
    }

    #[test]
    fn test_mesh_source_reference() {
        let source = MeshSource::Reference(MeshReference {
            asset_ref: "hull_part".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        });

        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"asset_ref\":\"hull_part\""));

        let parsed: MeshSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, MeshSource::Reference(_)));
    }

    // ========================================================================
    // BooleanOperation Tests
    // ========================================================================

    #[test]
    fn test_boolean_operation_union() {
        let op = BooleanOperation {
            op: BooleanOperationType::Union,
            target: MeshSource::Primitive(PrimitiveMesh {
                primitive: "sphere".to_string(),
                dimensions: [1.0, 1.0, 1.0],
                position: [0.5, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }),
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"op\":\"union\""));
        assert!(json.contains("\"primitive\":\"sphere\""));

        let parsed: BooleanOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.op, BooleanOperationType::Union);
    }

    #[test]
    fn test_boolean_operation_difference() {
        let op = BooleanOperation {
            op: BooleanOperationType::Difference,
            target: MeshSource::Primitive(PrimitiveMesh {
                primitive: "cylinder".to_string(),
                dimensions: [0.2, 0.2, 2.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }),
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"op\":\"difference\""));

        let parsed: BooleanOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.op, BooleanOperationType::Difference);
    }

    // ========================================================================
    // StaticMeshBooleanKitV1Params Tests
    // ========================================================================

    #[test]
    fn test_boolean_kit_params_basic() {
        let params = StaticMeshBooleanKitV1Params {
            base: MeshSource::Primitive(PrimitiveMesh {
                primitive: "cube".to_string(),
                dimensions: [2.0, 1.0, 1.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }),
            operations: vec![BooleanOperation {
                op: BooleanOperationType::Union,
                target: MeshSource::Primitive(PrimitiveMesh {
                    primitive: "sphere".to_string(),
                    dimensions: [1.0, 1.0, 1.0],
                    position: [1.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: 1.0,
                }),
            }],
            solver: BooleanSolver::Exact,
            cleanup: None,
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"base\""));
        assert!(json.contains("\"operations\""));
        // Default solver should not be serialized
        assert!(!json.contains("solver"));

        let parsed: StaticMeshBooleanKitV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operations.len(), 1);
        assert!(parsed.cleanup.is_none());
    }

    #[test]
    fn test_boolean_kit_params_with_cleanup() {
        let params = StaticMeshBooleanKitV1Params {
            base: MeshSource::Primitive(PrimitiveMesh {
                primitive: "cube".to_string(),
                dimensions: [1.0, 1.0, 1.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }),
            operations: vec![
                BooleanOperation {
                    op: BooleanOperationType::Difference,
                    target: MeshSource::Primitive(PrimitiveMesh {
                        primitive: "cylinder".to_string(),
                        dimensions: [0.3, 0.3, 2.0],
                        position: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0],
                        scale: 1.0,
                    }),
                },
                BooleanOperation {
                    op: BooleanOperationType::Difference,
                    target: MeshSource::Primitive(PrimitiveMesh {
                        primitive: "cylinder".to_string(),
                        dimensions: [0.3, 0.3, 2.0],
                        position: [0.0, 0.0, 0.0],
                        rotation: [90.0, 0.0, 0.0],
                        scale: 1.0,
                    }),
                },
            ],
            solver: BooleanSolver::Fast,
            cleanup: Some(BooleanCleanup {
                merge_distance: 0.001,
                remove_doubles: true,
                recalc_normals: true,
                fill_holes: false,
                dissolve_degenerate: true,
            }),
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"solver\":\"fast\""));
        assert!(json.contains("\"cleanup\""));
        assert!(json.contains("\"remove_doubles\":true"));

        let parsed: StaticMeshBooleanKitV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operations.len(), 2);
        assert!(parsed.cleanup.is_some());
        assert_eq!(parsed.solver, BooleanSolver::Fast);
    }

    #[test]
    fn test_boolean_kit_params_complex() {
        let params = StaticMeshBooleanKitV1Params {
            base: MeshSource::Primitive(PrimitiveMesh {
                primitive: "cube".to_string(),
                dimensions: [3.0, 2.0, 1.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1.0,
            }),
            operations: vec![
                BooleanOperation {
                    op: BooleanOperationType::Union,
                    target: MeshSource::Primitive(PrimitiveMesh {
                        primitive: "cylinder".to_string(),
                        dimensions: [0.5, 0.5, 1.0],
                        position: [1.0, 0.0, 0.5],
                        rotation: [0.0, 0.0, 0.0],
                        scale: 1.0,
                    }),
                },
                BooleanOperation {
                    op: BooleanOperationType::Difference,
                    target: MeshSource::Primitive(PrimitiveMesh {
                        primitive: "cube".to_string(),
                        dimensions: [1.5, 1.0, 0.5],
                        position: [0.0, 0.0, 0.5],
                        rotation: [0.0, 0.0, 0.0],
                        scale: 1.0,
                    }),
                },
                BooleanOperation {
                    op: BooleanOperationType::Intersect,
                    target: MeshSource::Primitive(PrimitiveMesh {
                        primitive: "sphere".to_string(),
                        dimensions: [4.0, 4.0, 4.0],
                        position: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0],
                        scale: 1.0,
                    }),
                },
            ],
            solver: BooleanSolver::Exact,
            cleanup: Some(BooleanCleanup::default()),
            export: Some(MeshExportSettings::default()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: StaticMeshBooleanKitV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.operations.len(), 3);
        assert_eq!(parsed.operations[0].op, BooleanOperationType::Union);
        assert_eq!(parsed.operations[1].op, BooleanOperationType::Difference);
        assert_eq!(parsed.operations[2].op, BooleanOperationType::Intersect);
        assert!(parsed.cleanup.is_some());
        assert!(parsed.export.is_some());
    }

    #[test]
    fn test_boolean_kit_params_from_json() {
        let json = r#"{
            "base": {
                "primitive": "cube",
                "dimensions": [2.0, 1.0, 1.0]
            },
            "operations": [
                {
                    "op": "union",
                    "target": {
                        "primitive": "sphere",
                        "dimensions": [1.0, 1.0, 1.0],
                        "position": [1.0, 0.0, 0.0]
                    }
                },
                {
                    "op": "difference",
                    "target": {
                        "primitive": "cylinder",
                        "dimensions": [0.2, 0.2, 2.0]
                    }
                }
            ],
            "solver": "exact",
            "cleanup": {
                "remove_doubles": true,
                "recalc_normals": true
            }
        }"#;

        let parsed: StaticMeshBooleanKitV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.operations.len(), 2);
        assert_eq!(parsed.solver, BooleanSolver::Exact);
        assert!(parsed.cleanup.is_some());
    }

    #[test]
    fn test_boolean_kit_params_rejects_unknown_fields() {
        let json = r#"{
            "base": {"primitive": "cube", "dimensions": [1.0, 1.0, 1.0]},
            "operations": [],
            "unknown_field": true
        }"#;
        let result: Result<StaticMeshBooleanKitV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
