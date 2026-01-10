//! Material presets and patterns for texture recipes.

use serde::{Deserialize, Serialize};

/// Base material properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
