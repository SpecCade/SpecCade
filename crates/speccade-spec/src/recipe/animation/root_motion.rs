//! Root motion handling modes and settings for animation export.

use serde::{Deserialize, Serialize};

/// Root motion handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RootMotionMode {
    /// Keep root motion as-is (default).
    #[default]
    Keep,
    /// Extract root motion into a separate delta, zero out root bone.
    Extract,
    /// Transfer root motion to the hip bone.
    BakeToHip,
    /// Lock root bone location on specified axes (zero out).
    Lock,
}

/// Root motion settings for animation export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootMotionSettings {
    /// Which root motion handling mode to use.
    #[serde(default)]
    pub mode: RootMotionMode,
    /// Which axes to apply to \[X, Y, Z\].
    #[serde(default = "default_axes")]
    pub axes: [bool; 3],
    /// Reference height for ground plane (used in BakeToHip mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_height: Option<f64>,
}

fn default_axes() -> [bool; 3] {
    [true, true, true]
}

impl Default for RootMotionSettings {
    fn default() -> Self {
        Self {
            mode: RootMotionMode::default(),
            axes: default_axes(),
            ground_height: None,
        }
    }
}

impl PartialEq for RootMotionSettings {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && self.axes == other.axes && self.ground_height == other.ground_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_motion_mode_default() {
        let mode = RootMotionMode::default();
        assert_eq!(mode, RootMotionMode::Keep);
    }

    #[test]
    fn test_root_motion_settings_default() {
        let settings = RootMotionSettings::default();
        assert_eq!(settings.mode, RootMotionMode::Keep);
        assert_eq!(settings.axes, [true, true, true]);
        assert!(settings.ground_height.is_none());
    }

    #[test]
    fn test_root_motion_mode_serde() {
        let modes = [
            (RootMotionMode::Keep, "\"keep\""),
            (RootMotionMode::Extract, "\"extract\""),
            (RootMotionMode::BakeToHip, "\"bake_to_hip\""),
            (RootMotionMode::Lock, "\"lock\""),
        ];

        for (mode, expected) in modes {
            let json = serde_json::to_string(&mode).unwrap();
            assert_eq!(json, expected);

            let parsed: RootMotionMode = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn test_root_motion_settings_serde_full() {
        let settings = RootMotionSettings {
            mode: RootMotionMode::Extract,
            axes: [true, false, true],
            ground_height: Some(0.05),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: RootMotionSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, settings);
    }

    #[test]
    fn test_root_motion_settings_serde_minimal() {
        let json = r#"{"mode": "lock"}"#;
        let parsed: RootMotionSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.mode, RootMotionMode::Lock);
        assert_eq!(parsed.axes, [true, true, true]);
        assert!(parsed.ground_height.is_none());
    }

    #[test]
    fn test_root_motion_settings_serde_defaults() {
        let json = r#"{}"#;
        let parsed: RootMotionSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.mode, RootMotionMode::Keep);
        assert_eq!(parsed.axes, [true, true, true]);
    }
}
