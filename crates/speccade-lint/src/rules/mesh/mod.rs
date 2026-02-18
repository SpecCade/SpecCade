//! Mesh quality lint rules.
//!
//! Rules for detecting perceptual problems in generated mesh assets.
//!
//! Includes 12 rules across three severity levels:
//! - **Error**: Non-manifold edges, degenerate faces, unweighted vertices, inverted normals
//! - **Warning**: Humanoid proportions, UV overlap/stretch, missing material, excessive ngons, isolated vertices
//! - **Info**: High poly count, no UVs

pub(crate) mod parsing;
pub mod quality;
pub mod topology;

#[cfg(test)]
mod tests;

use crate::report::{LintIssue, Severity};
use crate::rules::{AssetData, LintRule};

pub use quality::{
    HumanoidProportionsRule, InvertedNormalsRule, MissingMaterialRule, NoUvsRule,
    UnweightedVertsRule, UvOverlapRule, UvStretchRule,
};
pub use topology::{
    DegenerateFacesRule, ExcessiveNgonsRule, HighPolyRule, IsolatedVertsRule, NonManifoldRule,
};

/// Lint a GLB file and return all issues found
pub fn lint_glb(glb_data: &[u8]) -> Vec<LintIssue> {
    let mut issues = Vec::new();

    // Parse GLB to get mesh data
    let gltf = match gltf::Gltf::from_slice(glb_data) {
        Ok(g) => g,
        Err(e) => {
            issues.push(
                LintIssue::new(
                    "mesh/parse-error",
                    Severity::Error,
                    format!("Failed to parse GLB: {}", e),
                    "Check that the file is a valid GLB format",
                )
                .with_actual_value(format!("parse error: {}", e)),
            );
            return issues;
        }
    };

    // Convert to MeshData for analysis (currently unused, but may be used for
    // future direct mesh analysis without going through AssetData)
    let _mesh = parsing::MeshData::from_gltf(&gltf, Some(glb_data));

    // Run all mesh lint rules through the AssetData interface
    let asset = AssetData {
        path: std::path::Path::new("lint.glb"),
        bytes: glb_data,
    };

    // Run all registered rules
    for rule in all_rules() {
        let rule_issues = rule.check(&asset, None);
        issues.extend(rule_issues);
    }

    issues
}

/// Returns all mesh lint rules.
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        // Error-level rules
        Box::new(NonManifoldRule),
        Box::new(DegenerateFacesRule),
        Box::new(UnweightedVertsRule),
        Box::new(InvertedNormalsRule),
        // Warning-level rules
        Box::new(HumanoidProportionsRule),
        Box::new(UvOverlapRule),
        Box::new(UvStretchRule),
        Box::new(MissingMaterialRule),
        Box::new(ExcessiveNgonsRule),
        Box::new(IsolatedVertsRule),
        // Info-level rules
        Box::new(HighPolyRule),
        Box::new(NoUvsRule),
    ]
}
