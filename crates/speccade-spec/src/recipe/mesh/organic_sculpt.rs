//! Organic sculpt mesh recipe definitions.
//!
//! This module provides the `static_mesh.organic_sculpt_v1` recipe for generating
//! organic shapes using metaballs, remeshing, smoothing, and displacement noise
//! in Blender (Tier 2 backend).

use serde::{Deserialize, Serialize};

use super::common::MeshExportSettings;

/// Maximum number of metaballs allowed.
pub const MAX_METABALLS: usize = 200;

/// Maximum smooth iterations allowed.
pub const MAX_SMOOTH_ITERATIONS: u8 = 10;

/// Minimum remesh voxel size.
pub const MIN_REMESH_VOXEL_SIZE: f64 = 0.01;

/// Maximum remesh voxel size.
pub const MAX_REMESH_VOXEL_SIZE: f64 = 1.0;

/// Parameters for the `static_mesh.organic_sculpt_v1` recipe.
///
/// This recipe creates organic shapes using metaballs as the base,
/// followed by voxel remeshing, smoothing, and optional displacement noise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMeshOrganicSculptV1Params {
    /// Metaball sources that define the organic shape.
    pub metaballs: Vec<MetaballSource>,
    /// Voxel remesh resolution (0.01 to 1.0).
    /// Smaller values = higher detail, larger file size.
    pub remesh_voxel_size: f64,
    /// Number of smooth iterations to apply (0-10).
    #[serde(default)]
    pub smooth_iterations: u8,
    /// Optional displacement noise settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displacement: Option<DisplacementNoise>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
}

/// A metaball source definition.
///
/// Metaballs are implicit surfaces that blend together smoothly,
/// useful for creating organic shapes like blobs, characters, or liquids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaballSource {
    /// Position [X, Y, Z] in world coordinates.
    pub position: [f64; 3],
    /// Radius of the metaball.
    pub radius: f64,
    /// Influence/stiffness (controls blending falloff).
    /// Higher values = sharper falloff, less blending.
    /// Default: 2.0
    #[serde(
        default = "default_stiffness",
        skip_serializing_if = "is_default_stiffness"
    )]
    pub stiffness: f64,
}

fn default_stiffness() -> f64 {
    2.0
}

fn is_default_stiffness(v: &f64) -> bool {
    (*v - 2.0).abs() < f64::EPSILON
}

/// Displacement noise settings for adding surface detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplacementNoise {
    /// Displacement strength (positive value).
    pub strength: f64,
    /// Noise scale (affects the frequency of displacement).
    pub scale: f64,
    /// Number of octaves for fractal noise (1-8).
    #[serde(
        default = "default_octaves",
        skip_serializing_if = "is_default_octaves"
    )]
    pub octaves: u8,
    /// Random seed for noise generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

fn default_octaves() -> u8 {
    4
}

