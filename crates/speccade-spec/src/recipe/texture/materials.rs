//! Material presets and patterns for texture recipes.

use serde::{Deserialize, Serialize};

/// Base material properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseMaterial {
    /// Material type.
    #[serde(rename = "type")]
    pub material_type: MaterialType,
    /// Base color as [R, G, B] (0.0 to 1.0).
    pub base_color: [f64; 3],
    /// Roughness range [min, max] (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_range: Option<[f64; 2]>,
    /// Metallic value (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f64>,
    /// Brick pattern configuration (only used when material_type is Brick).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brick_pattern: Option<BrickPatternParams>,
    /// Normal map parameters for materials with structured patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_params: Option<NormalParams>,
}

/// Brick pattern configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrickPatternParams {
    /// Brick width in pixels.
    pub brick_width: u32,
    /// Brick height in pixels.
    pub brick_height: u32,
    /// Mortar width in pixels.
    pub mortar_width: u32,
    /// Row offset (0.5 = standard half-brick offset).
    #[serde(default = "default_brick_offset")]
    pub offset: f64,
}

fn default_brick_offset() -> f64 {
    0.5
}

/// Normal map parameters for structured materials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalParams {
    /// Bump strength multiplier.
    #[serde(default = "default_bump_strength")]
    pub bump_strength: f64,
    /// Mortar depth (0.0 = flush, 1.0 = deep).
    #[serde(default = "default_mortar_depth")]
    pub mortar_depth: f64,
}

fn default_bump_strength() -> f64 {
    1.0
}

fn default_mortar_depth() -> f64 {
    0.3
}

/// Base material types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    /// Metal surface.
    Metal,
    /// Wood surface.
    Wood,
    /// Stone surface.
    Stone,
    /// Fabric surface.
    Fabric,
    /// Plastic surface.
    Plastic,
    /// Concrete surface.
    Concrete,
    /// Brick surface.
    Brick,
    /// Generic procedural.
    Procedural,
}
