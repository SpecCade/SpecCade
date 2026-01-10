//! Pose and animation phase types.

use serde::{Deserialize, Serialize};

use super::common::TimingCurve;

// =============================================================================
// Pose Definitions
// =============================================================================

/// A named pose definition containing bone rotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoseDefinition {
    /// Bone transforms in this pose.
    /// Keys are bone names, values are the rotations.
    pub bones: std::collections::HashMap<String, PoseBoneTransform>,
}

impl PoseDefinition {
    /// Creates a new empty pose definition.
    pub fn new() -> Self {
        Self {
            bones: std::collections::HashMap::new(),
        }
    }

    /// Adds a bone transform to this pose.
    pub fn with_bone(mut self, name: impl Into<String>, transform: PoseBoneTransform) -> Self {
        self.bones.insert(name.into(), transform);
        self
    }
}

impl Default for PoseDefinition {
    fn default() -> Self {
        Self::new()
    }
}

/// Bone transform within a pose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PoseBoneTransform {
    /// Pitch rotation in degrees (X axis).
    #[serde(default)]
    pub pitch: f64,
    /// Yaw rotation in degrees (Y axis).
    #[serde(default)]
    pub yaw: f64,
    /// Roll rotation in degrees (Z axis).
    #[serde(default)]
    pub roll: f64,
    /// Location offset [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<[f64; 3]>,
}

impl PoseBoneTransform {
    /// Creates a new pose bone transform with rotation.
    pub fn rotation(pitch: f64, yaw: f64, roll: f64) -> Self {
        Self {
            pitch,
            yaw,
            roll,
            location: None,
        }
    }

    /// Creates a new pose bone transform with only pitch.
    pub fn pitch(angle: f64) -> Self {
        Self::rotation(angle, 0.0, 0.0)
    }

    /// Creates a new pose bone transform with only yaw.
    pub fn yaw(angle: f64) -> Self {
        Self::rotation(0.0, angle, 0.0)
    }

    /// Creates a new pose bone transform with only roll.
    pub fn roll(angle: f64) -> Self {
        Self::rotation(0.0, 0.0, angle)
    }

    /// Sets the location offset.
    pub fn with_location(mut self, location: [f64; 3]) -> Self {
        self.location = Some(location);
        self
    }

    /// Returns the rotation as an array [pitch, yaw, roll] in degrees.
    pub fn as_euler_degrees(&self) -> [f64; 3] {
        [self.pitch, self.yaw, self.roll]
    }
}

// =============================================================================
// Animation Phases
// =============================================================================

/// IK target keyframe within a phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseIkTarget {
    /// Frame number for this keyframe.
    pub frame: i32,
    /// World position [X, Y, Z].
    pub location: [f64; 3],
    /// IK/FK blend value (0.0 = FK, 1.0 = IK).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ikfk: Option<f64>,
}

impl PhaseIkTarget {
    /// Creates a new IK target keyframe.
    pub fn new(frame: i32, location: [f64; 3]) -> Self {
        Self {
            frame,
            location,
            ikfk: None,
        }
    }

    /// Sets the IK/FK blend value.
    pub fn with_ikfk(mut self, blend: f64) -> Self {
        self.ikfk = Some(blend.clamp(0.0, 1.0));
        self
    }
}

/// Animation phase definition.
/// Defines a segment of the animation with timing and pose/IK targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationPhase {
    /// Name of this phase (e.g., "contact", "passing", "lift").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Start frame of this phase.
    pub start_frame: i32,
    /// End frame of this phase.
    pub end_frame: i32,
    /// Timing curve for interpolation.
    #[serde(default)]
    pub curve: TimingCurve,
    /// Named pose to apply during this phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pose: Option<String>,
    /// IK target keyframes for this phase.
    /// Keys are IK chain names, values are lists of keyframes.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub ik_targets: std::collections::HashMap<String, Vec<PhaseIkTarget>>,
    /// IK/FK blend keyframes for this phase.
    /// Keys are IK chain names, values are lists of (frame, blend) pairs.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub ikfk_blend: std::collections::HashMap<String, Vec<(i32, f64)>>,
}

impl AnimationPhase {
    /// Creates a new animation phase.
    pub fn new(start_frame: i32, end_frame: i32) -> Self {
        Self {
            name: None,
            start_frame,
            end_frame,
            curve: TimingCurve::Linear,
            pose: None,
            ik_targets: std::collections::HashMap::new(),
            ikfk_blend: std::collections::HashMap::new(),
        }
    }

