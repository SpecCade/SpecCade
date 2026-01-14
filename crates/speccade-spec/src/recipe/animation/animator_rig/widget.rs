//! Widget shape styles for bone visualization.

use serde::{Deserialize, Serialize};

/// Widget shape styles for bone visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetStyle {
    /// Wireframe circle (default, good for rotation controls).
    #[default]
    WireCircle,
    /// Wireframe cube (good for position controls).
    WireCube,
    /// Wireframe sphere (good for ball joints).
    WireSphere,
    /// Wireframe diamond (good for IK targets).
    WireDiamond,
    /// Custom mesh widget (requires external mesh reference).
    CustomMesh,
}

impl WidgetStyle {
    /// Returns the Blender object name for this widget style.
    pub fn blender_name(&self) -> &'static str {
        match self {
            WidgetStyle::WireCircle => "WGT_circle",
            WidgetStyle::WireCube => "WGT_cube",
            WidgetStyle::WireSphere => "WGT_sphere",
            WidgetStyle::WireDiamond => "WGT_diamond",
            WidgetStyle::CustomMesh => "WGT_custom",
        }
    }

    /// Returns all standard widget styles (excluding CustomMesh).
    pub fn standard_styles() -> &'static [WidgetStyle] {
        &[
            WidgetStyle::WireCircle,
            WidgetStyle::WireCube,
            WidgetStyle::WireSphere,
            WidgetStyle::WireDiamond,
        ]
    }
}
