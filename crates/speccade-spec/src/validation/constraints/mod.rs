//! Constraint types for post-generate asset verification.
//!
//! Constraints allow validating generated assets against user-defined limits
//! using metrics from the generation report.

mod evaluate;
mod types;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use evaluate::evaluate_constraints;
pub use types::{ConstraintResult, VerifyResult};

/// A constraint that can be evaluated against report metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Constraint {
    /// Maximum allowed vertex count.
    MaxVertexCount {
        /// The maximum number of vertices allowed.
        value: u32,
    },
    /// Maximum allowed face count.
    MaxFaceCount {
        /// The maximum number of faces allowed.
        value: u32,
    },
    /// Minimum percentage of faces that must be quads (0.0-100.0).
    MinQuadPercentage {
        /// The minimum quad percentage required.
        value: f64,
    },
    /// Requires the mesh to be manifold (watertight).
    RequireManifold,
    /// Maximum number of non-manifold edges allowed.
    MaxNonManifoldEdges {
        /// The maximum number of non-manifold edges allowed.
        value: u32,
    },
    /// Minimum UV coverage ratio (0.0-1.0).
    UvCoverageMin {
        /// The minimum UV coverage ratio required.
        value: f64,
    },
    /// Maximum UV overlap percentage (0.0-100.0).
    UvOverlapMax {
        /// The maximum UV overlap percentage allowed.
        value: f64,
    },
    /// Maximum number of degenerate faces allowed.
    MaxDegenerateFaces {
        /// The maximum number of degenerate faces allowed.
        value: u32,
    },
    /// Maximum triangle count.
    MaxTriangleCount {
        /// The maximum number of triangles allowed.
        value: u32,
    },
    /// Maximum bone count for skeletal meshes.
    MaxBoneCount {
        /// The maximum number of bones allowed.
        value: u32,
    },
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::MaxVertexCount { value } => write!(f, "max_vertex_count({})", value),
            Constraint::MaxFaceCount { value } => write!(f, "max_face_count({})", value),
            Constraint::MinQuadPercentage { value } => write!(f, "min_quad_percentage({})", value),
            Constraint::RequireManifold => write!(f, "require_manifold"),
            Constraint::MaxNonManifoldEdges { value } => {
                write!(f, "max_non_manifold_edges({})", value)
            }
            Constraint::UvCoverageMin { value } => write!(f, "uv_coverage_min({})", value),
            Constraint::UvOverlapMax { value } => write!(f, "uv_overlap_max({})", value),
            Constraint::MaxDegenerateFaces { value } => {
                write!(f, "max_degenerate_faces({})", value)
            }
            Constraint::MaxTriangleCount { value } => write!(f, "max_triangle_count({})", value),
            Constraint::MaxBoneCount { value } => write!(f, "max_bone_count({})", value),
        }
    }
}

/// A set of constraints loaded from a constraints file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintSet {
    /// The list of constraints to evaluate.
    pub constraints: Vec<Constraint>,
}

impl ConstraintSet {
    /// Creates a new empty constraint set.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Creates a constraint set from a list of constraints.
    pub fn from_constraints(constraints: Vec<Constraint>) -> Self {
        Self { constraints }
    }

    /// Loads a constraint set from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes the constraint set to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the constraint set to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Adds a constraint to the set.
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Returns the number of constraints in the set.
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Returns whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self::new()
    }
}
