//! Stretch and bake settings for skeletal animation.

use serde::{Deserialize, Serialize};

// =============================================================================
// Stretch Settings
// =============================================================================

/// Stretch settings for IK chains.
/// Allows bones to stretch beyond their rest length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct StretchSettings {
    /// Whether stretch is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Maximum stretch factor (1.0 = no stretch, 2.0 = double length).
    #[serde(default = "default_max_stretch")]
    pub max_stretch: f64,
    /// Minimum stretch factor (0.5 = half length).
    #[serde(default = "default_min_stretch")]
    pub min_stretch: f64,
    /// Volume preservation mode.
    #[serde(default)]
    pub volume_preservation: VolumePreservation,
}

fn default_max_stretch() -> f64 {
    1.5
}

fn default_min_stretch() -> f64 {
    0.5
}

/// Volume preservation mode for stretch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumePreservation {
    /// No volume preservation.
    #[default]
    None,
    /// Preserve volume by scaling perpendicular axes.
    Uniform,
    /// Preserve volume on X axis.
    X,
    /// Preserve volume on Z axis.
    Z,
}

impl StretchSettings {
    /// Creates new stretch settings with stretch enabled.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            max_stretch: default_max_stretch(),
            min_stretch: default_min_stretch(),
            volume_preservation: VolumePreservation::default(),
        }
    }

    /// Sets the stretch limits.
    pub fn with_limits(mut self, min: f64, max: f64) -> Self {
        self.min_stretch = min.max(0.0);
        self.max_stretch = max.max(min);
        self
    }

    /// Sets the volume preservation mode.
    pub fn with_volume_preservation(mut self, mode: VolumePreservation) -> Self {
        self.volume_preservation = mode;
        self
    }
}

// =============================================================================
// Bake Settings
// =============================================================================

/// Settings for baking animation to keyframes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BakeSettings {
    /// Simplify curves after baking (remove redundant keyframes).
    #[serde(default = "default_true")]
    pub simplify: bool,
    /// Start frame for baking (None = use scene start).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i32>,
    /// End frame for baking (None = use scene end).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<i32>,
    /// Use visual keying (bake visual transforms, not raw values).
    #[serde(default = "default_true")]
    pub visual_keying: bool,
    /// Clear constraints after baking.
    #[serde(default = "default_true")]
    pub clear_constraints: bool,
    /// Frame step for baking (1 = every frame).
    #[serde(default = "default_frame_step")]
    pub frame_step: u32,
    /// Tolerance for curve simplification.
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
    /// Remove IK control bones after baking.
    #[serde(default = "default_true")]
    pub remove_ik_bones: bool,
}

fn default_true() -> bool {
    true
}

fn default_frame_step() -> u32 {
    1
}

fn default_tolerance() -> f64 {
    0.001
}

impl Default for BakeSettings {
    fn default() -> Self {
        Self {
            simplify: true,
            start_frame: None,
            end_frame: None,
            visual_keying: true,
            clear_constraints: true,
            frame_step: default_frame_step(),
            tolerance: default_tolerance(),
            remove_ik_bones: true,
        }
    }
}

impl BakeSettings {
    /// Creates new bake settings with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the frame range for baking.
    pub fn with_frame_range(mut self, start: i32, end: i32) -> Self {
        self.start_frame = Some(start);
        self.end_frame = Some(end);
        self
    }

    /// Sets the frame step.
    pub fn with_frame_step(mut self, step: u32) -> Self {
        self.frame_step = step.max(1);
        self
    }

    /// Sets whether to simplify curves.
    pub fn with_simplify(mut self, simplify: bool) -> Self {
        self.simplify = simplify;
        self
    }

    /// Sets the simplification tolerance.
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance.abs();
        self
    }

    /// Sets whether to clear constraints after baking.
    pub fn with_clear_constraints(mut self, clear: bool) -> Self {
        self.clear_constraints = clear;
        self
    }

    /// Sets whether to use visual keying.
    pub fn with_visual_keying(mut self, visual: bool) -> Self {
        self.visual_keying = visual;
        self
    }
}
