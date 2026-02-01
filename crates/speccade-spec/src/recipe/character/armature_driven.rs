//! Parameters and supporting types for `skeletal_mesh.armature_driven_v1`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::recipe::mesh::{MaterialSlot, MeshPrimitive};

use super::{SkeletalMeshConstraints, SkeletalMeshExportSettings, SkeletonBone, SkeletonPreset};

// ============================================================================
// Connection Mode Types
// ============================================================================

/// Connection mode for bone mesh boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// No topological connection (current behavior) - mesh ends are independent.
    Segmented,
    /// Bridge edge loops with adjacent bone's mesh, blend weights at junction.
    Bridge,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Segmented
    }
}

// ============================================================================
// Step-Based Extrusion Types
// ============================================================================

/// A single extrusion step - either shorthand (just distance) or full definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtrusionStep {
    /// Shorthand: just extrusion distance as fraction of bone length.
    Shorthand(f64),
    /// Full step with all parameters.
    Full(ExtrusionStepDef),
}

/// Full extrusion step definition with all modifiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtrusionStepDef {
    /// Extrusion distance as fraction of bone length (required, must be > 0).
    pub extrude: f64,

    /// Scale factor for the extruded ring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<ScaleValue>,

    /// Translation offset (bone-relative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<[f64; 3]>,

    /// Z-axis rotation in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<f64>,

    /// X/Y tilt in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt: Option<TiltValue>,

    /// Asymmetric bulge multiplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulge: Option<StepBulgeValue>,
}

/// Scale can be uniform or per-axis [x, y].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScaleValue {
    /// Uniform scale factor.
    Uniform(f64),
    /// Per-axis scale [x, y].
    PerAxis([f64; 2]),
}

/// Tilt can be single value (X only) or both axes [x, y].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TiltValue {
    /// Single axis tilt (X rotation in degrees).
    SingleAxis(f64),
    /// Both axes tilt [x, y] in degrees.
    BothAxes([f64; 2]),
}

/// Bulge can be uniform or asymmetric [side, front_back].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StepBulgeValue {
    /// Uniform bulge multiplier.
    Uniform(f64),
    /// Asymmetric bulge [side, front_back].
    Asymmetric([f64; 2]),
}

/// Parameters for the `skeletal_mesh.armature_driven_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMeshArmatureDrivenV1Params {
    /// Predefined skeleton rig.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,

    /// Custom skeleton definition (alternative to preset).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skeleton: Vec<SkeletonBone>,

    /// Per-bone mesh definitions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bone_meshes: HashMap<String, ArmatureDrivenBoneMeshDef>,

    /// Boolean (subtraction) shapes that are not rendered.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bool_shapes: HashMap<String, ArmatureDrivenBoolShapeDef>,

    /// Material slot definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,

    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<SkeletalMeshExportSettings>,

    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<SkeletalMeshConstraints>,
}

/// Bone mesh definition.
///
/// Supports either a concrete mesh definition or a mirror reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmatureDrivenBoneMeshDef {
    Mirror(ArmatureDrivenMirrorRef),
    Mesh(ArmatureDrivenBoneMesh),
}

/// Mirror reference (e.g. `{ "mirror": "arm_upper_L" }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenMirrorRef {
    pub mirror: String,
}

/// Mesh definition for a single bone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenBoneMesh {
    /// Cross-section profile name (e.g. `circle(8)`, `hexagon(6)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    /// Cross-section radius in bone-relative units, elliptical units, or absolute units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_radius: Option<BoneRelativeLength>,

    /// Step-based extrusion along the bone axis.
    /// Steps sum to 1.0 = mesh exactly spans bone head to tail.
    /// Steps sum to >1.0 = mesh extends past bone tail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extrusion_steps: Vec<ExtrusionStep>,

    /// Bone-relative translation offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<[f64; 3]>,

    /// Rotation (degrees) applied to the profile before extrusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<[f64; 3]>,

    /// Cap at bone start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_start: Option<bool>,

    /// Cap at bone end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_end: Option<bool>,

    /// Modifiers applied in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ArmatureDrivenModifier>,

    /// Material index (into `material_slots`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,

    /// Attachments (geometry not necessarily on the bone axis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<ArmatureDrivenAttachment>,

    /// Connection mode at the start (head) of the bone mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_start: Option<ConnectionMode>,

    /// Connection mode at the end (tail) of the bone mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_end: Option<ConnectionMode>,
}

/// Length value in bone-relative units, elliptical units, or absolute units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoneRelativeLength {
    /// Uniform radius, in bone-relative units.
    Relative(f64),
    /// Elliptical radius, in bone-relative units.
    Relative2([f64; 2]),
    /// Absolute units escape hatch.
    Absolute { absolute: f64 },
}

/// Modifier entry.
///
/// Matches YAML style like `- bevel: { width: 0.02, segments: 2 }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmatureDrivenModifier {
    Bevel {
        bevel: ArmatureDrivenBevel,
    },
    Subdivide {
        subdivide: ArmatureDrivenSubdivide,
    },
    Bool {
        #[serde(rename = "bool")]
        r#bool: ArmatureDrivenBoolean,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenBevel {
    pub width: f64,
    pub segments: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenSubdivide {
    pub cuts: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenBoolean {
    pub operation: String,
    pub target: String,
}

/// Attachment entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmatureDrivenAttachment {
    Primitive(ArmatureDrivenPrimitiveAttachment),
    Extrude {
        extrude: ArmatureDrivenExtrudeAttachment,
    },
    Asset(ArmatureDrivenAssetAttachment),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenPrimitiveAttachment {
    pub primitive: MeshPrimitive,

    /// Dimensions in bone-relative units.
    pub dimensions: [f64; 3],

    /// Offset in bone-relative units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,

    /// Rotation (degrees).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenExtrudeAttachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    pub start: [f64; 3],
    pub end: [f64; 3],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_radius: Option<BoneRelativeLength>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub taper: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenAssetAttachment {
    pub asset: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

/// Boolean shape definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmatureDrivenBoolShapeDef {
    Mirror(ArmatureDrivenMirrorRef),
    Shape(ArmatureDrivenBoolShape),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenBoolShape {
    pub primitive: MeshPrimitive,
    pub dimensions: [f64; 3],

    /// Position in bone-relative units.
    pub position: [f64; 3],

    /// Optional associated bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
}
