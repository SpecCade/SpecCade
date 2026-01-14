//! Bone color types and schemes for color coding bones.

use serde::{Deserialize, Serialize};

/// RGB color value for bone coloring (0.0-1.0 range).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneColor {
    /// Red component (0.0-1.0).
    pub r: f64,
    /// Green component (0.0-1.0).
    pub g: f64,
    /// Blue component (0.0-1.0).
    pub b: f64,
}

impl BoneColor {
    /// Creates a new bone color from RGB values.
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    /// Standard blue color for left-side bones.
    pub fn left_blue() -> Self {
        Self::new(0.2, 0.4, 1.0)
    }

    /// Standard red color for right-side bones.
    pub fn right_red() -> Self {
        Self::new(1.0, 0.3, 0.3)
    }

    /// Standard yellow color for center bones.
    pub fn center_yellow() -> Self {
        Self::new(1.0, 0.9, 0.2)
    }

    /// White color (default/neutral).
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Returns the color as an array [R, G, B].
    pub fn as_array(&self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for BoneColor {
    fn default() -> Self {
        Self::white()
    }
}

/// Bone color scheme for automatic bone coloring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "scheme", rename_all = "snake_case", deny_unknown_fields)]
pub enum BoneColorScheme {
    /// Standard scheme: left=blue, right=red, center=yellow.
    #[default]
    Standard,
    /// Custom color mapping by bone side.
    Custom {
        /// Color for left-side bones (suffix `_l` or `_L`).
        left: BoneColor,
        /// Color for right-side bones (suffix `_r` or `_R`).
        right: BoneColor,
        /// Color for center bones (no side suffix).
        center: BoneColor,
    },
    /// Per-bone custom colors.
    PerBone {
        /// Map of bone name to color.
        colors: std::collections::HashMap<String, BoneColor>,
        /// Default color for bones not in the map.
        #[serde(default)]
        default: BoneColor,
    },
}

impl BoneColorScheme {
    /// Creates the standard color scheme (L=blue, R=red, center=yellow).
    pub fn standard() -> Self {
        BoneColorScheme::Standard
    }

    /// Creates a custom color scheme with specified colors.
    pub fn custom(left: BoneColor, right: BoneColor, center: BoneColor) -> Self {
        BoneColorScheme::Custom {
            left,
            right,
            center,
        }
    }

    /// Returns the color for a given bone name based on the scheme.
    pub fn color_for_bone(&self, bone_name: &str) -> BoneColor {
        match self {
            BoneColorScheme::Standard => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    BoneColor::left_blue()
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    BoneColor::right_red()
                } else {
                    BoneColor::center_yellow()
                }
            }
            BoneColorScheme::Custom {
                left,
                right,
                center,
            } => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    *left
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    *right
                } else {
                    *center
                }
            }
            BoneColorScheme::PerBone { colors, default } => {
                colors.get(bone_name).copied().unwrap_or(*default)
            }
        }
    }
}
