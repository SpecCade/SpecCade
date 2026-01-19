//! Tests for constraint types and evaluation.

use crate::report::OutputMetrics;

use super::{evaluate_constraints, Constraint, ConstraintResult, ConstraintSet, VerifyResult};

#[test]
fn test_constraint_display() {
    assert_eq!(
        Constraint::MaxVertexCount { value: 1000 }.to_string(),
        "max_vertex_count(1000)"
    );
    assert_eq!(
        Constraint::MaxFaceCount { value: 500 }.to_string(),
        "max_face_count(500)"
    );
    assert_eq!(
        Constraint::MinQuadPercentage { value: 80.0 }.to_string(),
        "min_quad_percentage(80)"
    );
    assert_eq!(Constraint::RequireManifold.to_string(), "require_manifold");
    assert_eq!(
        Constraint::MaxNonManifoldEdges { value: 0 }.to_string(),
        "max_non_manifold_edges(0)"
    );
    assert_eq!(
        Constraint::UvCoverageMin { value: 0.5 }.to_string(),
        "uv_coverage_min(0.5)"
    );
    assert_eq!(
        Constraint::UvOverlapMax { value: 5.0 }.to_string(),
        "uv_overlap_max(5)"
    );
    assert_eq!(
        Constraint::MaxDegenerateFaces { value: 10 }.to_string(),
        "max_degenerate_faces(10)"
    );
}

#[test]
fn test_constraint_serialization() {
    let constraints = ConstraintSet::from_constraints(vec![
        Constraint::MaxVertexCount { value: 1000 },
        Constraint::RequireManifold,
    ]);

    let json = constraints.to_json_pretty().unwrap();
    assert!(json.contains("\"type\": \"max_vertex_count\""));
    assert!(json.contains("\"type\": \"require_manifold\""));

    let parsed = ConstraintSet::from_json(&json).unwrap();
    assert_eq!(parsed, constraints);
}

#[test]
fn test_max_vertex_count_pass() {
    let metrics = OutputMetrics::new().with_vertex_count(500);
    let constraint = Constraint::MaxVertexCount { value: 1000 };
    let result = constraint.evaluate(&metrics);

    assert!(result.passed);
    assert_eq!(result.actual, Some(serde_json::json!(500)));
    assert!(result.message.is_none());
}

#[test]
fn test_max_vertex_count_fail() {
    let metrics = OutputMetrics::new().with_vertex_count(1500);
    let constraint = Constraint::MaxVertexCount { value: 1000 };
    let result = constraint.evaluate(&metrics);

    assert!(!result.passed);
    assert_eq!(result.actual, Some(serde_json::json!(1500)));
    assert!(result.message.is_some());
    assert!(result.message.unwrap().contains("exceeds maximum"));
}

#[test]
fn test_max_vertex_count_skipped() {
    let metrics = OutputMetrics::new();
    let constraint = Constraint::MaxVertexCount { value: 1000 };
    let result = constraint.evaluate(&metrics);

    assert!(result.passed); // Skipped constraints pass
    assert!(result.actual.is_none());
    assert!(result.message.is_some());
    assert!(result.message.unwrap().contains("not available"));
}

#[test]
fn test_require_manifold_pass() {
    let metrics = OutputMetrics::new().with_manifold(true);
    let constraint = Constraint::RequireManifold;
    let result = constraint.evaluate(&metrics);

    assert!(result.passed);
    assert_eq!(result.actual, Some(serde_json::json!(true)));
}

#[test]
fn test_require_manifold_fail() {
    let metrics = OutputMetrics::new().with_manifold(false);
    let constraint = Constraint::RequireManifold;
    let result = constraint.evaluate(&metrics);

    assert!(!result.passed);
    assert_eq!(result.actual, Some(serde_json::json!(false)));
    assert!(result.message.unwrap().contains("not manifold"));
}

#[test]
fn test_min_quad_percentage_pass() {
    let metrics = OutputMetrics::new().with_quad_percentage(90.0);
    let constraint = Constraint::MinQuadPercentage { value: 80.0 };
    let result = constraint.evaluate(&metrics);

    assert!(result.passed);
}

#[test]
fn test_min_quad_percentage_fail() {
    let metrics = OutputMetrics::new().with_quad_percentage(70.0);
    let constraint = Constraint::MinQuadPercentage { value: 80.0 };
    let result = constraint.evaluate(&metrics);

    assert!(!result.passed);
    assert!(result.message.unwrap().contains("below minimum"));
}

