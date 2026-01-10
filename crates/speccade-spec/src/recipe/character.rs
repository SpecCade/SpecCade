//! Skeletal mesh (character) recipe types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::mesh::{MaterialSlot, MeshConstraints, MeshExportSettings, MeshPrimitive};

/// Parameters for the `skeletal_mesh.blender_rigged_mesh_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletalMeshBlenderRiggedMeshV1Params {
    /// Predefined skeleton rig.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,
    /// Custom skeleton definition (alternative to preset).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skeleton: Vec<SkeletonBone>,
    /// Body part mesh definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_parts: Vec<BodyPart>,
    /// Legacy parts definition (dict-style, keyed by part name).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parts: HashMap<String, LegacyPart>,
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
    /// Triangle budget for validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tri_budget: Option<u32>,
    /// Texturing options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texturing: Option<Texturing>,
}

// =============================================================================
// Skeleton Types
// =============================================================================

/// A bone in a custom skeleton definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonBone {
    /// Unique bone name.
    pub bone: String,
    /// Bone head position [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<[f64; 3]>,
    /// Bone tail position [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<[f64; 3]>,
    /// Parent bone name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Mirror from another bone (L->R reflection across X=0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<String>,
}

// =============================================================================
// Legacy Part Types (dict-style from ai-studio-core)
// =============================================================================

/// Legacy part definition matching ai-studio-core SPEC dict format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyPart {
    /// Associated bone name.
    pub bone: String,
    /// Base shape definition (e.g., "hexagon(6)", "circle(8)").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Base radius - uniform or tapered [bottom, top].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_radius: Option<BaseRadius>,
    /// Extrusion steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    /// Mirror from another part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<String>,
    /// Position offset [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    /// Initial rotation [X, Y, Z] in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    /// Cap the bottom face.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_start: Option<bool>,
    /// Cap the top face.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_end: Option<bool>,
    /// Skinning mode: "soft" or "rigid".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skinning_type: Option<SkinningType>,
    /// Thumb sub-parts for hands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<SubPartOrList>,
    /// Finger sub-parts for hands.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingers: Vec<SubPart>,
    /// Instanced copies of this part.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<Instance>,
}

/// Base radius can be uniform (single value) or tapered ([bottom, top]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BaseRadius {
    /// Uniform radius.
    Uniform(f64),
    /// Tapered radius [bottom, top].
    Tapered([f64; 2]),
}

impl Default for BaseRadius {
    fn default() -> Self {
        BaseRadius::Uniform(0.1)
    }
}

/// Skinning type for a part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinningType {
    /// Soft skinning with smooth weight blending.
    Soft,
    /// Rigid skinning with 100% weight to one bone.
    Rigid,
}

impl Default for SkinningType {
    fn default() -> Self {
        SkinningType::Soft
    }
}

/// Sub-part for thumbs/fingers - can be a single dict or list of dicts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubPartOrList {
    /// Single sub-part.
    Single(Box<SubPart>),
    /// List of sub-parts.
    List(Vec<SubPart>),
}

/// Sub-part definition (for thumbs, fingers, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubPart {
    /// Sub-part bone name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
    /// Base shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Base radius.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_radius: Option<BaseRadius>,
    /// Extrusion steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    /// Offset from parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    /// Rotation in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    /// Cap start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_start: Option<bool>,
    /// Cap end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_end: Option<bool>,
}

/// An instance of a part at a specific position and rotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    /// Instance position [X, Y, Z].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// Instance rotation [X, Y, Z] in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
}

// =============================================================================
// Step System
// =============================================================================

/// A step in the extrusion process.
/// Can be a string shorthand (e.g., "0.1") or a full step definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Step {
    /// Shorthand: just an extrusion distance as string.
    Shorthand(String),
    /// Full step definition.
    Full(StepDefinition),
}

/// Full step definition with all possible transformations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDefinition {
    /// Extrusion distance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extrude: Option<f64>,
    /// Scale factor - uniform or [X, Y].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<ScaleFactor>,
    /// Translation offset [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<[f64; 3]>,
    /// Rotation around Z axis in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<f64>,
    /// Asymmetric bulge [side, forward_back].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulge: Option<BulgeFactor>,
    /// Tilt rotation around X/Y axes in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt: Option<TiltFactor>,
}

/// Scale factor can be uniform or per-axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScaleFactor {
    /// Uniform scale.
    Uniform(f64),
    /// Per-axis scale [X, Y].
    PerAxis([f64; 2]),
}

impl Default for ScaleFactor {
    fn default() -> Self {
        ScaleFactor::Uniform(1.0)
    }
}

/// Bulge factor for asymmetric scaling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BulgeFactor {
    /// Uniform bulge.
    Uniform(f64),
    /// Asymmetric bulge [side, forward_back].
    Asymmetric([f64; 2]),
}

