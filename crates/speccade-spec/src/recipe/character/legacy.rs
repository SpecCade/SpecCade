//! Legacy dict-based parts system (from ai-studio-core SPEC format).

use serde::{Deserialize, Serialize};

/// Legacy part definition matching ai-studio-core SPEC dict format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
