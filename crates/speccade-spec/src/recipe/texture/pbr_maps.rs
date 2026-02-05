//! PBR material maps types for texture recipes.

use serde::{Deserialize, Serialize};

use super::common::TextureMapType;
use super::layers::TextureLayer;
use super::materials::BaseMaterial;

/// Parameters for the `texture.material_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureMaterialV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether the texture should tile seamlessly.
    pub tileable: bool,
    /// Which PBR maps to generate.
    pub maps: Vec<TextureMapType>,
    /// Base material properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_material: Option<BaseMaterial>,
    /// Procedural layers to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<TextureLayer>,
    /// Discrete color palette for remapping values (hex colors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<Vec<String>>,
    /// Interpolated color ramp (hex colors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_ramp: Option<Vec<String>>,
}
