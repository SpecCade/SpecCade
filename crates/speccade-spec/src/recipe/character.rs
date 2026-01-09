//! Skeletal mesh (character) recipe types.

use serde::{Deserialize, Serialize};

use super::mesh::{MaterialSlot, MeshConstraints, MeshExportSettings, MeshPrimitive};

/// Parameters for the `skeletal_mesh.blender_rigged_mesh_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletalMeshBlenderRiggedMeshV1Params {
    /// Predefined skeleton rig.
    pub skeleton_preset: SkeletonPreset,
    /// Body part mesh definitions.
    pub body_parts: Vec<BodyPart>,
    /// Material slot definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,
    /// Skinning settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skinning: Option<SkinningSettings>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<SkeletalMeshExportSettings>,
    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<SkeletalMeshConstraints>,
}

/// Predefined skeleton rigs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonPreset {
    /// Basic humanoid skeleton with 22 bones.
    HumanoidBasicV1,
}

impl SkeletonPreset {
    /// Returns the bone names for this skeleton preset.
    pub fn bone_names(&self) -> &'static [&'static str] {
        match self {
            SkeletonPreset::HumanoidBasicV1 => &[
                "root",
                "hips",
                "spine",
                "chest",
                "neck",
                "head",
                "shoulder_l",
                "upper_arm_l",
                "lower_arm_l",
                "hand_l",
                "shoulder_r",
                "upper_arm_r",
                "lower_arm_r",
                "hand_r",
                "upper_leg_l",
                "lower_leg_l",
                "foot_l",
                "upper_leg_r",
                "lower_leg_r",
                "foot_r",
            ],
        }
    }

    /// Returns the number of bones in this skeleton.
    pub fn bone_count(&self) -> usize {
        self.bone_names().len()
    }
}

/// Body part definition attached to a bone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyPart {
    /// Name of the bone this part is attached to.
    pub bone: String,
    /// Mesh configuration.
    pub mesh: BodyPartMesh,
    /// Optional material index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,
}

/// Mesh configuration for a body part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyPartMesh {
    /// Base primitive type.
    pub primitive: MeshPrimitive,
    /// Dimensions [X, Y, Z].
    pub dimensions: [f64; 3],
    /// Number of segments (for cylinders, spheres, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<u8>,
    /// Position offset from bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    /// Rotation in euler angles [X, Y, Z] degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
}

/// Skinning and weight painting settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkinningSettings {
    /// Maximum bone influences per vertex (1-8).
    #[serde(default = "default_max_bone_influences")]
    pub max_bone_influences: u8,
    /// Use automatic weight painting.
    #[serde(default = "default_true")]
    pub auto_weights: bool,
}

fn default_max_bone_influences() -> u8 {
    4
}

fn default_true() -> bool {
    true
}

impl Default for SkinningSettings {
    fn default() -> Self {
        Self {
            max_bone_influences: 4,
            auto_weights: true,
        }
    }
}

/// Export settings for skeletal meshes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletalMeshExportSettings {
    /// Include armature in export.
    #[serde(default = "default_true")]
    pub include_armature: bool,
    /// Include vertex normals.
    #[serde(default = "default_true")]
    pub include_normals: bool,
    /// Include UV coordinates.
    #[serde(default = "default_true")]
    pub include_uvs: bool,
    /// Triangulate mesh.
    #[serde(default = "default_true")]
    pub triangulate: bool,
    /// Include skin weights.
    #[serde(default = "default_true")]
    pub include_skin_weights: bool,
}

impl Default for SkeletalMeshExportSettings {
    fn default() -> Self {
        Self {
            include_armature: true,
            include_normals: true,
            include_uvs: true,
            triangulate: true,
            include_skin_weights: true,
        }
    }
}

impl From<SkeletalMeshExportSettings> for MeshExportSettings {
    fn from(settings: SkeletalMeshExportSettings) -> Self {
        Self {
            apply_modifiers: true,
            triangulate: settings.triangulate,
            include_normals: settings.include_normals,
            include_uvs: settings.include_uvs,
            include_vertex_colors: false,
        }
    }
}

/// Constraints for skeletal meshes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkeletalMeshConstraints {
    /// Maximum triangle count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_triangles: Option<u32>,
    /// Maximum bone count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bones: Option<u32>,
    /// Maximum number of materials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_materials: Option<u32>,
}

impl From<SkeletalMeshConstraints> for MeshConstraints {
    fn from(constraints: SkeletalMeshConstraints) -> Self {
        Self {
            max_triangles: constraints.max_triangles,
            max_materials: constraints.max_materials,
            max_vertices: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_preset_bones() {
        let preset = SkeletonPreset::HumanoidBasicV1;
        let bones = preset.bone_names();
        assert!(bones.contains(&"root"));
        assert!(bones.contains(&"head"));
        assert!(bones.contains(&"hand_l"));
        assert!(bones.contains(&"foot_r"));
    }

    #[test]
    fn test_body_part_serde() {
        let part = BodyPart {
            bone: "head".to_string(),
            mesh: BodyPartMesh {
                primitive: MeshPrimitive::Cube,
                dimensions: [0.25, 0.3, 0.25],
                segments: None,
                offset: None,
                rotation: None,
            },
            material_index: Some(0),
        };

        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("head"));
        assert!(json.contains("cube"));

        let parsed: BodyPart = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bone, "head");
    }

    #[test]
    fn test_skinning_settings_defaults() {
        let settings = SkinningSettings::default();
        assert_eq!(settings.max_bone_influences, 4);
        assert!(settings.auto_weights);
    }
}
