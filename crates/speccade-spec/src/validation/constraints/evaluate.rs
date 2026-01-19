//! Constraint evaluation logic.

use crate::report::OutputMetrics;

use super::{Constraint, ConstraintResult, ConstraintSet, VerifyResult};

impl Constraint {
    /// Evaluates this constraint against the given metrics.
    pub fn evaluate(&self, metrics: &OutputMetrics) -> ConstraintResult {
        match self {
            Constraint::MaxVertexCount { value } => evaluate_max_u32(
                self,
                metrics.vertex_count,
                *value,
                "vertex count",
                "vertex_count",
            ),
            Constraint::MaxFaceCount { value } => {
                evaluate_max_u32(self, metrics.face_count, *value, "face count", "face_count")
            }
            Constraint::MinQuadPercentage { value } => evaluate_min_f64_percentage(
                self,
                metrics.quad_percentage,
                *value,
                "quad percentage",
                "quad_percentage",
            ),
            Constraint::RequireManifold => {
                if let Some(actual) = metrics.manifold {
                    if actual {
                        ConstraintResult::pass(self, Some(serde_json::json!(actual)))
                    } else {
                        ConstraintResult::fail(
                            self,
                            Some(serde_json::json!(actual)),
                            "mesh is not manifold",
                        )
                    }
                } else {
                    ConstraintResult::skipped(self, "manifold metric not available")
                }
            }
            Constraint::MaxNonManifoldEdges { value } => evaluate_max_u32(
                self,
                metrics.non_manifold_edge_count,
                *value,
                "non-manifold edge count",
                "non_manifold_edge_count",
            ),
            Constraint::UvCoverageMin { value } => evaluate_min_f64_ratio(
                self,
                metrics.uv_coverage,
                *value,
                "UV coverage",
                "uv_coverage",
            ),
            Constraint::UvOverlapMax { value } => evaluate_max_f64_percentage(
                self,
                metrics.uv_overlap_percentage,
                *value,
                "UV overlap",
                "uv_overlap_percentage",
            ),
            Constraint::MaxDegenerateFaces { value } => evaluate_max_u32(
                self,
                metrics.degenerate_face_count,
                *value,
                "degenerate face count",
                "degenerate_face_count",
            ),
            Constraint::MaxTriangleCount { value } => evaluate_max_u32(
                self,
                metrics.triangle_count,
                *value,
                "triangle count",
                "triangle_count",
            ),
            Constraint::MaxBoneCount { value } => {
                evaluate_max_u32(self, metrics.bone_count, *value, "bone count", "bone_count")
            }
        }
    }
}

/// Helper for evaluating maximum u32 constraints.
fn evaluate_max_u32(
    constraint: &Constraint,
    actual: Option<u32>,
    max: u32,
    name: &str,
    metric_name: &str,
) -> ConstraintResult {
    if let Some(actual) = actual {
        if actual <= max {
            ConstraintResult::pass(constraint, Some(serde_json::json!(actual)))
        } else {
            ConstraintResult::fail(
                constraint,
                Some(serde_json::json!(actual)),
                format!("{} {} exceeds maximum {}", name, actual, max),
            )
        }
    } else {
        ConstraintResult::skipped(constraint, format!("{} metric not available", metric_name))
    }
}

/// Helper for evaluating minimum f64 percentage constraints.
fn evaluate_min_f64_percentage(
    constraint: &Constraint,
    actual: Option<f64>,
    min: f64,
    name: &str,
    metric_name: &str,
) -> ConstraintResult {
    if let Some(actual) = actual {
        if actual >= min {
            ConstraintResult::pass(constraint, Some(serde_json::json!(actual)))
        } else {
            ConstraintResult::fail(
                constraint,
                Some(serde_json::json!(actual)),
                format!("{} {:.2}% is below minimum {:.2}%", name, actual, min),
            )
        }
    } else {
        ConstraintResult::skipped(constraint, format!("{} metric not available", metric_name))
    }
}

/// Helper for evaluating minimum f64 ratio constraints.
fn evaluate_min_f64_ratio(
    constraint: &Constraint,
    actual: Option<f64>,
    min: f64,
    name: &str,
    metric_name: &str,
) -> ConstraintResult {
    if let Some(actual) = actual {
        if actual >= min {
            ConstraintResult::pass(constraint, Some(serde_json::json!(actual)))
        } else {
            ConstraintResult::fail(
                constraint,
                Some(serde_json::json!(actual)),
                format!("{} {:.4} is below minimum {:.4}", name, actual, min),
            )
        }
    } else {
        ConstraintResult::skipped(constraint, format!("{} metric not available", metric_name))
    }
}

/// Helper for evaluating maximum f64 percentage constraints.
fn evaluate_max_f64_percentage(
    constraint: &Constraint,
    actual: Option<f64>,
    max: f64,
    name: &str,
    metric_name: &str,
) -> ConstraintResult {
    if let Some(actual) = actual {
        if actual <= max {
            ConstraintResult::pass(constraint, Some(serde_json::json!(actual)))
        } else {
            ConstraintResult::fail(
                constraint,
                Some(serde_json::json!(actual)),
                format!("{} {:.2}% exceeds maximum {:.2}%", name, actual, max),
            )
        }
    } else {
        ConstraintResult::skipped(constraint, format!("{} metric not available", metric_name))
    }
}

/// Evaluates a set of constraints against output metrics from a report.
pub fn evaluate_constraints(
    asset_id: &str,
    metrics: &OutputMetrics,
    constraints: &ConstraintSet,
) -> VerifyResult {
    let results: Vec<ConstraintResult> = constraints
        .constraints
        .iter()
        .map(|c| c.evaluate(metrics))
        .collect();

    VerifyResult::new(asset_id.to_string(), results)
}
