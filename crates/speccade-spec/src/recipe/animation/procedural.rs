//! Procedural animation layer types.

use serde::{Deserialize, Serialize};

// =============================================================================
// Procedural Animation Layers
// =============================================================================

/// Types of procedural animation layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralLayerType {
    /// Breathing animation (subtle chest/torso expansion).
    Breathing,
    /// Swaying motion (side-to-side idle motion).
    Sway,
    /// Bobbing motion (up-down motion, e.g., for floating).
    Bob,
    /// Noise-based random motion.
    Noise,
}

/// Rotation axis for procedural layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralAxis {
    /// Pitch rotation (X axis).
    #[default]
    Pitch,
    /// Yaw rotation (Y axis).
    Yaw,
    /// Roll rotation (Z axis).
    Roll,
}

impl ProceduralAxis {
    /// Converts to a rotation axis index (0=X, 1=Y, 2=Z).
    pub fn to_index(&self) -> usize {
        match self {
            ProceduralAxis::Pitch => 0,
            ProceduralAxis::Yaw => 1,
            ProceduralAxis::Roll => 2,
        }
    }
}

/// Procedural animation layer configuration.
/// Adds automatic motion overlays to bones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduralLayer {
    /// Type of procedural animation.
    #[serde(rename = "type")]
    pub layer_type: ProceduralLayerType,
    /// Target bone name.
    pub target: String,
    /// Rotation axis for the motion.
    #[serde(default)]
    pub axis: ProceduralAxis,
    /// Period in frames for sine-based animations.
    #[serde(default = "default_period_frames")]
    pub period_frames: u32,
    /// Amplitude of the motion (in radians for sine, degrees for noise).
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
    /// Phase offset (0.0-1.0) for staggered animations.
    #[serde(default)]
    pub phase_offset: f64,
    /// Frequency for noise-based animations.
    #[serde(default = "default_frequency")]
    pub frequency: f64,
}

fn default_period_frames() -> u32 {
    60
}

fn default_amplitude() -> f64 {
    0.01
}

fn default_frequency() -> f64 {
    0.3
}

impl ProceduralLayer {
    /// Creates a new breathing layer.
    pub fn breathing(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Breathing,
            target: target.into(),
            axis: ProceduralAxis::Pitch,
            period_frames: 90, // ~3 seconds at 30fps
            amplitude: 0.02,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new sway layer.
    pub fn sway(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Sway,
            target: target.into(),
            axis: ProceduralAxis::Roll,
            period_frames: 120, // ~4 seconds at 30fps
            amplitude: 0.03,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new bob layer.
    pub fn bob(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Bob,
            target: target.into(),
            axis: ProceduralAxis::Pitch,
            period_frames: 60,
            amplitude: 0.02,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new noise layer.
    pub fn noise(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Noise,
            target: target.into(),
            axis: ProceduralAxis::Roll,
            period_frames: default_period_frames(),
            amplitude: 1.0, // degrees
            phase_offset: 0.0,
            frequency: 0.3,
        }
    }

    /// Sets the rotation axis.
    pub fn with_axis(mut self, axis: ProceduralAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Sets the period in frames.
    pub fn with_period(mut self, frames: u32) -> Self {
        self.period_frames = frames.max(1);
        self
    }

    /// Sets the amplitude.
    pub fn with_amplitude(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude;
        self
    }

    /// Sets the phase offset.
    pub fn with_phase_offset(mut self, offset: f64) -> Self {
        self.phase_offset = offset;
        self
    }

    /// Sets the frequency for noise layers.
    pub fn with_frequency(mut self, frequency: f64) -> Self {
        self.frequency = frequency.max(0.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedural_layer_type_serde() {
        let types = [
            (ProceduralLayerType::Breathing, "\"breathing\""),
            (ProceduralLayerType::Sway, "\"sway\""),
            (ProceduralLayerType::Bob, "\"bob\""),
            (ProceduralLayerType::Noise, "\"noise\""),
        ];

        for (layer_type, expected) in types {
            let json = serde_json::to_string(&layer_type).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_procedural_axis_serde() {
        let axes = [
            (ProceduralAxis::Pitch, "\"pitch\""),
            (ProceduralAxis::Yaw, "\"yaw\""),
            (ProceduralAxis::Roll, "\"roll\""),
        ];

        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(ProceduralAxis::default(), ProceduralAxis::Pitch);

        // Test to_index
        assert_eq!(ProceduralAxis::Pitch.to_index(), 0);
        assert_eq!(ProceduralAxis::Yaw.to_index(), 1);
        assert_eq!(ProceduralAxis::Roll.to_index(), 2);
    }

    #[test]
    fn test_procedural_layer_breathing() {
        let layer = ProceduralLayer::breathing("chest");

        assert_eq!(layer.layer_type, ProceduralLayerType::Breathing);
        assert_eq!(layer.target, "chest");
        assert_eq!(layer.axis, ProceduralAxis::Pitch);
        assert_eq!(layer.period_frames, 90);
        assert_eq!(layer.amplitude, 0.02);
    }

    #[test]
    fn test_procedural_layer_sway() {
        let layer = ProceduralLayer::sway("spine");

        assert_eq!(layer.layer_type, ProceduralLayerType::Sway);
        assert_eq!(layer.target, "spine");
        assert_eq!(layer.axis, ProceduralAxis::Roll);
        assert_eq!(layer.period_frames, 120);
        assert_eq!(layer.amplitude, 0.03);
    }

    #[test]
    fn test_procedural_layer_bob() {
        let layer = ProceduralLayer::bob("body");

        assert_eq!(layer.layer_type, ProceduralLayerType::Bob);
        assert_eq!(layer.target, "body");
        assert_eq!(layer.axis, ProceduralAxis::Pitch);
        assert_eq!(layer.period_frames, 60);
        assert_eq!(layer.amplitude, 0.02);
    }

    #[test]
    fn test_procedural_layer_noise() {
        let layer = ProceduralLayer::noise("head");

        assert_eq!(layer.layer_type, ProceduralLayerType::Noise);
        assert_eq!(layer.target, "head");
        assert_eq!(layer.axis, ProceduralAxis::Roll);
        assert_eq!(layer.amplitude, 1.0);
        assert_eq!(layer.frequency, 0.3);
    }

    #[test]
    fn test_procedural_layer_builder() {
        let layer = ProceduralLayer::breathing("chest")
            .with_axis(ProceduralAxis::Yaw)
            .with_period(120)
            .with_amplitude(0.05)
            .with_phase_offset(0.5)
            .with_frequency(0.5);

        assert_eq!(layer.axis, ProceduralAxis::Yaw);
        assert_eq!(layer.period_frames, 120);
        assert_eq!(layer.amplitude, 0.05);
        assert_eq!(layer.phase_offset, 0.5);
        assert_eq!(layer.frequency, 0.5);
    }

    #[test]
    fn test_procedural_layer_serde() {
        let layer = ProceduralLayer::breathing("chest")
            .with_amplitude(0.05)
            .with_period(100);

        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("breathing"));
        assert!(json.contains("chest"));
        assert!(json.contains("0.05"));
        assert!(json.contains("100"));

        let parsed: ProceduralLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.layer_type, ProceduralLayerType::Breathing);
        assert_eq!(parsed.target, "chest");
        assert_eq!(parsed.amplitude, 0.05);
        assert_eq!(parsed.period_frames, 100);
    }
}