/// Tilt factor for rotation around X/Y axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TiltFactor {
    /// Uniform tilt (applied to X axis).
    Uniform(f64),
    /// Per-axis tilt [X, Y] in degrees.
    PerAxis([f64; 2]),
}

// =============================================================================
// Texturing
// =============================================================================

/// Texturing configuration for UV unwrapping and material regions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Texturing {
    /// UV unwrapping mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_mode: Option<UvMode>,
    /// Material region definitions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub regions: HashMap<String, TextureRegion>,
}

/// UV unwrapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvMode {
    /// Smart UV project (automatic island detection).
    SmartProject,
    /// Region-based UV mapping (manual region assignment).
    RegionBased,
    /// Lightmap pack.
    LightmapPack,
    /// Cube projection.
    CubeProject,
    /// Cylinder projection.
    CylinderProject,
    /// Sphere projection.
    SphereProject,
}

impl Default for UvMode {
    fn default() -> Self {
        UvMode::SmartProject
    }
}

/// A texture region definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureRegion {
    /// Parts included in this region.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
    /// Material index for this region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,
    /// UV island index hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island: Option<u32>,
    /// Color for this region (hex string or [R, G, B]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<RegionColor>,
}

/// Color specification for a region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegionColor {
    /// Hex color string (e.g., "#FF0000").
    Hex(String),
    /// RGB array [R, G, B] with values 0-1.
    Rgb([f64; 3]),
    /// RGBA array [R, G, B, A] with values 0-1.
    Rgba([f64; 4]),
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
    /// Save .blend file alongside GLB output.
    #[serde(default)]
    pub save_blend: bool,
}

impl Default for SkeletalMeshExportSettings {
    fn default() -> Self {
        Self {
            include_armature: true,
            include_normals: true,
            include_uvs: true,
            triangulate: true,
            include_skin_weights: true,
            save_blend: false,
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
            tangents: false,
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

    // =============================================================================
    // New Legacy Part System Tests
    // =============================================================================

    #[test]
    fn test_skeleton_bone_with_mirror() {
        let json = r#"{
            "bone": "arm_r",
            "mirror": "arm_l"
        }"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.bone, "arm_r");
        assert_eq!(bone.mirror, Some("arm_l".to_string()));
        assert!(bone.head.is_none());
        assert!(bone.tail.is_none());
    }

    #[test]
    fn test_skeleton_bone_with_positions() {
        let json = r#"{
            "bone": "spine",
            "head": [0, 0, 1.0],
            "tail": [0, 0, 1.2],
            "parent": "hips"
        }"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.bone, "spine");
        assert_eq!(bone.head, Some([0.0, 0.0, 1.0]));
        assert_eq!(bone.tail, Some([0.0, 0.0, 1.2]));
        assert_eq!(bone.parent, Some("hips".to_string()));
    }

    #[test]
    fn test_base_radius_uniform() {
        let json = r#"0.15"#;
        let radius: BaseRadius = serde_json::from_str(json).unwrap();
        assert_eq!(radius, BaseRadius::Uniform(0.15));
    }

    #[test]
    fn test_base_radius_tapered() {
        let json = r#"[0.1, 0.05]"#;
        let radius: BaseRadius = serde_json::from_str(json).unwrap();
        assert_eq!(radius, BaseRadius::Tapered([0.1, 0.05]));
    }

    #[test]
    fn test_step_shorthand() {
        let json = r#""0.1""#;
        let step: Step = serde_json::from_str(json).unwrap();
        assert!(matches!(step, Step::Shorthand(s) if s == "0.1"));
    }