    /// Sets the phase name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the timing curve.
    pub fn with_curve(mut self, curve: TimingCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the pose to apply.
    pub fn with_pose(mut self, pose: impl Into<String>) -> Self {
        self.pose = Some(pose.into());
        self
    }

    /// Adds IK target keyframes for a chain.
    pub fn with_ik_targets(
        mut self,
        chain: impl Into<String>,
        targets: Vec<PhaseIkTarget>,
    ) -> Self {
        self.ik_targets.insert(chain.into(), targets);
        self
    }

    /// Returns the duration in frames.
    pub fn duration_frames(&self) -> i32 {
        self.end_frame - self.start_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pose_bone_transform() {
        // Test rotation constructor
        let rot = PoseBoneTransform::rotation(15.0, 30.0, 45.0);
        assert_eq!(rot.pitch, 15.0);
        assert_eq!(rot.yaw, 30.0);
        assert_eq!(rot.roll, 45.0);
        assert!(rot.location.is_none());

        // Test pitch only
        let pitch = PoseBoneTransform::pitch(20.0);
        assert_eq!(pitch.pitch, 20.0);
        assert_eq!(pitch.yaw, 0.0);

        // Test yaw only
        let yaw = PoseBoneTransform::yaw(30.0);
        assert_eq!(yaw.yaw, 30.0);
        assert_eq!(yaw.pitch, 0.0);

        // Test roll only
        let roll = PoseBoneTransform::roll(40.0);
        assert_eq!(roll.roll, 40.0);
        assert_eq!(roll.pitch, 0.0);

        // Test with location
        let with_loc = PoseBoneTransform::rotation(10.0, 20.0, 30.0).with_location([0.1, 0.2, 0.3]);
        assert!(with_loc.location.is_some());
        assert_eq!(with_loc.location.unwrap(), [0.1, 0.2, 0.3]);

        // Test as_euler_degrees
        let euler = rot.as_euler_degrees();
        assert_eq!(euler, [15.0, 30.0, 45.0]);
    }

    #[test]
    fn test_pose_bone_transform_serde() {
        let transform =
            PoseBoneTransform::rotation(10.0, 20.0, 30.0).with_location([0.1, 0.2, 0.3]);

        let json = serde_json::to_string(&transform).unwrap();
        assert!(json.contains("\"pitch\":10.0"));
        assert!(json.contains("\"yaw\":20.0"));
        assert!(json.contains("\"roll\":30.0"));
        assert!(json.contains("location"));

        let parsed: PoseBoneTransform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pitch, 10.0);
        assert_eq!(parsed.yaw, 20.0);
        assert_eq!(parsed.roll, 30.0);
    }

    #[test]
    fn test_pose_definition() {
        let pose = PoseDefinition::new()
            .with_bone("arm_l", PoseBoneTransform::pitch(15.0))
            .with_bone("arm_r", PoseBoneTransform::pitch(-15.0));

        assert_eq!(pose.bones.len(), 2);
        assert!(pose.bones.contains_key("arm_l"));
        assert!(pose.bones.contains_key("arm_r"));
    }

    #[test]
    fn test_pose_definition_serde() {
        let pose =
            PoseDefinition::new().with_bone("leg_l", PoseBoneTransform::rotation(20.0, 0.0, 0.0));

        let json = serde_json::to_string(&pose).unwrap();
        assert!(json.contains("leg_l"));
        assert!(json.contains("pitch"));

        let parsed: PoseDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bones.len(), 1);
    }

    #[test]
    fn test_phase_ik_target() {
        let target = PhaseIkTarget::new(10, [0.1, 0.2, 0.3]);
        assert_eq!(target.frame, 10);
        assert_eq!(target.location, [0.1, 0.2, 0.3]);
        assert!(target.ikfk.is_none());

        let with_blend = target.with_ikfk(0.7);
        assert_eq!(with_blend.ikfk, Some(0.7));
    }

    #[test]
    fn test_phase_ik_target_serde() {
        let target = PhaseIkTarget::new(15, [0.5, 0.6, 0.7]).with_ikfk(0.8);

        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains("\"frame\":15"));
        assert!(json.contains("location"));
        assert!(json.contains("0.8"));

        let parsed: PhaseIkTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.frame, 15);
        assert_eq!(parsed.ikfk, Some(0.8));
    }

    #[test]
    fn test_animation_phase() {
        let phase = AnimationPhase::new(0, 30)
            .with_name("contact")
            .with_curve(TimingCurve::EaseInOut)
            .with_pose("standing");

        assert_eq!(phase.name, Some("contact".to_string()));
        assert_eq!(phase.start_frame, 0);
        assert_eq!(phase.end_frame, 30);
        assert_eq!(phase.curve, TimingCurve::EaseInOut);
        assert_eq!(phase.pose, Some("standing".to_string()));
        assert_eq!(phase.duration_frames(), 30);
    }

    #[test]
    fn test_animation_phase_with_ik_targets() {
        let targets = vec![
            PhaseIkTarget::new(0, [0.0, 0.0, 0.0]),
            PhaseIkTarget::new(15, [0.1, 0.0, 0.2]),
        ];

        let phase = AnimationPhase::new(0, 30).with_ik_targets("ik_foot_l", targets);

        assert!(phase.ik_targets.contains_key("ik_foot_l"));
        assert_eq!(phase.ik_targets["ik_foot_l"].len(), 2);
    }

    #[test]
    fn test_animation_phase_serde() {
        let phase = AnimationPhase::new(0, 60)
            .with_name("walk_cycle")
            .with_curve(TimingCurve::Linear)
            .with_pose("neutral");

        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("walk_cycle"));
        assert!(json.contains("linear"));
        assert!(json.contains("neutral"));

        let parsed: AnimationPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("walk_cycle".to_string()));
        assert_eq!(parsed.start_frame, 0);
        assert_eq!(parsed.end_frame, 60);
    }
}