#[test]
fn test_uv_coverage_min_pass() {
    let metrics = OutputMetrics::new().with_uv_coverage(0.75);
    let constraint = Constraint::UvCoverageMin { value: 0.5 };
    let result = constraint.evaluate(&metrics);

    assert!(result.passed);
}

#[test]
fn test_uv_overlap_max_pass() {
    let metrics = OutputMetrics::new().with_uv_overlap_percentage(2.0);
    let constraint = Constraint::UvOverlapMax { value: 5.0 };
    let result = constraint.evaluate(&metrics);

    assert!(result.passed);
}

#[test]
fn test_evaluate_constraints_all_pass() {
    let metrics = OutputMetrics::new()
        .with_vertex_count(500)
        .with_face_count(250)
        .with_manifold(true);

    let constraints = ConstraintSet::from_constraints(vec![
        Constraint::MaxVertexCount { value: 1000 },
        Constraint::MaxFaceCount { value: 500 },
        Constraint::RequireManifold,
    ]);

    let result = evaluate_constraints("test-asset", &metrics, &constraints);

    assert!(result.overall_pass);
    assert_eq!(result.results.len(), 3);
    assert!(result.results.iter().all(|r| r.passed));
}

#[test]
fn test_evaluate_constraints_some_fail() {
    let metrics = OutputMetrics::new()
        .with_vertex_count(1500)
        .with_face_count(250)
        .with_manifold(false);

    let constraints = ConstraintSet::from_constraints(vec![
        Constraint::MaxVertexCount { value: 1000 },
        Constraint::MaxFaceCount { value: 500 },
        Constraint::RequireManifold,
    ]);

    let result = evaluate_constraints("test-asset", &metrics, &constraints);

    assert!(!result.overall_pass);
    assert_eq!(result.results.len(), 3);
    assert!(!result.results[0].passed); // vertex count failed
    assert!(result.results[1].passed); // face count passed
    assert!(!result.results[2].passed); // manifold failed
}

#[test]
fn test_verify_result_serialization() {
    let result = VerifyResult::new(
        "test-asset".to_string(),
        vec![ConstraintResult::pass(
            &Constraint::MaxVertexCount { value: 1000 },
            Some(serde_json::json!(500)),
        )],
    );

    let json = result.to_json_pretty().unwrap();
    assert!(json.contains("\"asset_id\": \"test-asset\""));
    assert!(json.contains("\"overall_pass\": true"));
}

#[test]
fn test_constraint_set_builder() {
    let mut set = ConstraintSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);

    set.add(Constraint::MaxVertexCount { value: 1000 });
    set.add(Constraint::RequireManifold);

    assert!(!set.is_empty());
    assert_eq!(set.len(), 2);
}

#[test]
fn test_max_triangle_count() {
    let metrics = OutputMetrics::new().with_triangle_count(800);
    let constraint = Constraint::MaxTriangleCount { value: 1000 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);

    let metrics2 = OutputMetrics::new().with_triangle_count(1200);
    let result2 = constraint.evaluate(&metrics2);
    assert!(!result2.passed);
}

#[test]
fn test_max_bone_count() {
    let metrics = OutputMetrics::new().with_bone_count(64);
    let constraint = Constraint::MaxBoneCount { value: 128 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);

    let metrics2 = OutputMetrics::new().with_bone_count(256);
    let result2 = constraint.evaluate(&metrics2);
    assert!(!result2.passed);
}

#[test]
fn test_max_degenerate_faces() {
    let metrics = OutputMetrics::new().with_degenerate_face_count(0);
    let constraint = Constraint::MaxDegenerateFaces { value: 0 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);

    let metrics2 = OutputMetrics::new().with_degenerate_face_count(5);
    let result2 = constraint.evaluate(&metrics2);
    assert!(!result2.passed);
}

#[test]
fn test_max_non_manifold_edges() {
    let metrics = OutputMetrics::new().with_non_manifold_edge_count(0);
    let constraint = Constraint::MaxNonManifoldEdges { value: 0 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);

    let metrics2 = OutputMetrics::new().with_non_manifold_edge_count(3);
    let result2 = constraint.evaluate(&metrics2);
    assert!(!result2.passed);
}

// ========== Skinning constraint tests (MESHVER-004) ==========

