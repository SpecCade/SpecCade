//! Common types shared across animation modules.

use serde::{Deserialize, Serialize};

// =============================================================================
// Interpolation and Timing
// =============================================================================

/// Interpolation mode for animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMode {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Bezier curve interpolation.
    Bezier,
    /// No interpolation (step/constant).
    Constant,
}

/// Timing curve types for phase interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingCurve {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    EaseOut,
    /// Ease in and out (slow start and end).
    EaseInOut,
    /// Exponential ease in.
    ExponentialIn,
    /// Exponential ease out.
    ExponentialOut,
    /// Constant (no interpolation, snap).
    Constant,
}

// =============================================================================
// Axis Types
// =============================================================================

/// Axis specification for constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConstraintAxis {
    /// X axis (pitch).
    #[default]
    X,
    /// Y axis (yaw).
    Y,
    /// Z axis (roll).
    Z,
}

impl ConstraintAxis {
    /// Returns the axis name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConstraintAxis::X => "X",
            ConstraintAxis::Y => "Y",
            ConstraintAxis::Z => "Z",
        }
    }
}

/// Aim axis options for aim constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AimAxis {
    /// Positive X axis.
    #[default]
    #[serde(rename = "X")]
    PosX,
    /// Negative X axis.
    #[serde(rename = "-X")]
    NegX,
    /// Positive Y axis.
    #[serde(rename = "Y")]
    PosY,
    /// Negative Y axis.
    #[serde(rename = "-Y")]
    NegY,
    /// Positive Z axis.
    #[serde(rename = "Z")]
    PosZ,
    /// Negative Z axis.
    #[serde(rename = "-Z")]
    NegZ,
}

impl AimAxis {
    /// Returns the Blender track axis name.
    pub fn blender_track_axis(&self) -> &'static str {
        match self {
            AimAxis::PosX => "TRACK_X",
            AimAxis::NegX => "TRACK_NEGATIVE_X",
            AimAxis::PosY => "TRACK_Y",
            AimAxis::NegY => "TRACK_NEGATIVE_Y",
            AimAxis::PosZ => "TRACK_Z",
            AimAxis::NegZ => "TRACK_NEGATIVE_Z",
        }
    }
}

// =============================================================================
// Export Settings
// =============================================================================

/// Export settings for animations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationExportSettings {
    /// Bake all transforms to keyframes.
    #[serde(default = "default_true")]
    pub bake_transforms: bool,
    /// Optimize keyframes (remove redundant keys).
    #[serde(default)]
    pub optimize_keyframes: bool,
    /// Export as separate file (vs embedded in mesh).
    #[serde(default)]
    pub separate_file: bool,
    /// Save .blend file alongside GLB output.
    #[serde(default)]
    pub save_blend: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AnimationExportSettings {
    fn default() -> Self {
        Self {
            bake_transforms: true,
            optimize_keyframes: false,
            separate_file: false,
            save_blend: false,
        }
    }
}

/// Validation conventions configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConventionsConfig {
    /// Fail on validation errors (strict mode).
    #[serde(default)]
    pub strict: bool,
}

// =============================================================================
// Animation Presets
// =============================================================================

/// Common animation presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPreset {
    /// Idle breathing animation.
    Idle,
    /// Walk cycle.
    Walk,
    /// Run cycle.
    Run,
    /// Jump animation.
    Jump,
    /// Attack animation.
    Attack,
    /// Hit/damage reaction.
    Hit,
    /// Death animation.
    Death,
}

impl AnimationPreset {
    /// Returns a suggested duration for this animation type.
    pub fn suggested_duration(&self) -> f64 {
        match self {
            AnimationPreset::Idle => 2.0,
            AnimationPreset::Walk => 1.0,
            AnimationPreset::Run => 0.6,
            AnimationPreset::Jump => 0.8,
            AnimationPreset::Attack => 0.5,
            AnimationPreset::Hit => 0.3,
            AnimationPreset::Death => 1.5,
        }
    }