fn is_default_octaves(v: &u8) -> bool {
    *v == 4
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // MetaballSource Tests
    // ========================================================================

    #[test]
    fn test_metaball_source_basic() {
        let metaball = MetaballSource {
            position: [0.0, 0.0, 0.0],
            radius: 1.0,
            stiffness: 2.0,
        };

        let json = serde_json::to_string(&metaball).unwrap();
        assert!(json.contains("\"position\":[0.0,0.0,0.0]"));
        assert!(json.contains("\"radius\":1.0"));
        // Default stiffness should be omitted
        assert!(!json.contains("stiffness"));

        let parsed: MetaballSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.position, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.radius, 1.0);
        assert_eq!(parsed.stiffness, 2.0);
    }

    #[test]
    fn test_metaball_source_custom_stiffness() {
        let metaball = MetaballSource {
            position: [1.0, 2.0, 3.0],
            radius: 0.5,
            stiffness: 3.5,
        };

        let json = serde_json::to_string(&metaball).unwrap();
        assert!(json.contains("\"stiffness\":3.5"));

        let parsed: MetaballSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stiffness, 3.5);
    }

    #[test]
    fn test_metaball_source_from_json_defaults() {
        let json = r#"{"position":[0.0,0.0,0.0],"radius":1.0}"#;
        let parsed: MetaballSource = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.stiffness, 2.0);
    }

    #[test]
    fn test_metaball_source_rejects_unknown_fields() {
        let json = r#"{"position":[0.0,0.0,0.0],"radius":1.0,"unknown":true}"#;
        let result: Result<MetaballSource, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // DisplacementNoise Tests
    // ========================================================================

    #[test]
    fn test_displacement_noise_basic() {
        let displacement = DisplacementNoise {
            strength: 0.1,
            scale: 2.0,
            octaves: 4,
            seed: None,
        };

        let json = serde_json::to_string(&displacement).unwrap();
        assert!(json.contains("\"strength\":0.1"));
        assert!(json.contains("\"scale\":2.0"));
        // Default octaves should be omitted
        assert!(!json.contains("octaves"));
        // None seed should be omitted
        assert!(!json.contains("seed"));

        let parsed: DisplacementNoise = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.strength, 0.1);
        assert_eq!(parsed.scale, 2.0);
        assert_eq!(parsed.octaves, 4);
        assert_eq!(parsed.seed, None);
    }

    #[test]
    fn test_displacement_noise_with_seed() {
        let displacement = DisplacementNoise {
            strength: 0.2,
            scale: 3.0,
            octaves: 6,
            seed: Some(12345),
        };

        let json = serde_json::to_string(&displacement).unwrap();
        assert!(json.contains("\"octaves\":6"));
        assert!(json.contains("\"seed\":12345"));

        let parsed: DisplacementNoise = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.octaves, 6);
        assert_eq!(parsed.seed, Some(12345));
    }

    #[test]
    fn test_displacement_noise_from_json_defaults() {
        let json = r#"{"strength":0.1,"scale":2.0}"#;
        let parsed: DisplacementNoise = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.octaves, 4);
        assert_eq!(parsed.seed, None);
    }

    #[test]
    fn test_displacement_noise_rejects_unknown_fields() {
        let json = r#"{"strength":0.1,"scale":2.0,"unknown":true}"#;
        let result: Result<DisplacementNoise, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // StaticMeshOrganicSculptV1Params Tests
    // ========================================================================

    #[test]
    fn test_organic_sculpt_params_basic() {
        let params = StaticMeshOrganicSculptV1Params {
            metaballs: vec![MetaballSource {
                position: [0.0, 0.0, 0.0],
                radius: 1.0,
                stiffness: 2.0,
            }],
            remesh_voxel_size: 0.1,
            smooth_iterations: 2,
            displacement: None,
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"metaballs\""));
        assert!(json.contains("\"remesh_voxel_size\":0.1"));
        assert!(json.contains("\"smooth_iterations\":2"));
        assert!(!json.contains("displacement"));
        assert!(!json.contains("export"));

        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metaballs.len(), 1);
        assert_eq!(parsed.remesh_voxel_size, 0.1);
        assert_eq!(parsed.smooth_iterations, 2);
    }

    #[test]
    fn test_organic_sculpt_params_with_displacement() {
        let params = StaticMeshOrganicSculptV1Params {
            metaballs: vec![
                MetaballSource {
                    position: [0.0, 0.0, 0.0],
                    radius: 1.0,
                    stiffness: 2.0,
                },
                MetaballSource {
                    position: [0.5, 0.0, 0.0],
                    radius: 0.8,
                    stiffness: 2.0,
                },
            ],
            remesh_voxel_size: 0.05,
            smooth_iterations: 3,
            displacement: Some(DisplacementNoise {
                strength: 0.1,
                scale: 2.0,
                octaves: 4,
                seed: Some(42),
            }),
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"displacement\""));
        assert!(json.contains("\"strength\":0.1"));
        assert!(json.contains("\"seed\":42"));

        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metaballs.len(), 2);
        assert!(parsed.displacement.is_some());
        let disp = parsed.displacement.unwrap();
        assert_eq!(disp.strength, 0.1);
        assert_eq!(disp.seed, Some(42));
    }

    #[test]
    fn test_organic_sculpt_params_with_export() {
        let params = StaticMeshOrganicSculptV1Params {
            metaballs: vec![MetaballSource {
                position: [0.0, 0.0, 0.0],
                radius: 1.0,
                stiffness: 2.0,
            }],
            remesh_voxel_size: 0.1,
            smooth_iterations: 0,
            displacement: None,
            export: Some(MeshExportSettings {
                apply_modifiers: true,
                triangulate: true,
                include_normals: true,
                include_uvs: true,
                include_vertex_colors: false,
                tangents: false,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"export\""));
        assert!(json.contains("\"apply_modifiers\":true"));

        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.export.is_some());
    }

    #[test]
    fn test_organic_sculpt_params_default_smooth_iterations() {
        let json = r#"{
            "metaballs": [{"position":[0.0,0.0,0.0],"radius":1.0}],
            "remesh_voxel_size": 0.1
        }"#;
        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.smooth_iterations, 0);
    }

    #[test]
    fn test_organic_sculpt_params_rejects_unknown_fields() {
        let json = r#"{
            "metaballs": [{"position":[0.0,0.0,0.0],"radius":1.0}],
            "remesh_voxel_size": 0.1,
            "unknown_field": true
        }"#;
        let result: Result<StaticMeshOrganicSculptV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_organic_sculpt_params_complete() {
        let params = StaticMeshOrganicSculptV1Params {
            metaballs: vec![
                MetaballSource {
                    position: [0.0, 0.0, 0.0],
                    radius: 1.0,
                    stiffness: 2.0,
                },
                MetaballSource {
                    position: [0.8, 0.0, 0.0],
                    radius: 0.6,
                    stiffness: 2.5,
                },
                MetaballSource {
                    position: [-0.3, 0.5, 0.0],
                    radius: 0.4,
                    stiffness: 3.0,
                },
            ],
            remesh_voxel_size: 0.08,
            smooth_iterations: 4,
            displacement: Some(DisplacementNoise {
                strength: 0.05,
                scale: 4.0,
                octaves: 6,
                seed: Some(123),
            }),
            export: Some(MeshExportSettings::default()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.metaballs.len(), 3);
        assert_eq!(parsed.remesh_voxel_size, 0.08);
        assert_eq!(parsed.smooth_iterations, 4);
        assert!(parsed.displacement.is_some());
        assert!(parsed.export.is_some());
    }
}