#[test]
fn test_max_bone_influences_pass() {
    let metrics = OutputMetrics::new().with_max_bone_influences(4);
    let constraint = Constraint::MaxBoneInfluences { value: 4 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);
    assert_eq!(result.actual, Some(serde_json::json!(4)));
}

#[test]
fn test_max_bone_influences_fail() {
    let metrics = OutputMetrics::new().with_max_bone_influences(8);
    let constraint = Constraint::MaxBoneInfluences { value: 4 };
    let result = constraint.evaluate(&metrics);
    assert!(!result.passed);
    assert!(result.message.unwrap().contains("exceeds maximum"));
}

#[test]
fn test_max_bone_influences_skipped() {
    let metrics = OutputMetrics::new();
    let constraint = Constraint::MaxBoneInfluences { value: 4 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed); // Skipped constraints pass
    assert!(result.message.unwrap().contains("not available"));
}

#[test]
fn test_max_unweighted_vertices_pass() {
    let metrics = OutputMetrics::new().with_unweighted_vertex_count(0);
    let constraint = Constraint::MaxUnweightedVertices { value: 0 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);
}

#[test]
fn test_max_unweighted_vertices_fail() {
    let metrics = OutputMetrics::new().with_unweighted_vertex_count(50);
    let constraint = Constraint::MaxUnweightedVertices { value: 0 };
    let result = constraint.evaluate(&metrics);
    assert!(!result.passed);
    assert!(result.message.unwrap().contains("exceeds maximum"));
}

#[test]
fn test_max_unweighted_vertices_tolerant() {
    // Some workflows allow a small number of unweighted vertices
    let metrics = OutputMetrics::new().with_unweighted_vertex_count(5);
    let constraint = Constraint::MaxUnweightedVertices { value: 10 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);
}

#[test]
fn test_min_weight_normalization_pass() {
    let metrics = OutputMetrics::new().with_weight_normalization_percentage(100.0);
    let constraint = Constraint::MinWeightNormalization { value: 99.0 };
    let result = constraint.evaluate(&metrics);
    assert!(result.passed);
}

#[test]
fn test_min_weight_normalization_fail() {
    let metrics = OutputMetrics::new().with_weight_normalization_percentage(95.0);
    let constraint = Constraint::MinWeightNormalization { value: 99.0 };
    let result = constraint.evaluate(&metrics);
    assert!(!result.passed);
    assert!(result.message.unwrap().contains("below minimum"));
}

#[test]
fn test_skinning_constraint_display() {
    assert_eq!(
        Constraint::MaxBoneInfluences { value: 4 }.to_string(),
        "max_bone_influences(4)"
    );
    assert_eq!(
        Constraint::MaxUnweightedVertices { value: 0 }.to_string(),
        "max_unweighted_vertices(0)"
    );
    assert_eq!(
        Constraint::MinWeightNormalization { value: 99.0 }.to_string(),
        "min_weight_normalization(99)"
    );
}

#[test]
fn test_skinning_constraint_serialization() {
    let constraints = ConstraintSet::from_constraints(vec![
        Constraint::MaxBoneInfluences { value: 4 },
        Constraint::MaxUnweightedVertices { value: 0 },
        Constraint::MinWeightNormalization { value: 99.0 },
    ]);

    let json = constraints.to_json_pretty().unwrap();
    assert!(json.contains("\"type\": \"max_bone_influences\""));
    assert!(json.contains("\"type\": \"max_unweighted_vertices\""));
    assert!(json.contains("\"type\": \"min_weight_normalization\""));

    let parsed = ConstraintSet::from_json(&json).unwrap();
    assert_eq!(parsed, constraints);
}

#[test]
fn test_evaluate_skeletal_mesh_constraints() {
    let metrics = OutputMetrics::new()
        .with_bone_count(64)
        .with_max_bone_influences(4)
        .with_unweighted_vertex_count(0)
        .with_weight_normalization_percentage(100.0)
        .with_vertex_count(5000);

    let constraints = ConstraintSet::from_constraints(vec![
        Constraint::MaxBoneCount { value: 128 },
        Constraint::MaxBoneInfluences { value: 4 },
        Constraint::MaxUnweightedVertices { value: 0 },
        Constraint::MinWeightNormalization { value: 99.0 },
        Constraint::MaxVertexCount { value: 10000 },
    ]);

    let result = evaluate_constraints("skeletal-test", &metrics, &constraints);
    assert!(result.overall_pass);
    assert_eq!(result.results.len(), 5);
    assert!(result.results.iter().all(|r| r.passed));
}
