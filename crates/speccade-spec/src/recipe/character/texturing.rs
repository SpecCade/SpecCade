//! Texturing configuration for UV unwrapping and material regions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Texturing configuration for UV unwrapping and material regions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Texturing {
    /// UV unwrapping mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_mode: Option<UvMode>,
    /// Material region definitions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub regions: HashMap<String, TextureRegion>,
}

/// UV unwrapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvMode {
    /// Smart UV project (automatic island detection).
    SmartProject,
    /// Region-based UV mapping (manual region assignment).
    RegionBased,
    /// Lightmap pack.
    LightmapPack,
    /// Cube projection.
    CubeProject,
    /// Cylinder projection.
    CylinderProject,
    /// Sphere projection.
    SphereProject,
}

impl Default for UvMode {
    fn default() -> Self {
        UvMode::SmartProject
    }
}

/// A texture region definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureRegion {
    /// Parts included in this region.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
    /// Material index for this region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,
    /// UV island index hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island: Option<u32>,
    /// Color for this region (hex string or [R, G, B]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<RegionColor>,
}

/// Color specification for a region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegionColor {
    /// Hex color string (e.g., "#FF0000").
    Hex(String),
    /// RGB array [R, G, B] with values 0-1.
    Rgb([f64; 3]),
    /// RGBA array [R, G, B, A] with values 0-1.
    Rgba([f64; 4]),
}