    /// Returns whether this animation typically loops.
    pub fn typically_loops(&self) -> bool {
        matches!(
            self,
            AnimationPreset::Idle | AnimationPreset::Walk | AnimationPreset::Run
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_mode_serde() {
        let mode = InterpolationMode::Bezier;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"bezier\"");

        let parsed: InterpolationMode = serde_json::from_str("\"linear\"").unwrap();
        assert_eq!(parsed, InterpolationMode::Linear);
    }

    #[test]
    fn test_timing_curve_serde() {
        let curves = [
            (TimingCurve::Linear, "\"linear\""),
            (TimingCurve::EaseIn, "\"ease_in\""),
            (TimingCurve::EaseOut, "\"ease_out\""),
            (TimingCurve::EaseInOut, "\"ease_in_out\""),
            (TimingCurve::ExponentialIn, "\"exponential_in\""),
            (TimingCurve::ExponentialOut, "\"exponential_out\""),
            (TimingCurve::Constant, "\"constant\""),
        ];

        for (curve, expected) in curves {
            let json = serde_json::to_string(&curve).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(TimingCurve::default(), TimingCurve::Linear);
    }

    #[test]
    fn test_constraint_axis_serde() {
        // Test default
        assert_eq!(ConstraintAxis::default(), ConstraintAxis::X);

        // Test all variants serialize correctly
        let axes = [
            (ConstraintAxis::X, "\"X\""),
            (ConstraintAxis::Y, "\"Y\""),
            (ConstraintAxis::Z, "\"Z\""),
        ];
        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test as_str method
        assert_eq!(ConstraintAxis::X.as_str(), "X");
        assert_eq!(ConstraintAxis::Y.as_str(), "Y");
        assert_eq!(ConstraintAxis::Z.as_str(), "Z");
    }

    #[test]
    fn test_aim_axis_serde() {
        let axes = [
            (AimAxis::PosX, "\"X\""),
            (AimAxis::NegX, "\"-X\""),
            (AimAxis::PosY, "\"Y\""),
            (AimAxis::NegY, "\"-Y\""),
            (AimAxis::PosZ, "\"Z\""),
            (AimAxis::NegZ, "\"-Z\""),
        ];

        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(AimAxis::default(), AimAxis::PosX);

        // Test blender_track_axis
        assert_eq!(AimAxis::PosX.blender_track_axis(), "TRACK_X");
        assert_eq!(AimAxis::NegX.blender_track_axis(), "TRACK_NEGATIVE_X");
        assert_eq!(AimAxis::PosY.blender_track_axis(), "TRACK_Y");
        assert_eq!(AimAxis::NegY.blender_track_axis(), "TRACK_NEGATIVE_Y");
        assert_eq!(AimAxis::PosZ.blender_track_axis(), "TRACK_Z");
        assert_eq!(AimAxis::NegZ.blender_track_axis(), "TRACK_NEGATIVE_Z");
    }

    #[test]
    fn test_animation_preset() {
        assert_eq!(AnimationPreset::Walk.suggested_duration(), 1.0);
        assert!(AnimationPreset::Walk.typically_loops());
        assert!(!AnimationPreset::Death.typically_loops());
    }

    #[test]
    fn test_conventions_config_serde() {
        let config = ConventionsConfig { strict: true };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"strict\":true"));

        let parsed: ConventionsConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.strict);

        // Test default
        let default = ConventionsConfig::default();
        assert!(!default.strict);
    }

    #[test]
    fn test_animation_export_settings_default() {
        let export = AnimationExportSettings::default();

        assert!(export.bake_transforms);
        assert!(!export.optimize_keyframes);
        assert!(!export.separate_file);
        assert!(!export.save_blend);
    }

    #[test]
    fn test_animation_export_settings_serde() {
        let export = AnimationExportSettings {
            bake_transforms: false,
            optimize_keyframes: true,
            separate_file: true,
            save_blend: true,
        };

        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("\"bake_transforms\":false"));
        assert!(json.contains("\"optimize_keyframes\":true"));
        assert!(json.contains("\"separate_file\":true"));
        assert!(json.contains("\"save_blend\":true"));

        let parsed: AnimationExportSettings = serde_json::from_str(&json).unwrap();
        assert!(!parsed.bake_transforms);
        assert!(parsed.optimize_keyframes);
        assert!(parsed.separate_file);
        assert!(parsed.save_blend);
    }
}
