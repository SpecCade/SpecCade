//! Parameters and supporting types for `skeletal_mesh.armature_driven_v1`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::recipe::mesh::{MaterialSlot, MeshPrimitive};

use super::{SkeletalMeshConstraints, SkeletalMeshExportSettings, SkeletonBone, SkeletonPreset};

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

    /// End radius as multiplier of start radius.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taper: Option<f64>,

    /// Bone-relative translation offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<[f64; 3]>,

    /// Rotation (degrees) applied to the profile before extrusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<[f64; 3]>,

    /// Bulge control points along the bone axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bulge: Vec<ArmatureDrivenBulgePoint>,

    /// Twist (degrees) along the bone axis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twist: Option<f64>,

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
}

/// A bulge control point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmatureDrivenBulgePoint {
    /// Position along bone axis: 0.0 = head, 1.0 = tail.
    pub at: f64,
    /// Scale multiplier at that point.
    pub scale: f64,
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