    #[test]
    fn test_step_full_definition() {
        let json = r#"{
            "extrude": 0.15,
            "scale": 0.8,
            "rotate": 15.0,
            "bulge": [1.1, 0.9],
            "tilt": [5.0, -3.0]
        }"#;
        let step: Step = serde_json::from_str(json).unwrap();
        if let Step::Full(def) = step {
            assert_eq!(def.extrude, Some(0.15));
            assert_eq!(def.scale, Some(ScaleFactor::Uniform(0.8)));
            assert_eq!(def.rotate, Some(15.0));
            assert_eq!(def.bulge, Some(BulgeFactor::Asymmetric([1.1, 0.9])));
            assert_eq!(def.tilt, Some(TiltFactor::PerAxis([5.0, -3.0])));
        } else {
            panic!("Expected Full step definition");
        }
    }

    #[test]
    fn test_step_with_translate() {
        let json = r#"{
            "extrude": 0.1,
            "translate": [0.05, 0.0, 0.0]
        }"#;
        let step: Step = serde_json::from_str(json).unwrap();
        if let Step::Full(def) = step {
            assert_eq!(def.translate, Some([0.05, 0.0, 0.0]));
        } else {
            panic!("Expected Full step definition");
        }
    }

    #[test]
    fn test_scale_factor_uniform() {
        let json = r#"0.9"#;
        let scale: ScaleFactor = serde_json::from_str(json).unwrap();
        assert_eq!(scale, ScaleFactor::Uniform(0.9));
    }

    #[test]
    fn test_scale_factor_per_axis() {
        let json = r#"[0.8, 1.2]"#;
        let scale: ScaleFactor = serde_json::from_str(json).unwrap();
        assert_eq!(scale, ScaleFactor::PerAxis([0.8, 1.2]));
    }

    #[test]
    fn test_legacy_part_basic() {
        let json = r#"{
            "bone": "chest",
            "base": "hexagon(6)",
            "base_radius": 0.2,
            "cap_start": true,
            "cap_end": false
        }"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.bone, "chest");
        assert_eq!(part.base, Some("hexagon(6)".to_string()));
        assert_eq!(part.base_radius, Some(BaseRadius::Uniform(0.2)));
        assert_eq!(part.cap_start, Some(true));
        assert_eq!(part.cap_end, Some(false));
    }

    #[test]
    fn test_legacy_part_with_steps() {
        let json = r#"{
            "bone": "arm_upper_l",
            "base": "circle(8)",
            "base_radius": [0.08, 0.06],
            "steps": [
                "0.1",
                {"extrude": 0.15, "scale": 0.9},
                {"extrude": 0.1, "scale": 0.8, "rotate": 10}
            ]
        }"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.steps.len(), 3);
        assert!(matches!(&part.steps[0], Step::Shorthand(s) if s == "0.1"));
        assert!(matches!(&part.steps[1], Step::Full(_)));
        assert!(matches!(&part.steps[2], Step::Full(_)));
    }

    #[test]
    fn test_legacy_part_with_mirror() {
        let json = r#"{
            "bone": "arm_upper_r",
            "mirror": "arm_upper_l"
        }"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.mirror, Some("arm_upper_l".to_string()));
    }

    #[test]
    fn test_instance() {
        let json = r#"{
            "position": [0.1, 0.0, 0.5],
            "rotation": [0, 45, 0]
        }"#;
        let instance: Instance = serde_json::from_str(json).unwrap();
        assert_eq!(instance.position, Some([0.1, 0.0, 0.5]));
        assert_eq!(instance.rotation, Some([0.0, 45.0, 0.0]));
    }

    #[test]
    fn test_legacy_part_with_instances() {
        let json = r#"{
            "bone": "spike",
            "base": "circle(6)",
            "base_radius": 0.05,
            "instances": [
                {"position": [0.0, 0.0, 0.1]},
                {"position": [0.1, 0.0, 0.1], "rotation": [0, 0, 30]},
                {"position": [-0.1, 0.0, 0.1], "rotation": [0, 0, -30]}
            ]
        }"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.instances.len(), 3);
        assert_eq!(part.instances[0].position, Some([0.0, 0.0, 0.1]));
        assert_eq!(part.instances[1].rotation, Some([0.0, 0.0, 30.0]));
    }

    #[test]
    fn test_sub_part() {
        let json = r#"{
            "bone": "finger_index_1",
            "base": "circle(4)",
            "base_radius": 0.02,
            "steps": [{"extrude": 0.03, "scale": 0.9}],
            "cap_start": false,
            "cap_end": true
        }"#;
        let sub: SubPart = serde_json::from_str(json).unwrap();
        assert_eq!(sub.bone, Some("finger_index_1".to_string()));
        assert_eq!(sub.cap_end, Some(true));
    }

    #[test]
    fn test_sub_part_or_list_single() {
        let json = r#"{"bone": "thumb_1", "base_radius": 0.015}"#;
        let sub: SubPartOrList = serde_json::from_str(json).unwrap();
        assert!(matches!(sub, SubPartOrList::Single(_)));
    }

    #[test]
    fn test_sub_part_or_list_multiple() {
        let json = r#"[
            {"bone": "thumb_1", "base_radius": 0.015},
            {"bone": "thumb_2", "base_radius": 0.012}
        ]"#;
        let sub: SubPartOrList = serde_json::from_str(json).unwrap();
        if let SubPartOrList::List(list) = sub {
            assert_eq!(list.len(), 2);
        } else {
            panic!("Expected List variant");
        }
    }

    #[test]
    fn test_legacy_part_with_fingers() {
        let json = r#"{
            "bone": "hand_l",
            "base": "circle(8)",
            "base_radius": 0.04,
            "thumb": {"bone": "thumb_l", "base_radius": 0.015},
            "fingers": [
                {"bone": "finger_index_l", "base_radius": 0.012},
                {"bone": "finger_middle_l", "base_radius": 0.012},
                {"bone": "finger_ring_l", "base_radius": 0.011},
                {"bone": "finger_pinky_l", "base_radius": 0.01}
            ]
        }"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert!(part.thumb.is_some());
        assert_eq!(part.fingers.len(), 4);
    }

    #[test]
    fn test_texturing_basic() {
        let json = r#"{
            "uv_mode": "smart_project"
        }"#;
        let tex: Texturing = serde_json::from_str(json).unwrap();
        assert_eq!(tex.uv_mode, Some(UvMode::SmartProject));
    }

    #[test]
    fn test_texturing_with_regions() {
        let json = r##"{
            "uv_mode": "region_based",
            "regions": {
                "body": {
                    "parts": ["torso", "hips"],
                    "material_index": 0,
                    "color": "#FF5500"
                },
                "limbs": {
                    "parts": ["arm_l", "arm_r", "leg_l", "leg_r"],
                    "material_index": 1,
                    "color": [0.8, 0.6, 0.4]
                }
            }
        }"##;
        let tex: Texturing = serde_json::from_str(json).unwrap();
        assert_eq!(tex.uv_mode, Some(UvMode::RegionBased));
        assert_eq!(tex.regions.len(), 2);
        assert!(tex.regions.contains_key("body"));
        assert!(tex.regions.contains_key("limbs"));

        let body = tex.regions.get("body").unwrap();
        assert_eq!(body.parts, vec!["torso", "hips"]);
        assert!(matches!(&body.color, Some(RegionColor::Hex(s)) if s == "#FF5500"));
    }

    #[test]
    fn test_uv_modes() {
        let modes = vec![
            (r#""smart_project""#, UvMode::SmartProject),
            (r#""region_based""#, UvMode::RegionBased),
            (r#""lightmap_pack""#, UvMode::LightmapPack),
            (r#""cube_project""#, UvMode::CubeProject),
            (r#""cylinder_project""#, UvMode::CylinderProject),
            (r#""sphere_project""#, UvMode::SphereProject),
        ];
        for (json, expected) in modes {
            let mode: UvMode = serde_json::from_str(json).unwrap();
            assert_eq!(mode, expected);
        }
    }

    #[test]
    fn test_region_color_variants() {
        // Hex
        let hex: RegionColor = serde_json::from_str(r##""#AABBCC""##).unwrap();
        assert!(matches!(hex, RegionColor::Hex(s) if s == "#AABBCC"));

        // RGB
        let rgb: RegionColor = serde_json::from_str(r#"[1.0, 0.5, 0.25]"#).unwrap();
        assert!(matches!(rgb, RegionColor::Rgb([1.0, 0.5, 0.25])));

        // RGBA
        let rgba: RegionColor = serde_json::from_str(r#"[1.0, 0.5, 0.25, 0.8]"#).unwrap();
        assert!(matches!(rgba, RegionColor::Rgba([1.0, 0.5, 0.25, 0.8])));
    }

    #[test]
    fn test_skinning_type() {
        let soft: SkinningType = serde_json::from_str(r#""soft""#).unwrap();
        assert_eq!(soft, SkinningType::Soft);

        let rigid: SkinningType = serde_json::from_str(r#""rigid""#).unwrap();
        assert_eq!(rigid, SkinningType::Rigid);
    }

    #[test]
    fn test_full_character_spec() {
        let json = r#"{
            "skeleton": [
                {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.1]},
                {"bone": "spine", "head": [0, 0, 0.1], "tail": [0, 0, 0.3], "parent": "root"},
                {"bone": "arm_l", "head": [0.1, 0, 0.25], "tail": [0.3, 0, 0.25], "parent": "spine"},
                {"bone": "arm_r", "mirror": "arm_l"}
            ],
            "parts": {
                "torso": {
                    "bone": "spine",
                    "base": "hexagon(6)",
                    "base_radius": [0.15, 0.12],
                    "steps": [
                        {"extrude": 0.1, "scale": 0.95},
                        {"extrude": 0.1, "scale": 0.9}
                    ],
                    "cap_start": true,
                    "cap_end": true
                },
                "arm_l": {
                    "bone": "arm_l",
                    "base": "circle(8)",
                    "base_radius": 0.05,
                    "steps": [{"extrude": 0.2, "scale": 0.8}],
                    "skinning_type": "soft"
                },
                "arm_r": {
                    "bone": "arm_r",
                    "mirror": "arm_l"
                }
            },
            "tri_budget": 500,
            "texturing": {
                "uv_mode": "smart_project",
                "regions": {
                    "body": {"parts": ["torso"], "material_index": 0}
                }
            }
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.skeleton.len(), 4);
        assert_eq!(params.parts.len(), 3);
        assert_eq!(params.tri_budget, Some(500));
        assert!(params.texturing.is_some());

        // Check skeleton mirror
        let arm_r = params.skeleton.iter().find(|b| b.bone == "arm_r").unwrap();
        assert_eq!(arm_r.mirror, Some("arm_l".to_string()));

        // Check parts mirror
        let arm_r_part = params.parts.get("arm_r").unwrap();
        assert_eq!(arm_r_part.mirror, Some("arm_l".to_string()));
    }

    #[test]
    fn test_bulge_factor_variants() {
        let uniform: BulgeFactor = serde_json::from_str("1.2").unwrap();
        assert_eq!(uniform, BulgeFactor::Uniform(1.2));

        let asymmetric: BulgeFactor = serde_json::from_str("[1.1, 0.9]").unwrap();
        assert_eq!(asymmetric, BulgeFactor::Asymmetric([1.1, 0.9]));
    }

    #[test]
    fn test_tilt_factor_variants() {
        let uniform: TiltFactor = serde_json::from_str("5.0").unwrap();
        assert_eq!(uniform, TiltFactor::Uniform(5.0));

        let per_axis: TiltFactor = serde_json::from_str("[10.0, -5.0]").unwrap();
        assert_eq!(per_axis, TiltFactor::PerAxis([10.0, -5.0]));
    }

    // =============================================================================
    // Comprehensive Parity Matrix Coverage Tests
    // =============================================================================

    /// Test: Top-level key 'name'
    #[test]
    fn test_parity_name() {
        let json = r#"{
            "skeleton": [],
            "parts": {},
            "tri_budget": 100
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        // Name is not directly in params, it's in the asset wrapper
        assert!(params.skeleton.is_empty());
    }

    /// Test: Top-level key 'tri_budget'
    #[test]
    fn test_parity_tri_budget() {
        let json = r#"{"skeleton": [], "parts": {}, "tri_budget": 500}"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.tri_budget, Some(500));
    }

    /// Test: Top-level key 'skeleton' (array)
    #[test]
    fn test_parity_skeleton_array() {
        let json = r#"{
            "skeleton": [
                {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 1]}
            ],
            "parts": {}
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.skeleton.len(), 1);
        assert_eq!(params.skeleton[0].bone, "root");
    }

    /// Test: Top-level key 'skeleton_preset'
    #[test]
    fn test_parity_skeleton_preset() {
        let json = r#"{
            "skeleton_preset": "humanoid_basic_v1",
            "parts": {}
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.skeleton_preset, Some(SkeletonPreset::HumanoidBasicV1));
        assert!(params.skeleton.is_empty());
    }

    /// Test: Top-level key 'parts' (dict)
    #[test]
    fn test_parity_parts_dict() {
        let json = r#"{
            "skeleton": [],
            "parts": {
                "head": {
                    "bone": "head_bone",
                    "base": "circle(8)",
                    "base_radius": 0.1
                }
            }
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.parts.len(), 1);
        assert!(params.parts.contains_key("head"));
    }

    /// Test: Top-level key 'body_parts' (array, modern style)
    #[test]
    fn test_parity_body_parts_array() {
        let json = r#"{
            "skeleton": [],
            "body_parts": [
                {
                    "bone": "head",
                    "mesh": {
                        "primitive": "sphere",
                        "dimensions": [0.2, 0.2, 0.2]
                    }
                }
            ],
            "parts": {}
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.body_parts.len(), 1);
        assert_eq!(params.body_parts[0].bone, "head");
    }

    /// Test: Top-level key 'texturing'
    #[test]
    fn test_parity_texturing() {
        let json = r#"{
            "skeleton": [],
            "parts": {},
            "texturing": {"uv_mode": "smart_project"}
        }"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert!(params.texturing.is_some());
    }

    /// Test: Skeleton bone key 'bone' (required)
    #[test]
    fn test_parity_skeleton_bone_name() {
        let bone = SkeletonBone {
            bone: "test_bone".to_string(),
            head: None,
            tail: None,
            parent: None,
            mirror: None,
        };
        let json = serde_json::to_string(&bone).unwrap();
        assert!(json.contains("test_bone"));
    }

    /// Test: Skeleton bone key 'head'
    #[test]
    fn test_parity_skeleton_bone_head() {
        let json = r#"{"bone": "test", "head": [1.0, 2.0, 3.0]}"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.head, Some([1.0, 2.0, 3.0]));
    }

    /// Test: Skeleton bone key 'tail'
    #[test]
    fn test_parity_skeleton_bone_tail() {
        let json = r#"{"bone": "test", "tail": [4.0, 5.0, 6.0]}"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.tail, Some([4.0, 5.0, 6.0]));
    }

    /// Test: Skeleton bone key 'parent'
    #[test]
    fn test_parity_skeleton_bone_parent() {
        let json = r#"{"bone": "child", "parent": "parent_bone"}"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.parent, Some("parent_bone".to_string()));
    }

    /// Test: Skeleton bone key 'mirror'
    #[test]
    fn test_parity_skeleton_bone_mirror() {
        let json = r#"{"bone": "arm_r", "mirror": "arm_l"}"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert_eq!(bone.mirror, Some("arm_l".to_string()));
    }

    /// Test: Part key 'bone' (required)
    #[test]
    fn test_parity_part_bone() {
        let json = r#"{"bone": "torso", "base": "hexagon(6)", "base_radius": 0.1}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.bone, "torso");
    }

    /// Test: Part key 'base'
    #[test]
    fn test_parity_part_base() {
        let json = r#"{"bone": "test", "base": "hexagon(6)"}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.base, Some("hexagon(6)".to_string()));
    }

    /// Test: Part key 'base_radius' (uniform)
    #[test]
    fn test_parity_part_base_radius_uniform() {
        let json = r#"{"bone": "test", "base_radius": 0.15}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.base_radius, Some(BaseRadius::Uniform(0.15)));
    }

    /// Test: Part key 'base_radius' (tapered array)
    #[test]
    fn test_parity_part_base_radius_tapered() {
        let json = r#"{"bone": "test", "base_radius": [0.2, 0.1]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.base_radius, Some(BaseRadius::Tapered([0.2, 0.1])));
    }

    /// Test: Part key 'steps'
    #[test]
    fn test_parity_part_steps() {
        let json = r#"{"bone": "test", "steps": ["0.1", {"extrude": 0.2}]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.steps.len(), 2);
    }

    /// Test: Part key 'mirror'
    #[test]
    fn test_parity_part_mirror() {
        let json = r#"{"bone": "arm_r", "mirror": "arm_l"}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.mirror, Some("arm_l".to_string()));
    }

    /// Test: Part key 'offset'
    #[test]
    fn test_parity_part_offset() {
        let json = r#"{"bone": "test", "offset": [0.5, 0.0, 0.1]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.offset, Some([0.5, 0.0, 0.1]));
    }

    /// Test: Part key 'rotation'
    #[test]
    fn test_parity_part_rotation() {
        let json = r#"{"bone": "test", "rotation": [45.0, 0.0, -30.0]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.rotation, Some([45.0, 0.0, -30.0]));
    }

    /// Test: Part key 'cap_start'
    #[test]
    fn test_parity_part_cap_start() {
        let json = r#"{"bone": "test", "cap_start": false}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.cap_start, Some(false));
    }

    /// Test: Part key 'cap_end'
    #[test]
    fn test_parity_part_cap_end() {
        let json = r#"{"bone": "test", "cap_end": true}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.cap_end, Some(true));
    }

    /// Test: Part key 'skinning_type'
    #[test]
    fn test_parity_part_skinning_type() {
        let json = r#"{"bone": "test", "skinning_type": "rigid"}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.skinning_type, Some(SkinningType::Rigid));
    }

    /// Test: Part key 'thumb'
    #[test]
    fn test_parity_part_thumb() {
        let json = r#"{"bone": "hand", "thumb": {"bone": "thumb_1", "base_radius": 0.015}}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert!(part.thumb.is_some());
    }

    /// Test: Part key 'fingers'
    #[test]
    fn test_parity_part_fingers() {
        let json = r#"{"bone": "hand", "fingers": [{"bone": "index_1"}, {"bone": "middle_1"}]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.fingers.len(), 2);
    }

    /// Test: Part key 'instances'
    #[test]
    fn test_parity_part_instances() {
        let json = r#"{"bone": "spike", "instances": [{"position": [0, 0, 1]}]}"#;
        let part: LegacyPart = serde_json::from_str(json).unwrap();
        assert_eq!(part.instances.len(), 1);
    }

    /// Test: Step key 'extrude'
    #[test]
    fn test_parity_step_extrude() {
        let json = r#"{"extrude": 0.25}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.extrude, Some(0.25));
    }

    /// Test: Step key 'scale' (uniform)
    #[test]
    fn test_parity_step_scale_uniform() {
        let json = r#"{"scale": 0.8}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.scale, Some(ScaleFactor::Uniform(0.8)));
    }

    /// Test: Step key 'scale' (per-axis)
    #[test]
    fn test_parity_step_scale_per_axis() {
        let json = r#"{"scale": [1.2, 0.9]}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.scale, Some(ScaleFactor::PerAxis([1.2, 0.9])));
    }

    /// Test: Step key 'translate'
    #[test]
    fn test_parity_step_translate() {
        let json = r#"{"translate": [0.1, 0.2, 0.3]}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.translate, Some([0.1, 0.2, 0.3]));
    }

    /// Test: Step key 'rotate'
    #[test]
    fn test_parity_step_rotate() {
        let json = r#"{"rotate": 45.0}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.rotate, Some(45.0));
    }

    /// Test: Step key 'bulge' (uniform)
    #[test]
    fn test_parity_step_bulge_uniform() {
        let json = r#"{"bulge": 1.3}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.bulge, Some(BulgeFactor::Uniform(1.3)));
    }

    /// Test: Step key 'bulge' (asymmetric)
    #[test]
    fn test_parity_step_bulge_asymmetric() {
        let json = r#"{"bulge": [1.1, 0.9]}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.bulge, Some(BulgeFactor::Asymmetric([1.1, 0.9])));
    }

    /// Test: Step key 'tilt' (uniform)
    #[test]
    fn test_parity_step_tilt_uniform() {
        let json = r#"{"tilt": 10.0}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.tilt, Some(TiltFactor::Uniform(10.0)));
    }

    /// Test: Step key 'tilt' (per-axis)
    #[test]
    fn test_parity_step_tilt_per_axis() {
        let json = r#"{"tilt": [15.0, -5.0]}"#;
        let step: StepDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(step.tilt, Some(TiltFactor::PerAxis([15.0, -5.0])));
    }

    /// Test: Instance key 'position'
    #[test]
    fn test_parity_instance_position() {
        let json = r#"{"position": [1.0, 2.0, 3.0]}"#;
        let instance: Instance = serde_json::from_str(json).unwrap();
        assert_eq!(instance.position, Some([1.0, 2.0, 3.0]));
    }

    /// Test: Instance key 'rotation'
    #[test]
    fn test_parity_instance_rotation() {
        let json = r#"{"rotation": [90.0, 0.0, 45.0]}"#;
        let instance: Instance = serde_json::from_str(json).unwrap();
        assert_eq!(instance.rotation, Some([90.0, 0.0, 45.0]));
    }

    /// Test: Texturing key 'uv_mode'
    #[test]
    fn test_parity_texturing_uv_mode() {
        let json = r#"{"uv_mode": "region_based"}"#;
        let tex: Texturing = serde_json::from_str(json).unwrap();
        assert_eq!(tex.uv_mode, Some(UvMode::RegionBased));
    }

    /// Test: Texturing key 'regions'
    #[test]
    fn test_parity_texturing_regions() {
        let json = r#"{
            "regions": {
                "head": {"parts": ["head_part"]},
                "body": {"parts": ["torso", "hips"]}
            }
        }"#;
        let tex: Texturing = serde_json::from_str(json).unwrap();
        assert_eq!(tex.regions.len(), 2);
        assert!(tex.regions.contains_key("head"));
        assert!(tex.regions.contains_key("body"));
    }

    /// Test: Skinning settings key 'max_bone_influences'
    #[test]
    fn test_parity_skinning_max_bone_influences() {
        let json = r#"{"max_bone_influences": 8}"#;
        let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
        assert_eq!(skinning.max_bone_influences, 8);
    }

    /// Test: Export settings key 'save_blend'
    #[test]
    fn test_parity_export_save_blend() {
        let json = r#"{"save_blend": true}"#;
        let export: SkeletalMeshExportSettings = serde_json::from_str(json).unwrap();
        assert_eq!(export.save_blend, true);
    }

    /// Test: Export settings key 'include_armature'
    #[test]
    fn test_parity_export_include_armature() {
        let json = r#"{"include_armature": false}"#;
        let export: SkeletalMeshExportSettings = serde_json::from_str(json).unwrap();
        assert_eq!(export.include_armature, false);
    }

    /// Test: Body part mesh key 'primitive'
    #[test]
    fn test_parity_body_part_primitive() {
        let json = r#"{
            "primitive": "cylinder",
            "dimensions": [0.1, 0.1, 0.5]
        }"#;
        let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
        assert_eq!(mesh.primitive, MeshPrimitive::Cylinder);
    }

    /// Test: Body part mesh key 'dimensions'
    #[test]
    fn test_parity_body_part_dimensions() {
        let json = r#"{
            "primitive": "cube",
            "dimensions": [0.3, 0.4, 0.5]
        }"#;
        let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
        assert_eq!(mesh.dimensions, [0.3, 0.4, 0.5]);
    }

    /// Test: Body part mesh key 'segments'
    #[test]
    fn test_parity_body_part_segments() {
        let json = r#"{
            "primitive": "sphere",
            "dimensions": [0.2, 0.2, 0.2],
            "segments": 16
        }"#;
        let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
        assert_eq!(mesh.segments, Some(16));
    }

    /// Test: Body part mesh key 'offset'
    #[test]
    fn test_parity_body_part_offset() {
        let json = r#"{
            "primitive": "cube",
            "dimensions": [1.0, 1.0, 1.0],
            "offset": [0.5, 0.0, 0.1]
        }"#;
        let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
        assert_eq!(mesh.offset, Some([0.5, 0.0, 0.1]));
    }

    /// Test: Body part mesh key 'rotation'
    #[test]
    fn test_parity_body_part_rotation() {
        let json = r#"{
            "primitive": "cube",
            "dimensions": [1.0, 1.0, 1.0],
            "rotation": [90.0, 0.0, 0.0]
        }"#;
        let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
        assert_eq!(mesh.rotation, Some([90.0, 0.0, 0.0]));
    }

    // =============================================================================
    // Validation Tests
    // =============================================================================

    /// Validation: Skeleton bone with neither head/tail nor mirror should be invalid in practice
    #[test]
    fn test_validation_skeleton_bone_incomplete() {
        // This tests that we can deserialize it, but validation would catch it
        let json = r#"{"bone": "incomplete"}"#;
        let bone: SkeletonBone = serde_json::from_str(json).unwrap();
        assert!(bone.head.is_none());
        assert!(bone.tail.is_none());
        assert!(bone.mirror.is_none());
    }

    /// Validation: Empty skeleton and empty parts
    #[test]
    fn test_validation_empty_character() {
        let json = r#"{"skeleton": [], "parts": {}}"#;
        let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
        assert!(params.skeleton.is_empty());
        assert!(params.parts.is_empty());
    }

    /// Validation: All UV modes
    #[test]
    fn test_validation_all_uv_modes() {
        let modes = [
            "smart_project",
            "region_based",
            "lightmap_pack",
            "cube_project",
            "cylinder_project",
            "sphere_project",
        ];
        for mode in &modes {
            let json = format!(r#"{{"uv_mode": "{}"}}"#, mode);
            let tex: Texturing = serde_json::from_str(&json).unwrap();
            assert!(tex.uv_mode.is_some());
        }
    }

    /// Validation: Skinning type values
    #[test]
    fn test_validation_skinning_types() {
        let soft: SkinningType = serde_json::from_str(r#""soft""#).unwrap();
        assert_eq!(soft, SkinningType::Soft);

        let rigid: SkinningType = serde_json::from_str(r#""rigid""#).unwrap();
        assert_eq!(rigid, SkinningType::Rigid);
    }

    /// Validation: Max bone influences bounds
    #[test]
    fn test_validation_max_bone_influences_bounds() {
        // Test valid values
        let json = r#"{"max_bone_influences": 1}"#;
        let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
        assert_eq!(skinning.max_bone_influences, 1);

        let json = r#"{"max_bone_influences": 8}"#;
        let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
        assert_eq!(skinning.max_bone_influences, 8);
    }

    /// Round-trip serialization test
    #[test]
    fn test_roundtrip_full_character() {
        let params = SkeletalMeshBlenderRiggedMeshV1Params {
            skeleton_preset: None,
            skeleton: vec![SkeletonBone {
                bone: "root".to_string(),
                head: Some([0.0, 0.0, 0.0]),
                tail: Some([0.0, 0.0, 1.0]),
                parent: None,
                mirror: None,
            }],
            body_parts: vec![],
            parts: {
                let mut map = HashMap::new();
                map.insert(
                    "torso".to_string(),
                    LegacyPart {
                        bone: "root".to_string(),
                        base: Some("hexagon(6)".to_string()),
                        base_radius: Some(BaseRadius::Uniform(0.2)),
                        steps: vec![Step::Full(StepDefinition {
                            extrude: Some(0.1),
                            scale: Some(ScaleFactor::Uniform(0.9)),
                            translate: None,
                            rotate: None,
                            bulge: None,
                            tilt: None,
                        })],
                        mirror: None,
                        offset: None,
                        rotation: None,
                        cap_start: Some(true),
                        cap_end: Some(true),
                        skinning_type: Some(SkinningType::Soft),
                        thumb: None,
                        fingers: vec![],
                        instances: vec![],
                    },
                );
                map
            },
            material_slots: vec![],
            skinning: Some(SkinningSettings {
                max_bone_influences: 4,
                auto_weights: true,
            }),
            export: Some(SkeletalMeshExportSettings {
                include_armature: true,
                include_normals: true,
                include_uvs: true,
                triangulate: true,
                include_skin_weights: true,
                save_blend: false,
            }),
            constraints: None,
            tri_budget: Some(500),
            texturing: Some(Texturing {
                uv_mode: Some(UvMode::SmartProject),
                regions: HashMap::new(),
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.skeleton.len(), 1);
        assert_eq!(parsed.parts.len(), 1);
        assert_eq!(parsed.tri_budget, Some(500));
    }
}
